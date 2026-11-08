use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{Mutex, RwLock};

const BASE: &str = "https://api.warframe.market";
const USER_AGENT: &str = "WM-Market/0.1.0 (desktop development client)";

use crate::catalog_cache::{self, CatalogCache, CatalogDataStatus, CatalogSeed};

#[derive(Debug)]
pub enum MarketError {
    Network(String),
    Http(u16),
    InvalidResponse(String),
    UnknownWeapon(String),
    Validation(String),
    Cooldown(u64),
}
impl fmt::Display for MarketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(s) => write!(f, "网络请求失败：{s}"),
            Self::Http(s) => write!(f, "市场接口返回 HTTP {s}"),
            Self::InvalidResponse(s) => write!(f, "市场数据格式无效：{s}"),
            Self::UnknownWeapon(s) => write!(f, "未知紫卡武器：{s}"),
            Self::Validation(s) => write!(f, "筛选条件无效：{s}"),
            Self::Cooldown(s) => write!(f, "市场搜索冷却中，请等待 {s} 秒"),
        }
    }
}
impl std::error::Error for MarketError {}

#[derive(Clone)]
pub struct MarketService {
    client: reqwest::Client,
    base_url: String,
    catalog_cache: Arc<CatalogCache>,
    catalog_snapshot: Arc<RwLock<Option<MarketCatalog>>>,
    catalog_status: Arc<RwLock<Option<CatalogDataStatus>>>,
    catalog_initialized: Arc<Mutex<bool>>,
    update_gate: Arc<Mutex<()>>,
    request_gate: Arc<Mutex<Instant>>,
    search_gate: Arc<Mutex<Instant>>,
    blocked_until: Arc<Mutex<Option<Instant>>>,
}
impl MarketService {
    pub fn new() -> Self {
        Self::with_catalog_path_inner(None, BASE.into())
    }
    pub fn with_catalog_path(path: PathBuf) -> Self {
        Self::with_catalog_path_inner(Some(path), BASE.into())
    }
    #[cfg(test)]
    fn with_test_base(path: PathBuf, base_url: String) -> Self {
        Self::with_catalog_path_inner(Some(path), base_url)
    }
    fn with_catalog_path_inner(path: Option<PathBuf>, base_url: String) -> Self {
        let has_path = path.is_some();
        let seed = CatalogCache::seed().expect("embedded catalog seed");
        let catalog = catalog_from_seed(&seed.seed).expect("embedded catalog data");
        let status = catalog_cache::status_for(&seed.seed, "memory", None);
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .timeout(Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            base_url,
            catalog_cache: Arc::new(
                path.map(CatalogCache::sqlite)
                    .unwrap_or_else(CatalogCache::memory),
            ),
            catalog_snapshot: Arc::new(RwLock::new(Some(catalog))),
            catalog_status: Arc::new(RwLock::new(Some(status))),
            catalog_initialized: Arc::new(Mutex::new(!has_path)),
            update_gate: Arc::new(Mutex::new(())),
            request_gate: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(2))),
            search_gate: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(7))),
            blocked_until: Arc::new(Mutex::new(None)),
        }
    }
    async fn ensure_catalog(&self) -> Result<(), MarketError> {
        let mut initialized = self.catalog_initialized.lock().await;
        if *initialized {
            return Ok(());
        }
        let cache = self.catalog_cache.clone();
        let result = tokio::task::spawn_blocking(move || cache.load())
            .await
            .map_err(|e| MarketError::InvalidResponse(format!("目录缓存任务失败：{e}")))?;
        match result {
            Ok(Some(stored)) => match catalog_from_seed(&stored.seed) {
                Ok(catalog) => {
                    *self.catalog_snapshot.write().await = Some(catalog);
                    let mut status = stored.status;
                    if stored.recovered_from_previous {
                        status.last_error = Some("当前目录快照损坏，已回退到上一份有效目录".into());
                    }
                    *self.catalog_status.write().await = Some(status);
                }
                Err(e) => {
                    let mut status = self
                        .catalog_status
                        .read()
                        .await
                        .clone()
                        .expect("embedded catalog status");
                    status.last_error = Some("本地目录缓存内容无效，已回退内置目录".into());
                    *self.catalog_status.write().await = Some(status);
                    eprintln!("catalog cache parse failed: {e}");
                }
            },
            Ok(None) => {
                let seed = CatalogCache::seed().map_err(MarketError::InvalidResponse)?;
                let cache = self.catalog_cache.clone();
                let to_save = seed.clone();
                if let Err(e) = tokio::task::spawn_blocking(move || cache.save(&to_save))
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r)
                {
                    *self.catalog_status.write().await = Some(catalog_cache::status_for(
                        &seed.seed,
                        "memory",
                        Some("本地目录缓存无法写入，已使用内存目录".into()),
                    ));
                    eprintln!("catalog cache write failed: {e}");
                } else {
                    *self.catalog_status.write().await =
                        Some(catalog_cache::status_for(&seed.seed, "sqlite", None));
                }
            }
            Err(e) => {
                let status = self.catalog_status.read().await.clone().map(|mut s| {
                    s.last_error = Some("本地目录缓存读取失败，已使用内存目录".into());
                    s
                });
                eprintln!("catalog cache load failed: {e}");
                *self.catalog_status.write().await = status;
            }
        }
        *initialized = true;
        Ok(())
    }
    async fn gate(g: &Mutex<Instant>, d: Duration) {
        let mut last = g.lock().await;
        let e = last.elapsed();
        if e < d {
            tokio::time::sleep(d - e).await;
        }
        *last = Instant::now();
    }
    async fn blocked(&self) -> Result<(), MarketError> {
        if let Some(until) = *self.blocked_until.lock().await {
            let r = until.saturating_duration_since(Instant::now());
            if !r.is_zero() {
                return Err(MarketError::Cooldown(r.as_secs().saturating_add(1)));
            }
        }
        Ok(())
    }
    async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        headers: &[(&str, &str)],
    ) -> Result<T, MarketError> {
        self.blocked().await?;
        Self::gate(&self.request_gate, Duration::from_secs(1)).await;
        self.blocked().await?;
        let mut r = self.client.get(url).header("Accept", "application/json");
        for (k, v) in headers {
            r = r.header(*k, *v);
        }
        let response = r.send().await.map_err(|e| {
            MarketError::Network(if e.is_timeout() {
                "请求超时（15秒）".into()
            } else {
                e.to_string()
            })
        })?;
        let s = response.status();
        if s.as_u16() == 429 || s.as_u16() == 509 {
            let requested = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            let seconds = if Instant::now()
                .checked_add(Duration::from_secs(requested))
                .is_some()
            {
                requested
            } else {
                60
            };
            let deadline = Instant::now()
                .checked_add(Duration::from_secs(seconds))
                .unwrap_or_else(|| Instant::now() + Duration::from_secs(60));
            let mut b = self.blocked_until.lock().await;
            *b = Some(b.map(|x| x.max(deadline)).unwrap_or(deadline));
            return Err(MarketError::Cooldown(seconds));
        }
        if !s.is_success() {
            return Err(MarketError::Http(s.as_u16()));
        }
        response
            .json()
            .await
            .map_err(|e| MarketError::InvalidResponse(e.to_string()))
    }
    pub async fn weapons(&self) -> Result<Vec<RivenWeapon>, MarketError> {
        self.ensure_catalog().await?;
        self.catalog_snapshot
            .read()
            .await
            .as_ref()
            .map(|c| c.weapons.clone())
            .ok_or_else(|| MarketError::InvalidResponse("目录不可用".into()))
    }
    pub async fn attributes(&self) -> Result<Vec<RivenAttribute>, MarketError> {
        self.ensure_catalog().await?;
        self.catalog_snapshot
            .read()
            .await
            .as_ref()
            .map(|c| c.attributes.clone())
            .ok_or_else(|| MarketError::InvalidResponse("目录不可用".into()))
    }
    pub async fn catalog(&self) -> Result<MarketCatalog, MarketError> {
        self.ensure_catalog().await?;
        self.catalog_snapshot
            .read()
            .await
            .clone()
            .ok_or_else(|| MarketError::InvalidResponse("目录不可用".into()))
    }
    pub async fn search(&self, filters: MarketFilters) -> Result<MarketSnapshot, MarketError> {
        let catalog = self.catalog().await?;
        let weapons = catalog.weapons;
        let attrs = catalog.attributes;
        filters.validate(&weapons, &attrs)?;
        let weapon = weapons
            .iter()
            .find(|w| w.slug == filters.weapon_slug)
            .cloned()
            .ok_or_else(|| MarketError::UnknownWeapon(filters.weapon_slug.clone()))?;
        Self::gate(&self.search_gate, Duration::from_secs(6)).await;
        let started = Instant::now();
        let raw: V1Envelope = self
            .get_json(
                &build_search_url_with_base(&filters, &self.base_url),
                &[("Platform", "pc"), ("Crossplay", "false")],
            )
            .await?;
        let mut listings = normalize_response(raw, &filters.weapon_slug, &attrs)?;
        sort_listings(&mut listings, &filters.sort_order, &filters.positive_stats);
        let count = listings.len() as u32;
        Ok(MarketSnapshot {
            weapon,
            listings,
            fetched_at: now_ms(),
            elapsed_ms: started.elapsed().as_millis() as u64,
            batch_count: count,
            source: "Warframe.market".into(),
            platform: "pc".into(),
            filters,
        })
    }
    pub async fn catalog_status(&self) -> Result<CatalogDataStatus, MarketError> {
        self.ensure_catalog().await?;
        self.catalog_status
            .read()
            .await
            .clone()
            .ok_or_else(|| MarketError::InvalidResponse("目录状态不可用".into()))
    }
    pub async fn update_catalog(&self, force: bool) -> Result<CatalogUpdateResult, MarketError> {
        self.ensure_catalog().await?;
        let _update_guard = self.update_gate.lock().await;
        let before = self.fetch_collection_version().await?;
        let current = self.catalog_status.read().await.clone();
        if !force
            && current
                .as_ref()
                .and_then(|s| s.collection_version.as_deref())
                == Some(before.0.as_str())
        {
            let checked = catalog_cache::now_ms();
            if let Some(mut status) = current {
                status.last_checked_at = Some(checked);
                let cache = self.catalog_cache.clone();
                let copy = status.clone();
                tokio::task::spawn_blocking(move || cache.touch_checked(&copy))
                    .await
                    .map_err(|e| MarketError::InvalidResponse(format!("目录检查任务失败：{e}")))?
                    .map_err(|e| {
                        eprintln!("catalog check timestamp failed: {e}");
                        MarketError::InvalidResponse("本地目录缓存检查时间更新失败".into())
                    })?;
                *self.catalog_status.write().await = Some(status);
            }
            return Ok(CatalogUpdateResult {
                status: self.catalog_status().await?,
                updated: false,
                catalog: self.catalog().await?,
            });
        }
        let weapons: V2Envelope<Vec<serde_json::Value>> = self
            .get_json(
                &format!("{}/v2/riven/weapons", self.base_url),
                &[("Language", "zh-hans"), ("Platform", "pc")],
            )
            .await?;
        let attributes: V2Envelope<Vec<serde_json::Value>> = self
            .get_json(
                &format!("{}/v2/riven/attributes", self.base_url),
                &[("Language", "zh-hans"), ("Platform", "pc")],
            )
            .await?;
        let after = self.fetch_collection_version().await?;
        if before.0 != after.0 {
            return Err(MarketError::InvalidResponse(
                "目录版本在下载期间变化，请重试".into(),
            ));
        }
        let seed = CatalogSeed {
            schema_version: 1,
            source: BASE.into(),
            language: "zh-hans".into(),
            platform: "pc".into(),
            downloaded_at: catalog_cache::now_ms(),
            collection_version: before.0,
            api_version: after.1,
            game_version: None,
            source_updated_at: None,
            weapons: weapons
                .data
                .ok_or_else(|| MarketError::InvalidResponse("武器目录缺少 data".into()))?,
            attributes: attributes
                .data
                .ok_or_else(|| MarketError::InvalidResponse("词条目录缺少 data".into()))?,
        };
        catalog_cache::validate_seed(&seed).map_err(MarketError::InvalidResponse)?;
        let raw =
            serde_json::to_vec(&seed).map_err(|e| MarketError::InvalidResponse(e.to_string()))?;
        let catalog = catalog_from_seed(&seed)?;
        let mut status = catalog_cache::status_for(&seed, "sqlite", None);
        status.last_checked_at = Some(catalog_cache::now_ms());
        let stored = catalog_cache::StoredCatalog {
            seed,
            raw,
            status: status.clone(),
            recovered_from_previous: false,
        };
        let cache = self.catalog_cache.clone();
        tokio::task::spawn_blocking(move || cache.save(&stored))
            .await
            .map_err(|e| MarketError::InvalidResponse(format!("目录保存任务失败：{e}")))?
            .map_err(|e| {
                eprintln!("catalog save failed: {e}");
                MarketError::InvalidResponse("本地目录缓存保存失败".into())
            })?;
        *self.catalog_snapshot.write().await = Some(catalog.clone());
        *self.catalog_status.write().await = Some(status.clone());
        Ok(CatalogUpdateResult {
            status,
            updated: true,
            catalog,
        })
    }
    async fn fetch_collection_version(&self) -> Result<(String, String), MarketError> {
        let value: serde_json::Value = self
            .get_json(
                &format!("{}/v2/versions", self.base_url),
                &[("Language", "zh-hans"), ("Platform", "pc")],
            )
            .await?;
        let collections = value
            .get("data")
            .and_then(|v| v.get("collections"))
            .and_then(|v| v.get("rivens"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                value
                    .get("data")
                    .and_then(|v| v.get("collections"))
                    .and_then(|v| v.get("rivens"))
                    .and_then(|v| v.get("version"))
                    .and_then(|v| v.as_str())
            })
            .ok_or_else(|| {
                MarketError::InvalidResponse("版本响应缺少 collections.rivens".into())
            })?;
        let api_version = value
            .get("apiVersion")
            .and_then(|v| v.as_str())
            .or_else(|| {
                value
                    .get("data")
                    .and_then(|v| v.get("apiVersion"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("unknown");
        Ok((collections.into(), api_version.into()))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct V2Envelope<T> {
    pub data: Option<T>,
}
#[derive(Debug, Serialize, Clone)]
pub struct CatalogUpdateResult {
    pub status: CatalogDataStatus,
    pub updated: bool,
    pub catalog: MarketCatalog,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalizedName {
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RivenWeapon {
    pub id: String,
    pub slug: String,
    pub i18n: HashMap<String, LocalizedName>,
    #[serde(alias = "rivenType", default)]
    pub riven_type: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub disposition: Option<f64>,
    #[serde(alias = "gameRef", default)]
    pub game_ref: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RivenAttribute {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub english_name: String,
    pub unit: String,
    pub positive_only: bool,
    pub negative_only: bool,
    pub exclusive_to: Vec<String>,
    #[serde(default)]
    pub game_ref: String,
    #[serde(default)]
    pub positive_is_negative: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketCatalog {
    pub weapons: Vec<RivenWeapon>,
    pub attributes: Vec<RivenAttribute>,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct MarketFilters {
    pub weapon_slug: String,
    pub positive_stats: Vec<String>,
    pub negative_stat: Option<String>,
    pub sort_order: String,
    pub polarity: Option<String>,
    pub mastery_rank_max: Option<u32>,
    pub re_rolls_min: Option<u32>,
    pub re_rolls_max: Option<u32>,
    pub mod_rank: Option<String>,
}
impl MarketFilters {
    fn validate(
        &self,
        weapons: &[RivenWeapon],
        attrs: &[RivenAttribute],
    ) -> Result<(), MarketError> {
        let w = weapons
            .iter()
            .find(|x| x.slug == self.weapon_slug)
            .ok_or_else(|| MarketError::UnknownWeapon(self.weapon_slug.clone()))?;
        if !matches!(
            self.sort_order.as_str(),
            "asc" | "desc" | "positive_attr_asc" | "positive_attr_desc"
        ) {
            return Err(MarketError::Validation(
                "sort_order 必须为 asc、desc、positive_attr_asc 或 positive_attr_desc".into(),
            ));
        }
        if self.positive_stats.len() > 3 {
            return Err(MarketError::Validation("positive_stats 最多 3 项".into()));
        }
        if matches!(
            self.sort_order.as_str(),
            "positive_attr_asc" | "positive_attr_desc"
        ) && self.positive_stats.is_empty()
        {
            return Err(MarketError::Validation(
                "正面词条排序至少选择一个正面词条".into(),
            ));
        }
        let mut seen = HashSet::new();
        for s in &self.positive_stats {
            if !seen.insert(s) {
                return Err(MarketError::Validation("positive_stats 不能重复".into()));
            }
            validate_attr(s, true, w, attrs)?;
        }
        if let Some(s) = &self.negative_stat {
            if seen.contains(s) {
                return Err(MarketError::Validation("同一词条不能同时为正负面".into()));
            }
            validate_attr(s, false, w, attrs)?;
        }
        if let Some(p) = &self.polarity {
            if !["madurai", "vazarin", "naramon", "zenurik"].contains(&p.as_str()) {
                return Err(MarketError::Validation("polarity 无效".into()));
            }
        }
        if self
            .mastery_rank_max
            .map(|x| !(7..=16).contains(&x))
            .unwrap_or(false)
        {
            return Err(MarketError::Validation("mastery_rank 必须在 7..16".into()));
        }
        if let (Some(a), Some(b)) = (self.re_rolls_min, self.re_rolls_max) {
            if a > b {
                return Err(MarketError::Validation("re_rolls_min 不能大于 max".into()));
            }
        }
        if self.mod_rank.as_deref().is_some_and(|rank| rank != "maxed") {
            return Err(MarketError::Validation(
                "mod_rank 目前仅支持 maxed 或留空".into(),
            ));
        }
        Ok(())
    }
}
fn validate_attr(
    slug: &str,
    positive: bool,
    w: &RivenWeapon,
    attrs: &[RivenAttribute],
) -> Result<(), MarketError> {
    let a = attrs
        .iter()
        .find(|x| x.slug == slug)
        .ok_or_else(|| MarketError::Validation(format!("未知词条：{slug}")))?;
    if positive && a.negative_only {
        return Err(MarketError::Validation(format!("词条 {slug} 只能为负面")));
    }
    if !positive && a.positive_only {
        return Err(MarketError::Validation(format!("词条 {slug} 只能为正面")));
    }
    let weapon_type = if w.group.eq_ignore_ascii_case("archgun") {
        &w.group
    } else {
        &w.riven_type
    };
    if !a.exclusive_to.is_empty() && !a.exclusive_to.iter().any(|x| x == weapon_type) {
        return Err(MarketError::Validation(format!(
            "词条 {slug} 不适用于该武器"
        )));
    }
    Ok(())
}
fn build_search_url(f: &MarketFilters) -> String {
    build_search_url_with_base(f, BASE)
}
fn build_search_url_with_base(f: &MarketFilters, base: &str) -> String {
    let mut q = vec![
        "type=riven".into(),
        format!("weapon_url_name={}", encode_query(&f.weapon_slug)),
        format!("sort_by={}", sort_by_query(&f.sort_order)),
    ];
    if !f.positive_stats.is_empty() {
        q.push(format!(
            "positive_stats={}",
            encode_query(&f.positive_stats.join(","))
        ));
    }
    if let Some(v) = &f.negative_stat {
        q.push(format!("negative_stats={}", encode_query(v)));
    }
    if let Some(v) = &f.polarity {
        q.push(format!("polarity={}", encode_query(v)));
    }
    if let Some(v) = &f.mod_rank {
        q.push(format!("mod_rank={}", encode_query(v)));
    }
    for (k, v) in [
        ("mastery_rank_max", f.mastery_rank_max),
        ("re_rolls_min", f.re_rolls_min),
        ("re_rolls_max", f.re_rolls_max),
    ] {
        if let Some(v) = v {
            q.push(format!("{k}={v}"));
        }
    }
    format!("{base}/v1/auctions/search?{}", q.join("&"))
}
fn encode_query(v: &str) -> String {
    v.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
#[derive(Debug, Deserialize)]
struct V1Envelope {
    payload: Option<V1Payload>,
}
#[derive(Debug, Deserialize)]
struct V1Payload {
    auctions: Vec<ApiAuction>,
}
#[derive(Debug, Deserialize)]
struct ApiAuction {
    id: String,
    visible: bool,
    closed: bool,
    starting_price: Option<u32>,
    buyout_price: Option<u32>,
    owner: ApiOwner,
    item: ApiItem,
}
#[derive(Debug, Deserialize)]
struct ApiOwner {
    ingame_name: String,
    status: String,
}
#[derive(Debug, Deserialize)]
struct ApiItem {
    name: String,
    re_rolls: Option<u32>,
    mod_rank: Option<u32>,
    mastery_level: Option<u32>,
    polarity: Option<String>,
    attributes: Vec<ApiAttribute>,
    #[serde(rename = "type")]
    item_type: String,
    weapon_url_name: String,
}
#[derive(Debug, Deserialize)]
struct ApiAttribute {
    value: f64,
    positive: bool,
    url_name: String,
}
#[derive(Debug, Deserialize)]
struct ApiAttributeDef {
    slug: String,
    unit: Option<String>,
    i18n: HashMap<String, LocalizedName>,
    #[serde(rename = "positiveOnly", default)]
    positive_only: bool,
    #[serde(rename = "negativeOnly", default)]
    negative_only: bool,
    #[serde(rename = "exclusiveTo", default)]
    exclusive_to: Vec<String>,
    id: String,
    #[serde(rename = "gameRef", default)]
    game_ref: String,
    #[serde(rename = "positiveIsNegative", default)]
    positive_is_negative: bool,
}
pub(crate) fn catalog_from_seed(seed: &CatalogSeed) -> Result<MarketCatalog, MarketError> {
    let weapons = seed
        .weapons
        .iter()
        .cloned()
        .map(|value| {
            serde_json::from_value(value)
                .map_err(|e| MarketError::InvalidResponse(format!("武器目录字段无效：{e}")))
        })
        .collect::<Result<Vec<RivenWeapon>, _>>()?;
    let attributes = seed
        .attributes
        .iter()
        .cloned()
        .map(|value| {
            serde_json::from_value::<ApiAttributeDef>(value)
                .map(RivenAttribute::from_api)
                .map_err(|e| MarketError::InvalidResponse(format!("词条目录字段无效：{e}")))
        })
        .collect::<Result<Vec<RivenAttribute>, _>>()?;
    Ok(MarketCatalog {
        weapons,
        attributes,
    })
}
impl RivenAttribute {
    fn from_api(a: ApiAttributeDef) -> Self {
        let name = a
            .i18n
            .get("zh-hans")
            .or_else(|| a.i18n.get("en"))
            .map(|x| x.name.clone())
            .unwrap_or_else(|| a.slug.clone());
        let english_name = a
            .i18n
            .get("en")
            .map(|x| x.name.clone())
            .unwrap_or_else(|| name.clone());
        Self {
            id: a.id,
            slug: a.slug,
            name,
            english_name,
            unit: a.unit.unwrap_or_default(),
            positive_only: a.positive_only,
            negative_only: a.negative_only,
            exclusive_to: a.exclusive_to,
            game_ref: a.game_ref,
            positive_is_negative: a.positive_is_negative,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Listing {
    pub id: String,
    pub riven_name: String,
    pub price: Option<u32>,
    pub price_kind: String,
    pub seller: String,
    pub seller_status: String,
    pub rerolls: Option<u32>,
    pub mod_rank: Option<u32>,
    pub attributes: Vec<DisplayAttribute>,
    pub polarity: Option<String>,
    pub mastery_rank: Option<u32>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayAttribute {
    pub slug: String,
    pub name: String,
    pub value: f64,
    pub positive: bool,
    pub unit: String,
}
#[derive(Debug, Serialize, Clone)]
pub struct MarketSnapshot {
    pub weapon: RivenWeapon,
    pub listings: Vec<Listing>,
    pub fetched_at: u64,
    pub elapsed_ms: u64,
    pub batch_count: u32,
    pub source: String,
    pub platform: String,
    pub filters: MarketFilters,
}
fn normalize_response(
    raw: V1Envelope,
    slug: &str,
    attrs: &[RivenAttribute],
) -> Result<Vec<Listing>, MarketError> {
    let p = raw
        .payload
        .ok_or_else(|| MarketError::InvalidResponse("缺少 payload".into()))?;
    let mut seen = HashSet::new();
    Ok(p.auctions
        .into_iter()
        .filter(|a| {
            a.visible && !a.closed && a.item.item_type == "riven" && a.item.weapon_url_name == slug
        })
        .filter(|a| seen.insert(a.id.clone()))
        .map(|a| listing_from_api(a, attrs))
        .collect())
}
fn listing_from_api(a: ApiAuction, attrs: &[RivenAttribute]) -> Listing {
    let (price, kind) = a
        .buyout_price
        .map(|v| (Some(v), "一口价".into()))
        .or_else(|| a.starting_price.map(|v| (Some(v), "起拍价".into())))
        .unwrap_or((None, "暂无报价".into()));
    let mut names = HashMap::new();
    for x in attrs {
        names.insert(x.slug.clone(), (x.name.clone(), x.unit.clone()));
    }
    Listing {
        id: a.id,
        riven_name: a.item.name,
        price,
        price_kind: kind,
        seller: a.owner.ingame_name,
        seller_status: a.owner.status,
        rerolls: a.item.re_rolls,
        mod_rank: a.item.mod_rank,
        polarity: a.item.polarity,
        mastery_rank: a.item.mastery_level,
        attributes: a
            .item
            .attributes
            .into_iter()
            .map(|x| {
                let (n, u) = names
                    .get(&x.url_name)
                    .cloned()
                    .unwrap_or((x.url_name.clone(), String::new()));
                DisplayAttribute {
                    slug: x.url_name,
                    name: n,
                    value: x.value,
                    positive: x.positive,
                    unit: match u.as_str() {
                        "percent" => "%",
                        "multiply" => "倍",
                        "seconds" => "秒",
                        _ => u.as_str(),
                    }
                    .into(),
                }
            })
            .collect(),
    }
}
fn sort_by_query(order: &str) -> &str {
    match order {
        "asc" => "price_asc",
        "desc" => "price_desc",
        "positive_attr_asc" | "positive_attr_desc" => order,
        _ => "price_asc",
    }
}

fn sort_listings(listings: &mut [Listing], order: &str, positive_stats: &[String]) {
    if matches!(order, "asc" | "desc") {
        listings.sort_by(|a, b| price_cmp(a, b, order));
    } else if matches!(order, "positive_attr_asc" | "positive_attr_desc") {
        listings.sort_by(|a, b| positive_attr_cmp(a, b, positive_stats, order));
    }
}

fn positive_attr_cmp(
    a: &Listing,
    b: &Listing,
    positive_stats: &[String],
    order: &str,
) -> std::cmp::Ordering {
    for slug in positive_stats {
        let left = a
            .attributes
            .iter()
            .find(|attribute| attribute.positive && attribute.slug == *slug)
            .map(|attribute| attribute.value)
            .filter(|value| value.is_finite());
        let right = b
            .attributes
            .iter()
            .find(|attribute| attribute.positive && attribute.slug == *slug)
            .map(|attribute| attribute.value)
            .filter(|value| value.is_finite());
        let both_present = left.is_some() && right.is_some();
        let ordering = match (left, right) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
        };
        if ordering != std::cmp::Ordering::Equal {
            return if both_present && order == "positive_attr_desc" {
                ordering.reverse()
            } else {
                ordering
            };
        }
    }
    a.id.cmp(&b.id)
}

fn price_cmp(a: &Listing, b: &Listing, order: &str) -> std::cmp::Ordering {
    match (a.price, b.price) {
        (Some(x), Some(y)) => {
            let cmp = if order == "desc" {
                y.cmp(&x)
            } else {
                x.cmp(&y)
            };
            cmp.then_with(|| a.id.cmp(&b.id))
        }
        (None, None) => a.id.cmp(&b.id),
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
    }
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
        time::{Duration, Instant},
    };

    fn mock_server(responses: Vec<String>) -> (String, thread::JoinHandle<(usize, bool)>) {
        mock_server_codes(responses.into_iter().map(|body| (200, body)).collect())
    }

    fn mock_server_codes(
        responses: Vec<(u16, String)>,
    ) -> (String, thread::JoinHandle<(usize, bool)>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock listener");
        listener
            .set_nonblocking(true)
            .expect("nonblocking listener");
        let base = format!("http://{}", listener.local_addr().expect("mock addr"));
        let handle = thread::spawn(move || {
            let mut served = 0;
            let mut paths_ok = true;
            let expected_paths: Vec<&str> = if responses.len() == 1 {
                vec!["/v2/versions"]
            } else {
                vec![
                    "/v2/versions",
                    "/v2/riven/weapons",
                    "/v2/riven/attributes",
                    "/v2/versions",
                ]
            };
            let deadline = Instant::now() + Duration::from_secs(15);
            for (index, (code, body)) in responses.into_iter().enumerate() {
                let (mut stream, _) = loop {
                    match listener.accept() {
                        Ok(pair) => break pair,
                        Err(e)
                            if e.kind() == std::io::ErrorKind::WouldBlock
                                && Instant::now() < deadline =>
                        {
                            thread::sleep(Duration::from_millis(10))
                        }
                        Err(e) => panic!("mock request timeout/error: {e}"),
                    }
                };
                stream.set_nonblocking(false).expect("blocking stream");
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("read timeout");
                stream
                    .set_write_timeout(Some(Duration::from_secs(2)))
                    .expect("write timeout");
                let mut request = [0u8; 4096];
                let read = stream.read(&mut request).expect("mock read");
                let request_text = String::from_utf8_lossy(&request[..read]);
                if !request_text
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .contains(expected_paths[index])
                {
                    paths_ok = false;
                }
                let reason = if code == 200 {
                    "OK"
                } else {
                    "Internal Server Error"
                };
                let response = format!("HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", code, reason, body.len(), body);
                stream
                    .write_all(response.as_bytes())
                    .expect("mock response");
                served += 1;
            }
            (served, paths_ok)
        });
        (base, handle)
    }

    fn version_response(version: &str) -> String {
        serde_json::json!({"data":{"collections":{"rivens":version}},"apiVersion":"0.25.0"})
            .to_string()
    }

    fn catalog_response() -> (String, String) {
        let seed = CatalogCache::seed().expect("seed").seed;
        (
            serde_json::json!({"data":seed.weapons}).to_string(),
            serde_json::json!({"data":seed.attributes}).to_string(),
        )
    }

    #[test]
    fn update_same_version_only_checks_version() {
        tauri::async_runtime::block_on(async {
            let path = std::env::temp_dir().join(format!(
                "wm-market-update-same-{}-{}.sqlite3",
                std::process::id(),
                catalog_cache::now_ms()
            ));
            let service = MarketService::with_catalog_path(path.clone());
            let current = service.catalog_status().await.expect("seed status");
            let (base, server) = mock_server(vec![version_response(
                current.collection_version.as_deref().unwrap(),
            )]);
            let service = MarketService::with_test_base(path.clone(), base);
            let result = service.update_catalog(false).await.expect("same version");
            assert!(!result.updated);
            assert_eq!(result.catalog.weapons.len(), 420);
            assert_eq!(server.join().expect("mock join"), (1, true));
            let _ = std::fs::remove_file(path);
        });
    }

    #[test]
    fn force_update_downloads_and_persists_complete_catalog() {
        tauri::async_runtime::block_on(async {
            let path = std::env::temp_dir().join(format!(
                "wm-market-update-force-{}-{}.sqlite3",
                std::process::id(),
                catalog_cache::now_ms()
            ));
            let (weapons, attributes) = catalog_response();
            let version = CatalogCache::seed().expect("seed").seed.collection_version;
            let (base, server) = mock_server(vec![
                version_response(&version),
                weapons,
                attributes,
                version_response(&version),
            ]);
            let service = MarketService::with_test_base(path.clone(), base);
            let result = service.update_catalog(true).await.expect("force update");
            assert!(result.updated);
            assert_eq!(
                result.status.collection_version.as_deref(),
                Some(version.as_str())
            );
            assert_eq!(result.catalog.weapons.len(), 420);
            assert!(result.status.last_checked_at.is_some());
            assert_eq!(server.join().expect("mock join"), (4, true));
            let reopened = MarketService::with_catalog_path(path.clone());
            let catalog = reopened.catalog().await.expect("reopen updated catalog");
            assert_eq!(catalog.weapons.len(), 420);
            let reopened_status = reopened.catalog_status().await.expect("reopen status");
            assert_eq!(
                reopened_status.last_checked_at,
                result.status.last_checked_at
            );
            let _ = std::fs::remove_file(path);
        });
    }

    #[test]
    fn update_version_change_keeps_existing_catalog() {
        tauri::async_runtime::block_on(async {
            let path = std::env::temp_dir().join(format!(
                "wm-market-update-rollback-{}-{}.sqlite3",
                std::process::id(),
                catalog_cache::now_ms()
            ));
            let seed = CatalogCache::seed().expect("seed").seed;
            let old_version = seed.collection_version.clone();
            let (weapons, attributes) = catalog_response();
            let (base, server) = mock_server(vec![
                version_response(&old_version),
                weapons,
                attributes,
                version_response("changed"),
            ]);
            let service = MarketService::with_test_base(path.clone(), base);
            let result = service.update_catalog(true).await;
            assert!(result.is_err());
            assert_eq!(
                service
                    .catalog_status()
                    .await
                    .expect("status")
                    .collection_version
                    .as_deref(),
                Some(old_version.as_str())
            );
            assert_eq!(
                service.catalog().await.expect("old catalog").weapons.len(),
                420
            );
            assert_eq!(server.join().expect("mock join"), (4, true));
            let _ = std::fs::remove_file(path);
        });
    }

    #[test]
    fn update_changed_version_downloads_without_force() {
        tauri::async_runtime::block_on(async {
            let path = std::env::temp_dir().join(format!(
                "wm-market-update-changed-{}-{}.sqlite3",
                std::process::id(),
                catalog_cache::now_ms()
            ));
            let (weapons, attributes) = catalog_response();
            let (base, server) = mock_server(vec![
                version_response("new-version"),
                weapons,
                attributes,
                version_response("new-version"),
            ]);
            let service = MarketService::with_test_base(path.clone(), base);
            let result = service
                .update_catalog(false)
                .await
                .expect("changed version update");
            assert!(result.updated);
            assert_eq!(
                result.status.collection_version.as_deref(),
                Some("new-version")
            );
            assert!(result.status.last_checked_at.is_some());
            assert_eq!(server.join().expect("mock join"), (4, true));
            let reopened = MarketService::with_catalog_path(path.clone());
            let reopened_status = reopened.catalog_status().await.expect("reopen status");
            assert_eq!(
                reopened_status.last_checked_at,
                result.status.last_checked_at
            );
            let _ = std::fs::remove_file(path);
        });
    }

    #[test]
    fn update_http_failure_keeps_memory_and_sqlite_catalog() {
        tauri::async_runtime::block_on(async {
            let path = std::env::temp_dir().join(format!(
                "wm-market-update-http-failure-{}-{}.sqlite3",
                std::process::id(),
                catalog_cache::now_ms()
            ));
            let seed_version = CatalogCache::seed().expect("seed").seed.collection_version;
            let (weapons, _) = catalog_response();
            let (base, server) = mock_server_codes(vec![
                (200, version_response(&seed_version)),
                (200, weapons),
                (500, "{}".into()),
            ]);
            let service = MarketService::with_test_base(path.clone(), base);
            assert!(service.update_catalog(true).await.is_err());
            assert_eq!(
                service
                    .catalog_status()
                    .await
                    .expect("memory status")
                    .collection_version
                    .as_deref(),
                Some(seed_version.as_str())
            );
            assert_eq!(
                service
                    .catalog()
                    .await
                    .expect("memory catalog")
                    .weapons
                    .len(),
                420
            );
            assert_eq!(server.join().expect("mock join"), (3, true));
            let reopened = MarketService::with_catalog_path(path.clone());
            assert_eq!(
                reopened
                    .catalog()
                    .await
                    .expect("sqlite catalog")
                    .weapons
                    .len(),
                420
            );
            let _ = std::fs::remove_file(path);
        });
    }
    fn fixtures() -> (Vec<RivenWeapon>, Vec<RivenAttribute>) {
        (
            vec![RivenWeapon {
                id: "1".into(),
                slug: "torid".into(),
                i18n: HashMap::new(),
                riven_type: "rifle".into(),
                group: "".into(),
                disposition: None,
                game_ref: String::new(),
            }],
            vec![
                RivenAttribute {
                    id: "1".into(),
                    slug: "critical_chance".into(),
                    name: "暴击几率".into(),
                    english_name: "Critical Chance".into(),
                    unit: "percent".into(),
                    positive_only: false,
                    negative_only: false,
                    exclusive_to: vec![],
                    game_ref: String::new(),
                    positive_is_negative: false,
                },
                RivenAttribute {
                    id: "2".into(),
                    slug: "damage_vs_infested".into(),
                    name: "对 Infested 伤害".into(),
                    english_name: "Damage vs Infested".into(),
                    unit: "multiply".into(),
                    positive_only: false,
                    negative_only: false,
                    exclusive_to: vec![],
                    game_ref: String::new(),
                    positive_is_negative: false,
                },
                RivenAttribute {
                    id: "3".into(),
                    slug: "critical_damage".into(),
                    name: "暴击伤害".into(),
                    english_name: "Critical Damage".into(),
                    unit: "percent".into(),
                    positive_only: false,
                    negative_only: false,
                    exclusive_to: vec![],
                    game_ref: String::new(),
                    positive_is_negative: false,
                },
            ],
        )
    }
    fn base() -> MarketFilters {
        MarketFilters {
            weapon_slug: "torid".into(),
            positive_stats: vec!["critical_chance".into()],
            negative_stat: Some("damage_vs_infested".into()),
            sort_order: "asc".into(),
            polarity: Some("madurai".into()),
            mastery_rank_max: Some(16),
            re_rolls_min: Some(0),
            re_rolls_max: Some(1000),
            mod_rank: None,
        }
    }
    fn listing_with_attrs(id: &str, price: u32, attributes: Vec<DisplayAttribute>) -> Listing {
        Listing {
            id: id.into(),
            riven_name: id.into(),
            price: Some(price),
            price_kind: "一口价".into(),
            seller: String::new(),
            seller_status: String::new(),
            rerolls: None,
            mod_rank: None,
            attributes,
            polarity: None,
            mastery_rank: None,
        }
    }
    fn attr(slug: &str, value: f64, positive: bool) -> DisplayAttribute {
        DisplayAttribute {
            slug: slug.into(),
            name: slug.into(),
            value,
            positive,
            unit: "%".into(),
        }
    }
    #[test]
    fn query_encodes_and_omits_none() {
        let mut f = base();
        f.weapon_slug = "a/b".into();
        f.positive_stats = vec!["x/y".into(), "z".into()];
        f.mod_rank = Some("maxed".into());
        let q = build_search_url(&f);
        assert!(
            q.contains("a%2Fb")
                && q.contains("x%2Fy%2Cz")
                && q.contains("mastery_rank_max=16")
                && q.contains("re_rolls_min=0")
                && q.contains("re_rolls_max=1000")
                && q.contains("mod_rank=maxed")
                && q.contains("negative_stats=damage_vs_infested")
                && q.contains("polarity=madurai")
                && !q.contains("mastery_rank_min=")
        );
        f.negative_stat = None;
        f.polarity = None;
        f.mastery_rank_max = None;
        f.re_rolls_min = None;
        f.re_rolls_max = None;
        f.mod_rank = None;
        let q = build_search_url(&f);
        assert!(
            !q.contains("negative_stats=")
                && !q.contains("polarity=")
                && !q.contains("mastery_rank_max=")
                && !q.contains("re_rolls_min=")
                && !q.contains("re_rolls_max=")
                && !q.contains("mod_rank=")
        );
    }
    #[test]
    fn maps_server_sort_values_and_preserves_non_price_order() {
        let mut f = base();
        f.sort_order = "positive_attr_asc".into();
        assert!(build_search_url(&f).contains("sort_by=positive_attr_asc"));
        f.sort_order = "positive_attr_desc".into();
        assert!(build_search_url(&f).contains("sort_by=positive_attr_desc"));
        f.sort_order = "asc".into();
        assert!(build_search_url(&f).contains("sort_by=price_asc"));

        let mut listings = vec![
            Listing {
                id: "first".into(),
                riven_name: "first".into(),
                price: Some(99),
                price_kind: "一口价".into(),
                seller: String::new(),
                seller_status: String::new(),
                rerolls: None,
                mod_rank: None,
                attributes: vec![],
                polarity: None,
                mastery_rank: None,
            },
            Listing {
                id: "second".into(),
                riven_name: "second".into(),
                price: Some(1),
                price_kind: "一口价".into(),
                seller: String::new(),
                seller_status: String::new(),
                rerolls: None,
                mod_rank: None,
                attributes: vec![],
                polarity: None,
                mastery_rank: None,
            },
        ];
        sort_listings(
            &mut listings,
            "positive_attr_desc",
            &["critical_chance".into()],
        );
        assert_eq!(listings[0].id, "first");
        sort_listings(&mut listings, "asc", &[]);
        assert_eq!(listings[0].id, "second");
    }
    #[test]
    fn positive_sort_attributes_with_priority_and_missing_last() {
        let stats = vec!["critical_chance".into(), "critical_damage".into()];
        let mut listings = vec![
            listing_with_attrs(
                "tie-second",
                1,
                vec![
                    attr("critical_chance", 10.0, true),
                    attr("critical_damage", 30.0, true),
                ],
            ),
            listing_with_attrs(
                "tie-first",
                99,
                vec![
                    attr("critical_chance", 10.0, true),
                    attr("critical_damage", 20.0, true),
                ],
            ),
            listing_with_attrs("missing", 2, vec![attr("critical_chance", f64::NAN, true)]),
            listing_with_attrs(
                "negative-only",
                3,
                vec![attr("critical_chance", 5.0, false)],
            ),
        ];
        sort_listings(&mut listings, "positive_attr_asc", &stats);
        assert_eq!(
            listings
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["tie-first", "tie-second", "missing", "negative-only"]
        );
        sort_listings(&mut listings, "positive_attr_desc", &stats);
        assert_eq!(listings[0].id, "tie-second");
        assert_eq!(listings[1].id, "tie-first");

        let mut inf = vec![
            listing_with_attrs("finite", 1, vec![attr("critical_chance", 5.0, true)]),
            listing_with_attrs(
                "infinite",
                2,
                vec![attr("critical_chance", f64::INFINITY, true)],
            ),
        ];
        sort_listings(&mut inf, "positive_attr_desc", &["critical_chance".into()]);
        assert_eq!(inf[0].id, "finite");
        let (weapons, attrs) = fixtures();
        let mut filters = base();
        filters.sort_order = "positive_attr_asc".into();
        filters.positive_stats.clear();
        assert!(filters.validate(&weapons, &attrs).is_err());
    }
    #[test]
    fn validates_overlap_duplicates_and_ranges() {
        let (w, a) = fixtures();
        let mut f = base();
        f.positive_stats.push("critical_chance".into());
        assert!(f.validate(&w, &a).is_err());
        f = base();
        f.negative_stat = Some("critical_chance".into());
        assert!(f.validate(&w, &a).is_err());
        f = base();
        f.weapon_slug = "unknown".into();
        assert!(f.validate(&w, &a).is_err());
        f = base();
        f.positive_stats = vec!["unknown".into()];
        assert!(f.validate(&w, &a).is_err());
        f = base();
        f.mastery_rank_max = Some(17);
        assert!(f.validate(&w, &a).is_err());
        f = base();
        f.mod_rank = Some("rank8".into());
        assert!(f.validate(&w, &a).is_err());
        f = base();
        f.re_rolls_min = Some(2);
        f.re_rolls_max = Some(1);
        assert!(f.validate(&w, &a).is_err());
        f = base();
        f.sort_order = "sideways".into();
        assert!(f.validate(&w, &a).is_err());
    }
    #[test]
    fn allows_re_rolls_1000_and_stable_price_none_last() {
        let (w, a) = fixtures();
        let f = base();
        assert!(f.validate(&w, &a).is_ok());
        let mut one = Listing {
            id: "a".into(),
            riven_name: "a".into(),
            price: None,
            price_kind: "暂无报价".into(),
            seller: String::new(),
            seller_status: String::new(),
            rerolls: None,
            mod_rank: None,
            attributes: vec![],
            polarity: None,
            mastery_rank: None,
        };
        let mut two = one.clone();
        two.id = "b".into();
        two.price = Some(2);
        assert_eq!(price_cmp(&two, &one, "asc"), std::cmp::Ordering::Less);
        assert_eq!(price_cmp(&one, &two, "desc"), std::cmp::Ordering::Greater);
        one.price = Some(2);
        assert_eq!(price_cmp(&one, &two, "asc"), std::cmp::Ordering::Less);
    }
    #[test]
    fn serde_riven_type_is_frontend_snake_case() {
        let weapon: RivenWeapon =
            serde_json::from_str(r#"{"id":"1","slug":"torid","i18n":{},"rivenType":"rifle"}"#)
                .unwrap();
        let value = serde_json::to_value(weapon).unwrap();
        assert_eq!(value["riven_type"], "rifle");
        assert!(value.get("rivenType").is_none());
    }
    #[test]
    fn validates_attribute_polarity_exclusive_and_maximum() {
        let (w, mut a) = fixtures();
        a[0].negative_only = true;
        let mut f = base();
        assert!(f.validate(&w, &a).is_err());
        a[0].negative_only = false;
        a[0].positive_only = true;
        f.positive_stats.clear();
        f.negative_stat = Some("critical_chance".into());
        assert!(f.validate(&w, &a).is_err());
        a[0].positive_only = false;
        a[0].exclusive_to = vec!["pistol".into()];
        f = base();
        f.negative_stat = None;
        assert!(f.validate(&w, &a).is_err());
        a[0].exclusive_to = vec![];
        f.positive_stats = vec!["critical_chance".into(); 4];
        assert!(f.validate(&w, &a).is_err());
        f.positive_stats = vec![
            "critical_chance".into(),
            "critical_damage".into(),
            "damage_vs_infested".into(),
        ];
        f.negative_stat = None;
        assert!(f.validate(&w, &a).is_ok());
        f.positive_stats.clear();
        f.polarity = Some("bad".into());
        assert!(f.validate(&w, &a).is_err());
        f.polarity = Some("madurai".into());
        f.mod_rank = Some("maxed".into());
        assert!(f.validate(&w, &a).is_ok());
    }
    #[test]
    fn normalize_rejects_malformed_and_preserves_dto() {
        let attrs = fixtures().1;
        let raw = serde_json::json!({"payload":{"auctions":[
            {"id":"dup","visible":true,"closed":false,"starting_price":serde_json::Value::Null,"buyout_price":serde_json::Value::Null,"owner":{"ingame_name":"x","status":"offline"},"item":{"name":"r","re_rolls":4,"mod_rank":8,"mastery_level":12,"polarity":"madurai","type":"riven","weapon_url_name":"torid","attributes":[{"value":0.57,"positive":false,"url_name":"damage_vs_infested"}]}},
            {"id":"dup","visible":true,"closed":false,"starting_price":2,"buyout_price":serde_json::Value::Null,"owner":{"ingame_name":"x","status":"offline"},"item":{"name":"r","type":"riven","weapon_url_name":"torid","attributes":[]}},
            {"id":"hidden","visible":false,"closed":false,"starting_price":serde_json::Value::Null,"buyout_price":serde_json::Value::Null,"owner":{"ingame_name":"x","status":"offline"},"item":{"name":"hidden","type":"riven","weapon_url_name":"torid","attributes":[]}},
            {"id":"closed","visible":true,"closed":true,"starting_price":1,"buyout_price":serde_json::Value::Null,"owner":{"ingame_name":"x","status":"offline"},"item":{"name":"closed","type":"riven","weapon_url_name":"torid","attributes":[]}},
            {"id":"wrong","visible":true,"closed":false,"starting_price":serde_json::Value::Null,"buyout_price":serde_json::Value::Null,"owner":{"ingame_name":"x","status":"offline"},"item":{"name":"wrong","type":"riven","weapon_url_name":"other","attributes":[]}}
        ]}});
        let rows =
            normalize_response(serde_json::from_value(raw).unwrap(), "torid", &attrs).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].price, None);
        assert_eq!(rows[0].polarity.as_deref(), Some("madurai"));
        assert_eq!(rows[0].mastery_rank, Some(12));
        assert_eq!(rows[0].attributes[0].value, 0.57);
        assert_eq!(rows[0].attributes[0].unit, "倍");
        assert!(normalize_response(
            serde_json::from_str::<V1Envelope>(r#"{}"#).unwrap(),
            "torid",
            &attrs
        )
        .is_err());
    }
    #[test]
    fn blocked_gate_stops_before_network() {
        tauri::async_runtime::block_on(async {
            let service = MarketService::new();
            *service.blocked_until.lock().await = Some(Instant::now() + Duration::from_secs(5));
            let result: Result<V1Envelope, _> = service.get_json("http://127.0.0.1:1", &[]).await;
            assert!(matches!(result, Err(MarketError::Cooldown(_))));
        });
    }
    #[test]
    #[ignore]
    fn live_startup_latency_probe() {
        tauri::async_runtime::block_on(async {
            let full_started = Instant::now();
            let path = std::env::temp_dir().join(format!(
                "wm-market-probe-{}-{}.sqlite3",
                std::process::id(),
                catalog_cache::now_ms()
            ));
            let first_started = Instant::now();
            let service = MarketService::with_catalog_path(path.clone());

            let gate_before = *service.request_gate.lock().await;
            let first = service.catalog().await.expect("first local catalog");
            let first_ms = first_started.elapsed().as_millis();
            let gate_after = *service.request_gate.lock().await;
            assert_eq!(gate_before, gate_after);
            assert!(!first.weapons.is_empty());
            assert!(!first.attributes.is_empty());

            drop(service);
            let reopen_started = Instant::now();
            let reopened = MarketService::with_catalog_path(path.clone());
            let reopen_gate_before = *reopened.request_gate.lock().await;
            let second = reopened.catalog().await.expect("reopened local catalog");
            let reopen_ms = reopen_started.elapsed().as_millis();
            let reopen_gate_after = *reopened.request_gate.lock().await;
            assert_eq!(reopen_gate_before, reopen_gate_after);
            assert_eq!(first.weapons.len(), second.weapons.len());
            assert_eq!(first.attributes.len(), second.attributes.len());
            let status = reopened.catalog_status().await.expect("catalog status");

            let search_started = Instant::now();
            let snapshot = reopened
                .search(MarketFilters {
                    weapon_slug: "torid".into(),
                    positive_stats: vec![],
                    negative_stat: None,
                    sort_order: "asc".into(),
                    polarity: None,
                    mastery_rank_max: None,
                    re_rolls_min: None,
                    re_rolls_max: None,
                    mod_rank: None,
                })
                .await
                .expect("initial torid search");
            assert!(!snapshot.listings.is_empty());
            let search_ms = search_started.elapsed().as_millis();

            println!(
                "startup probe: first_install_local_ms={} reopen_local_ms={} search_ms={} startup_to_results_ms={} probe_total_ms={} search_elapsed_ms={} storage={} weapons={} attributes={} listings={}",
                first_ms,
                reopen_ms,
                search_ms,
                reopen_started.elapsed().as_millis(),
                full_started.elapsed().as_millis(),
                snapshot.elapsed_ms,
                status.storage,
                first.weapons.len(),
                first.attributes.len(),
                snapshot.listings.len()
            );
            let _ = std::fs::remove_file(path);
        });
    }
    #[test]
    #[ignore]
    fn live_filtered_torid_smoke() {
        tauri::async_runtime::block_on(async {
            let service = MarketService::new();
            let filters = MarketFilters {
                weapon_slug: "torid".into(),
                positive_stats: vec!["critical_chance".into(), "critical_damage".into()],
                negative_stat: Some("damage_vs_infested".into()),
                sort_order: "desc".into(),
                polarity: Some("madurai".into()),
                mastery_rank_max: Some(16),
                re_rolls_min: Some(1),
                re_rolls_max: Some(200),
                mod_rank: Some("maxed".into()),
            };
            let snapshot = service
                .search(filters)
                .await
                .expect("filtered torid search");
            assert!(!snapshot.listings.is_empty());
            for row in &snapshot.listings {
                assert_eq!(row.polarity.as_deref(), Some("madurai"));
                assert!(row.mastery_rank.unwrap_or(99) <= 16);
                assert_eq!(row.mod_rank, Some(8));
                assert!(row.rerolls.unwrap_or(0) >= 1 && row.rerolls.unwrap_or(201) <= 200);
                assert!(row
                    .attributes
                    .iter()
                    .any(|a| a.slug == "critical_chance" && a.positive));
                assert!(row
                    .attributes
                    .iter()
                    .any(|a| a.slug == "critical_damage" && a.positive));
                assert!(row
                    .attributes
                    .iter()
                    .any(|a| a.slug == "damage_vs_infested" && !a.positive));
            }
            assert!(snapshot
                .listings
                .windows(2)
                .all(|w| price_cmp(&w[0], &w[1], "desc") != std::cmp::Ordering::Greater));
            println!(
                "filtered torid directory={} batch={} elapsed_ms={}",
                snapshot.weapon.slug, snapshot.batch_count, snapshot.elapsed_ms
            );
        });
    }
}
