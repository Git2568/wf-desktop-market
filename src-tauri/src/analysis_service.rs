use crate::{
    analysis_cache::{AnalysisCache, CachedAnalysisData},
    market::{Listing, MarketCatalog, MarketFilters, MarketService},
    riven_analysis::{self, AnalysisRequest, ReferenceData, StatAnalysisResult},
    riven_valuation::{self, ValuationResult},
};
use reqwest::header::LAST_MODIFIED;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;

const WEEKLY_URL: &str = "https://www-static.warframe.com/repos/weeklyRivensPC.json";

#[derive(Debug, Clone, Serialize)]
pub struct WeeklyGroup {
    pub rerolled: bool,
    pub popularity: f64,
    pub avg: f64,
    pub median: Option<f64>,
    pub min: f64,
    pub max: f64,
    pub stddev: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct WeeklyWeaponSummary {
    pub source: String,
    pub source_updated_at: Option<String>,
    pub fetched_at: i64,
    pub groups: Vec<WeeklyGroup>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Frequency {
    pub slug: String,
    pub name: String,
    pub positive: bool,
    pub count: u32,
    pub selected: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonListing {
    pub id: String,
    pub price: u32,
    pub positive_matches: u32,
    pub positive_total: u32,
    pub negative_match: bool,
    pub exact_match: bool,
    pub attributes: Vec<ComparisonAttribute>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonAttribute {
    pub slug: String,
    pub name: String,
    pub positive: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonResult {
    pub scope: String,
    pub requested_count: u32,
    pub fetched_count: u32,
    pub sample_count: u32,
    pub quoted_min: Option<u32>,
    pub quoted_max: Option<u32>,
    pub frequencies: Vec<Frequency>,
    pub listings: Vec<ComparisonListing>,
}
#[derive(Debug, Clone, Serialize)]
pub struct MarketAnalysisResult {
    pub weapon_slug: String,
    pub sampled_at: i64,
    pub from_cache: bool,
    pub stale: bool,
    pub weekly: Option<WeeklyWeaponSummary>,
    pub weekly_error: Option<String>,
    pub comparison: Option<ComparisonResult>,
    pub comparison_error: Option<String>,
    pub valuation: Option<ValuationResult>,
    pub valuation_error: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisDataStatus {
    pub reference_version: String,
    pub rule_version: String,
    pub weekly_source_updated_at: Option<String>,
    pub weekly_fetched_at: Option<i64>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeeklyRow {
    #[serde(rename = "itemType")]
    item_type: String,
    compatibility: Option<String>,
    rerolled: bool,
    avg: f64,
    stddev: f64,
    min: f64,
    max: f64,
    pop: f64,
    median: Option<f64>,
}
#[derive(Debug, Clone)]
struct WeeklyCache {
    source_updated_at: Option<String>,
    fetched_at: i64,
    rows: Vec<WeeklyRow>,
    recovered: bool,
}
#[derive(Debug, Clone)]
struct SampleCache {
    fetched_at: i64,
    fetched_count: u32,
    listings: Vec<Listing>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SamplePayload {
    fetched_count: u32,
    listings: Vec<Listing>,
}
struct WeeklyOutcome {
    summary: Option<WeeklyWeaponSummary>,
    error: Option<String>,
    stale: bool,
}
struct SampleOutcome {
    listings: Vec<Listing>,
    fetched_at: i64,
    fetched_count: u32,
    from_cache: bool,
    stale: bool,
    error: Option<String>,
}

#[derive(Clone)]
pub struct AnalysisService {
    market: MarketService,
    cache: Arc<AnalysisCache>,
    client: reqwest::Client,
    update_lock: Arc<Mutex<()>>,
    weekly_lock: Arc<Mutex<()>>,
    sample_lock: Arc<Mutex<()>>,
    samples: Arc<Mutex<HashMap<String, SampleCache>>>,
}

impl AnalysisService {
    pub fn new(path: PathBuf, market: MarketService) -> Self {
        Self {
            market,
            cache: Arc::new(AnalysisCache::new(path)),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .expect("analysis HTTP client"),
            update_lock: Arc::new(Mutex::new(())),
            weekly_lock: Arc::new(Mutex::new(())),
            sample_lock: Arc::new(Mutex::new(())),
            samples: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn stats(&self, request: AnalysisRequest) -> Result<StatAnalysisResult, String> {
        let catalog = analysis_catalog(self.market.catalog().await.map_err(|e| e.to_string())?);
        let reference = self.reference().await?;
        riven_analysis::analyze(&request, &catalog, &reference)
    }

    pub async fn range_hints(
        &self,
        request: riven_analysis::RangeHintRequest,
    ) -> Result<riven_analysis::RangeHintsResult, String> {
        let catalog = analysis_catalog(self.market.catalog().await.map_err(|e| e.to_string())?);
        let reference = self.reference().await?;
        riven_analysis::range_hints(&request, &catalog, &reference)
    }

    pub async fn market(&self, request: AnalysisRequest) -> Result<MarketAnalysisResult, String> {
        if !(1..=50).contains(&request.sample_size) {
            return Err("样本数量必须在 1 到 50 之间".into());
        }
        let catalog = analysis_catalog(self.market.catalog().await.map_err(|e| e.to_string())?);
        riven_analysis::validate_request(&request, &catalog)?;
        let weapon_name = catalog
            .weapons
            .iter()
            .find(|w| w.slug == request.weapon_slug)
            .and_then(|w| w.i18n.get("en"))
            .map(|x| x.name.clone())
            .unwrap_or_default();
        let weekly_service = self.clone();
        let weekly_name = weapon_name.clone();
        let weekly_task =
            tokio::spawn(async move { weekly_service.weekly(false, Some(&weekly_name)).await });
        let sample_service = self.clone();
        let sample_weapon = request.weapon_slug.clone();
        let sample_task = tokio::spawn(async move { sample_service.sample(&sample_weapon).await });
        let weekly = weekly_task.await.map_err(|e| e.to_string())?;
        let sample = sample_task.await.map_err(|e| e.to_string())?;
        let (weekly_value, weekly_error, weekly_stale) = match weekly {
            Ok(v) => (v.summary, v.error, v.stale),
            Err(e) => (None, Some(e), true),
        };
        let (
            comparison,
            comparison_error,
            sampled_at,
            from_cache,
            sample_stale,
            valuation,
            valuation_error,
        ) = match sample {
            Ok(v) => {
                let reference_result = self.reference().await;
                let (valuation, valuation_error) = match reference_result {
                    Ok(reference) => (
                        Some(riven_valuation::estimate(
                            &request,
                            &catalog,
                            &reference,
                            &v.listings,
                            if weekly_stale {
                                None
                            } else {
                                weekly_value.as_ref()
                            },
                            v.stale,
                        )),
                        None,
                    ),
                    Err(e) => (None, Some(format!("估价参考数据不可用：{e}"))),
                };
                (
                    Some(compare(
                        &request,
                        &catalog,
                        v.listings.clone(),
                        v.fetched_count,
                    )),
                    v.error.clone(),
                    v.fetched_at,
                    v.from_cache,
                    v.stale,
                    valuation,
                    valuation_error,
                )
            }
            Err(e) => (
                None,
                Some(e.clone()),
                now_ms(),
                false,
                true,
                Some(riven_valuation::ValuationResult {
                    rule_version: riven_valuation::RULE_VERSION.into(),
                    status: "unavailable".into(),
                    method: "none".into(),
                    estimate: None,
                    low: None,
                    high: None,
                    confidence: "low".into(),
                    pool_count: 0,
                    comparable_count: 0,
                    hot_stats: vec![],
                    popularity_factor: 1.,
                    numeric_factor: 1.,
                    heat_factor: 1.,
                    numeric_coverage: 0.,
                    layout_focus: "balanced".into(),
                    references: vec![],
                    warnings: vec!["挂单样本获取失败".into()],
                    reason: Some(e.clone()),
                }),
                Some(e),
            ),
        };
        Ok(MarketAnalysisResult {
            weapon_slug: request.weapon_slug,
            sampled_at,
            from_cache,
            stale: weekly_stale || sample_stale,
            weekly: weekly_value,
            weekly_error,
            comparison,
            comparison_error,
            valuation,
            valuation_error,
        })
    }

    pub async fn status(&self) -> Result<AnalysisDataStatus, String> {
        let reference = self.reference().await?;
        let row = self.read_weekly().await.ok().flatten();
        Ok(AnalysisDataStatus {
            reference_version: reference.reference_version,
            rule_version: reference.rule_version,
            weekly_source_updated_at: row.as_ref().and_then(|x| x.source_updated_at.clone()),
            weekly_fetched_at: row.as_ref().map(|x| x.fetched_at),
            message: row
                .as_ref()
                .map(|x| {
                    if x.recovered {
                        "周统计由上一份有效缓存恢复".into()
                    } else {
                        String::new()
                    }
                })
                .filter(|x| !x.is_empty())
                .or_else(|| row.is_none().then_some("官方周统计尚无有效本地缓存".into())),
        })
    }

    pub async fn refresh(&self, force: bool) -> Result<AnalysisDataStatus, String> {
        let _guard = self.update_lock.lock().await;
        let reference = self.reference().await?;
        let weekly = self.weekly(force, None).await?;
        self.status().await.map(|mut s| {
            s.reference_version = reference.reference_version;
            s.rule_version = reference.rule_version;
            s.message = weekly
                .error
                .or_else(|| Some("已校验随包参考数据并检查官方周统计".into()));
            s
        })
    }

    async fn reference(&self) -> Result<ReferenceData, String> {
        let embedded = riven_analysis::embedded_reference()?;
        let cache = self.cache.clone();
        let loaded = tokio::task::spawn_blocking(move || cache.load("reference", "current"))
            .await
            .map_err(|e| e.to_string())
            .ok()
            .and_then(|x| x.ok())
            .flatten();
        if let Some(row) = loaded {
            if let Ok(reference) = serde_json::from_slice::<ReferenceData>(&row.payload) {
                if reference.schema == embedded.schema
                    && reference.rule_version == embedded.rule_version
                    && reference.reference_version == embedded.reference_version
                    && riven_analysis::validate_reference(&reference).is_ok()
                {
                    return Ok(reference);
                }
            }
        }
        let cache = self.cache.clone();
        let payload = serde_json::to_vec(&embedded).map_err(|e| e.to_string())?;
        let retrieved_at = embedded.retrieved_at.clone();
        let _ = tokio::task::spawn_blocking(move || {
            cache.save(
                "reference",
                "current",
                &CachedAnalysisData {
                    payload,
                    fetched_at: now_ms(),
                    source_updated_at: Some(retrieved_at),
                    source: "embedded reference".into(),
                    recovered: false,
                },
            )
        })
        .await;
        Ok(embedded)
    }

    async fn read_weekly(&self) -> Result<Option<WeeklyCache>, String> {
        let cache = self.cache.clone();
        let row = tokio::task::spawn_blocking(move || cache.load("weekly", "pc"))
            .await
            .map_err(|e| e.to_string())??;
        row.map(|row| {
            let rows: Vec<WeeklyRow> =
                serde_json::from_slice(&row.payload).map_err(|e| e.to_string())?;
            validate_weekly_rows(&rows)?;
            Ok(WeeklyCache {
                source_updated_at: row.source_updated_at,
                fetched_at: row.fetched_at,
                rows,
                recovered: row.recovered,
            })
        })
        .transpose()
    }

    async fn weekly(
        &self,
        force: bool,
        weapon_name: Option<&str>,
    ) -> Result<WeeklyOutcome, String> {
        let _guard = self.weekly_lock.lock().await;
        let cached = self.read_weekly().await.ok().flatten();
        if !force {
            if let Some(cached) = cached.clone() {
                if now_ms().saturating_sub(cached.fetched_at) < 86_400_000 {
                    return Ok(WeeklyOutcome {
                        summary: Some(summary_for(&cached, "Digital Extremes", weapon_name)),
                        error: cached
                            .recovered
                            .then_some("周统计由上一份有效缓存恢复".into()),
                        stale: cached.recovered,
                    });
                }
            }
        }
        let response = match self.client.get(WEEKLY_URL).send().await {
            Ok(response) => response,
            Err(e) => {
                return Ok(weekly_fallback(
                    cached,
                    weapon_name,
                    format!("官方周统计请求失败：{e}"),
                ))
            }
        };
        if !response.status().is_success() {
            return Ok(weekly_fallback(
                cached,
                weapon_name,
                format!("官方周统计 HTTP {}", response.status()),
            ));
        }
        let updated = response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|x| x.to_str().ok())
            .map(str::to_owned);
        let raw = match response.text().await {
            Ok(raw) => raw,
            Err(e) => {
                return Ok(weekly_fallback(
                    cached,
                    weapon_name,
                    format!("读取官方周统计失败：{e}"),
                ))
            }
        };
        let rows: Vec<WeeklyRow> = match parse_weekly_rows(&raw) {
            Ok(rows) => rows,
            Err(e) => {
                return Ok(weekly_fallback(
                    cached,
                    weapon_name,
                    format!("官方周统计格式无效：{e}"),
                ))
            }
        };
        if validate_weekly_rows(&rows).is_err() {
            return Ok(weekly_fallback(
                cached,
                weapon_name,
                "官方周统计字段校验失败".into(),
            ));
        }
        let fetched = now_ms();
        let cache = self.cache.clone();
        let payload = serde_json::to_vec(&rows).map_err(|e| e.to_string())?;
        let source_updated_at = updated.clone();
        let save_result = tokio::task::spawn_blocking(move || {
            cache.save(
                "weekly",
                "pc",
                &CachedAnalysisData {
                    payload,
                    fetched_at: fetched,
                    source_updated_at,
                    source: WEEKLY_URL.into(),
                    recovered: false,
                },
            )
        })
        .await
        .map_err(|e| e.to_string())?;
        let cache_warning = save_result
            .err()
            .map(|e| format!("官方周统计已获取，但缓存写入失败：{e}"));
        Ok(WeeklyOutcome {
            summary: Some(summary_for(
                &WeeklyCache {
                    source_updated_at: updated,
                    fetched_at: fetched,
                    rows,
                    recovered: false,
                },
                "Digital Extremes",
                weapon_name,
            )),
            error: cache_warning,
            stale: false,
        })
    }

    async fn sample(&self, weapon: &str) -> Result<SampleOutcome, String> {
        let _guard = self.sample_lock.lock().await;
        let key = format!("{weapon}:pc");
        let now = now_ms();
        if let Some(c) = self.samples.lock().await.get(&key).cloned() {
            if now.saturating_sub(c.fetched_at) < 300_000 {
                return Ok(SampleOutcome {
                    listings: c.listings,
                    fetched_at: c.fetched_at,
                    fetched_count: c.fetched_count,
                    from_cache: true,
                    stale: false,
                    error: None,
                });
            }
        }
        let cache = self.cache.clone();
        let cache_key = weapon.to_string();
        let cached_row = tokio::task::spawn_blocking(move || cache.load("market", &cache_key))
            .await
            .ok()
            .and_then(|x| x.ok())
            .flatten();
        let mut stale_cache = None;
        if let Some(row) = cached_row {
            if let Ok(payload) = serde_json::from_slice::<SamplePayload>(&row.payload) {
                let listings = payload.listings;
                stale_cache = Some((listings.clone(), row.fetched_at, payload.fetched_count));
                if now.saturating_sub(row.fetched_at) < 300_000 {
                    self.samples.lock().await.insert(
                        key.clone(),
                        SampleCache {
                            fetched_at: row.fetched_at,
                            fetched_count: payload.fetched_count,
                            listings: listings.clone(),
                        },
                    );
                    return Ok(SampleOutcome {
                        listings,
                        fetched_at: row.fetched_at,
                        fetched_count: payload.fetched_count,
                        from_cache: true,
                        stale: row.recovered,
                        error: row
                            .recovered
                            .then_some("市场样本由上一份有效缓存恢复".into()),
                    });
                }
            }
        }
        let filters = MarketFilters {
            weapon_slug: weapon.into(),
            positive_stats: vec![],
            negative_stat: None,
            sort_order: "desc".into(),
            polarity: None,
            mastery_rank_max: None,
            re_rolls_min: None,
            re_rolls_max: None,
            mod_rank: None,
        };
        let snapshot = match self.market.search(filters).await {
            Ok(snapshot) => snapshot,
            Err(e) => {
                if let Some(cached) = self.samples.lock().await.get(&key).cloned() {
                    return Ok(SampleOutcome {
                        listings: cached.listings,
                        fetched_at: cached.fetched_at,
                        fetched_count: cached.fetched_count,
                        from_cache: true,
                        stale: true,
                        error: Some(format!("挂单刷新失败，保留旧样本：{e}")),
                    });
                }
                if let Some((listings, fetched_at, fetched_count)) = stale_cache {
                    return Ok(SampleOutcome {
                        listings,
                        fetched_at,
                        fetched_count,
                        from_cache: true,
                        stale: true,
                        error: Some(format!("挂单刷新失败，保留旧样本：{e}")),
                    });
                }
                return Err(format!("挂单样本请求失败：{e}"));
            }
        };
        let fetched_count = snapshot.listings.len() as u32;
        let fetched_at = snapshot.fetched_at as i64;
        let mut listings: Vec<Listing> = snapshot
            .listings
            .into_iter()
            .filter(|x| x.price_kind == "一口价" && x.price.unwrap_or(0) > 0)
            .collect();
        listings.sort_by(|a, b| b.price.cmp(&a.price).then_with(|| a.id.cmp(&b.id)));
        self.samples.lock().await.insert(
            key,
            SampleCache {
                fetched_at,
                fetched_count,
                listings: listings.clone(),
            },
        );
        let cache = self.cache.clone();
        let payload = serde_json::to_vec(&SamplePayload {
            fetched_count,
            listings: listings.clone(),
        })
        .map_err(|e| e.to_string())?;
        let cache_key = weapon.to_string();
        let save_result = tokio::task::spawn_blocking(move || {
            cache.save(
                "market",
                &cache_key,
                &CachedAnalysisData {
                    payload,
                    fetched_at,
                    source_updated_at: None,
                    source: "Warframe.market current batch".into(),
                    recovered: false,
                },
            )
        })
        .await
        .map_err(|e| e.to_string())?;
        Ok(SampleOutcome {
            listings,
            fetched_at,
            fetched_count,
            from_cache: false,
            stale: false,
            error: save_result
                .err()
                .map(|e| format!("市场样本已获取，但缓存写入失败：{e}")),
        })
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
fn analysis_catalog(mut c: MarketCatalog) -> MarketCatalog {
    for w in &mut c.weapons {
        if w.group.eq_ignore_ascii_case("archgun") {
            w.riven_type = w.group.clone();
        }
    }
    c
}
fn summary_for(c: &WeeklyCache, source: &str, weapon_name: Option<&str>) -> WeeklyWeaponSummary {
    let wanted = weapon_name.map(|x| x.trim().to_ascii_lowercase());
    let groups = c
        .rows
        .iter()
        .filter(|row| {
            wanted
                .as_deref()
                .map(|x| {
                    row.compatibility
                        .as_deref()
                        .map(|v| v.trim().to_ascii_lowercase() == x)
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        })
        .map(|row| WeeklyGroup {
            rerolled: row.rerolled,
            popularity: row.pop,
            avg: row.avg,
            median: row.median,
            min: row.min,
            max: row.max,
            stddev: row.stddev,
        })
        .collect();
    WeeklyWeaponSummary {
        source: source.into(),
        source_updated_at: c.source_updated_at.clone(),
        fetched_at: c.fetched_at,
        groups,
    }
}

fn parse_weekly_rows(raw: &str) -> Result<Vec<WeeklyRow>, String> {
    match serde_json::from_str(raw) {
        Ok(rows) => return Ok(rows),
        Err(_) => {}
    }
    let mut out = String::with_capacity(raw.len() + 32);
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '"' => {
                out.push('"');
                i += 1;
                let mut closed = false;
                while i < chars.len() {
                    let ch = chars[i];
                    out.push(ch);
                    if ch == '\\' {
                        i += 1;
                        if i >= chars.len() {
                            return Err("weekly 双引号字符串转义不完整".into());
                        }
                        out.push(chars[i]);
                    } else if ch == '"' {
                        closed = true;
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                if !closed {
                    return Err("weekly 双引号字符串未闭合".into());
                }
            }
            '\'' => {
                i += 1;
                out.push('"');
                while i < chars.len() {
                    let ch = chars[i];
                    if ch == '\'' {
                        out.push('"');
                        i += 1;
                        break;
                    }
                    if ch == '\\' {
                        i += 1;
                        if i >= chars.len() {
                            return Err("weekly 单引号字符串转义不完整".into());
                        }
                        let escaped = chars[i];
                        if escaped == '\'' {
                            out.push('\'');
                        } else if escaped == '"' || escaped == '\\' || escaped == '/' {
                            out.push('\\');
                            out.push(escaped);
                        } else if escaped == 'n'
                            || escaped == 'r'
                            || escaped == 't'
                            || escaped == 'b'
                            || escaped == 'f'
                        {
                            out.push('\\');
                            out.push(escaped);
                        } else if escaped == 'u' {
                            out.push_str("\\u");
                            for _ in 0..4 {
                                i += 1;
                                if i >= chars.len() || !chars[i].is_ascii_hexdigit() {
                                    return Err("weekly unicode 转义无效".into());
                                }
                                out.push(chars[i]);
                            }
                        } else {
                            return Err("weekly 字符串包含未知转义".into());
                        }
                    } else if ch == '"' {
                        out.push_str("\\\"");
                    } else {
                        out.push(ch);
                    }
                    i += 1;
                }
                if !out.ends_with('"') {
                    return Err("weekly 单引号字符串未闭合".into());
                }
            }
            ch if (ch == 'e' || ch == 'E')
                && out.chars().last().is_some_and(|x| x.is_ascii_digit()) =>
            {
                out.push(ch);
                i += 1;
                if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                    out.push(chars[i]);
                    i += 1;
                }
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    out.push(chars[i]);
                    i += 1;
                }
                if i == start {
                    return Err("weekly 指数缺少数字".into());
                }
            }
            ch if ch.is_ascii_alphabetic() || ch == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let mut j = i;
                while j < chars.len() && chars[j].is_ascii_whitespace() {
                    j += 1;
                }
                if word == "true" || word == "false" || word == "null" {
                    out.push_str(&word);
                } else if j < chars.len() && chars[j] == ':' {
                    out.push('"');
                    out.push_str(&word);
                    out.push('"');
                } else {
                    return Err(format!("weekly 未知标识符 {word}"));
                }
            }
            ch => {
                out.push(ch);
                i += 1;
            }
        }
    }
    serde_json::from_str(&out).map_err(|e| format!("weekly 受限解析失败：{e}"))
}

fn validate_weekly_rows(rows: &[WeeklyRow]) -> Result<(), String> {
    if rows.is_empty() {
        return Err("weekly data is empty".into());
    }
    let mut seen = std::collections::HashSet::new();
    for row in rows {
        if row.item_type.trim().is_empty()
            || !seen.insert((
                row.item_type.trim().to_ascii_lowercase(),
                row.compatibility
                    .as_deref()
                    .map(|x| x.trim().to_ascii_lowercase()),
                row.rerolled,
            ))
        {
            return Err("weekly row identity is invalid or duplicated".into());
        }
        if ![row.avg, row.stddev, row.min, row.max, row.pop]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.0)
            || row.pop > 100.0
            || row.min > row.max
            || row
                .median
                .map(|v| !v.is_finite() || v < 0.0)
                .unwrap_or(false)
        {
            return Err("weekly numeric fields are invalid".into());
        }
    }
    Ok(())
}

fn weekly_fallback(
    cached: Option<WeeklyCache>,
    weapon_name: Option<&str>,
    error: String,
) -> WeeklyOutcome {
    match cached {
        Some(cached) => WeeklyOutcome {
            summary: Some(summary_for(&cached, "Digital Extremes", weapon_name)),
            error: Some(format!("{error}，已保留旧周统计")),
            stale: true,
        },
        None => WeeklyOutcome {
            summary: None,
            error: Some(error),
            stale: true,
        },
    }
}

fn compare(
    req: &AnalysisRequest,
    catalog: &MarketCatalog,
    mut listings: Vec<Listing>,
    fetched_count: u32,
) -> ComparisonResult {
    let n = req.sample_size as usize;
    listings.truncate(n);
    let selected_pos: std::collections::HashSet<_> = req
        .attributes
        .iter()
        .filter(|a| a.positive)
        .map(|a| a.slug.as_str())
        .collect();
    let selected_neg = req
        .attributes
        .iter()
        .find(|a| !a.positive)
        .map(|a| a.slug.as_str());
    let mut counts: HashMap<(String, bool), u32> = HashMap::new();
    let mut output = Vec::new();
    for l in &listings {
        let mut seen_card = std::collections::HashSet::new();
        for a in &l.attributes {
            if seen_card.insert((a.slug.clone(), a.positive)) {
                *counts.entry((a.slug.clone(), a.positive)).or_default() += 1;
            }
        }
        let positives: std::collections::HashSet<_> = l
            .attributes
            .iter()
            .filter(|a| a.positive)
            .map(|a| a.slug.as_str())
            .collect();
        let negatives: std::collections::HashSet<_> = l
            .attributes
            .iter()
            .filter(|a| !a.positive)
            .map(|a| a.slug.as_str())
            .collect();
        let selected_neg_set: std::collections::HashSet<_> = selected_neg.into_iter().collect();
        let exact = positives == selected_pos && negatives == selected_neg_set;
        let positive_matches = positives.intersection(&selected_pos).count() as u32;
        let negative_match = selected_neg
            .map(|x| negatives.contains(x))
            .unwrap_or(negatives.is_empty());
        output.push(ComparisonListing {
            id: l.id.clone(),
            price: l.price.unwrap_or(0),
            positive_matches,
            positive_total: selected_pos.len() as u32,
            negative_match,
            exact_match: exact,
            attributes: l
                .attributes
                .iter()
                .map(|a| ComparisonAttribute {
                    slug: a.slug.clone(),
                    name: a.name.clone(),
                    positive: a.positive,
                })
                .collect(),
        });
    }
    let mut frequencies = Vec::new();
    for a in &catalog.attributes {
        for positive in [true, false] {
            let count = counts
                .get(&(a.slug.clone(), positive))
                .copied()
                .unwrap_or(0);
            let selected = if positive {
                selected_pos.contains(a.slug.as_str())
            } else {
                selected_neg == Some(a.slug.as_str())
            };
            if count > 0 || selected {
                frequencies.push(Frequency {
                    slug: a.slug.clone(),
                    name: a.name.clone(),
                    positive,
                    count,
                    selected,
                });
            }
        }
    }
    let prices: Vec<_> = listings.iter().filter_map(|x| x.price).collect();
    ComparisonResult {
        scope: "当前批次明确一口价样本".into(),
        requested_count: req.sample_size,
        fetched_count,
        sample_count: output.len() as u32,
        quoted_min: prices.iter().min().copied(),
        quoted_max: prices.iter().max().copied(),
        frequencies,
        listings: output,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::{DisplayAttribute, LocalizedName, RivenAttribute, RivenWeapon};

    fn request() -> AnalysisRequest {
        AnalysisRequest {
            weapon_slug: "torid".into(),
            mod_rank: 8,
            rule_mode: "current".into(),
            disposition: None,
            attributes: vec![
                crate::riven_analysis::AnalysisAttributeInput {
                    slug: "a".into(),
                    value: 1.0,
                    positive: true,
                },
                crate::riven_analysis::AnalysisAttributeInput {
                    slug: "b".into(),
                    value: 2.0,
                    positive: true,
                },
                crate::riven_analysis::AnalysisAttributeInput {
                    slug: "n".into(),
                    value: -1.0,
                    positive: false,
                },
            ],
            sample_size: 2,
        }
    }
    fn listing(id: &str, attrs: Vec<(&str, bool)>) -> Listing {
        Listing {
            id: id.into(),
            riven_name: id.into(),
            price: Some(100),
            price_kind: "一口价".into(),
            seller: String::new(),
            seller_status: String::new(),
            rerolls: None,
            mod_rank: Some(8),
            polarity: None,
            mastery_rank: Some(16),
            attributes: attrs
                .into_iter()
                .map(|(slug, positive)| DisplayAttribute {
                    slug: slug.into(),
                    name: slug.into(),
                    value: 1.0,
                    positive,
                    unit: "%".into(),
                })
                .collect(),
        }
    }
    #[test]
    fn compare_counts_selected_and_negative_zero_to_n() {
        let catalog = MarketCatalog {
            weapons: vec![RivenWeapon {
                id: "w".into(),
                slug: "torid".into(),
                i18n: HashMap::<String, LocalizedName>::new(),
                riven_type: "rifle".into(),
                group: String::new(),
                disposition: None,
                game_ref: String::new(),
            }],
            attributes: vec!["a", "b", "n"]
                .into_iter()
                .map(|slug| RivenAttribute {
                    id: slug.into(),
                    slug: slug.into(),
                    name: slug.into(),
                    english_name: slug.into(),
                    unit: "%".into(),
                    positive_only: false,
                    negative_only: false,
                    exclusive_to: vec![],
                    game_ref: String::new(),
                    positive_is_negative: false,
                })
                .collect(),
        };
        let result = compare(
            &request(),
            &catalog,
            vec![
                listing("1", vec![("a", true), ("b", true), ("n", false)]),
                listing("2", vec![("a", true), ("b", true)]),
            ],
            5,
        );
        assert_eq!(result.fetched_count, 5);
        assert_eq!(result.sample_count, 2);
        assert!(result.listings.iter().all(|x| x.positive_total == 2));
        assert_eq!(
            result
                .frequencies
                .iter()
                .find(|x| x.slug == "n")
                .map(|x| x.count),
            Some(1)
        );
        assert_eq!(
            result
                .frequencies
                .iter()
                .find(|x| x.slug == "b")
                .map(|x| x.count),
            Some(2)
        );
        assert!(result.listings[0].exact_match);
    }
    #[test]
    fn weekly_validation_rejects_duplicates_and_bad_ranges() {
        let row = WeeklyRow {
            item_type: "riven".into(),
            compatibility: Some("Torid".into()),
            rerolled: false,
            avg: 1.0,
            stddev: 1.0,
            min: 3.0,
            max: 2.0,
            pop: 20.0,
            median: Some(1.0),
        };
        assert!(validate_weekly_rows(&[row]).is_err());
    }
    #[test]
    fn parser_accepts_official_object_notation_and_null_compatibility() {
        let rows = parse_weekly_rows("[{itemType: 'Archgun Riven Mod', compatibility: null, rerolled: false, avg: 1, stddev: 0, min: 0, max: 2, pop: 10, median: 1}, {itemType: 'Rifle Riven Mod', compatibility: 'Torid\\'s Test', rerolled: true, avg: 2, stddev: 1, min: 1, max: 3, pop: 20, median: 2}]").unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows[0].compatibility.is_none());
        assert_eq!(rows[1].compatibility.as_deref(), Some("Torid's Test"));
    }
    #[test]
    fn parser_rejects_unknown_identifier() {
        assert!(parse_weekly_rows("[{itemType: nope, compatibility: null}]").is_err());
    }
    #[test]
    fn parser_accepts_mixed_quotes_and_exponents_but_rejects_code() {
        let rows = parse_weekly_rows(r#"[{"itemType": 'Rifle', compatibility: "Torid", rerolled: false, avg: 1e2, stddev: 0, min: 1, max: 2, pop: 3, median: null}]"#).unwrap();
        assert_eq!(rows[0].avg, 100.0);
        assert!(
            parse_weekly_rows("[{itemType: 'Rifle', compatibility: null, rerolled: foo()}]")
                .is_err()
        );
    }
    #[test]
    fn official_fixture_smoke_when_requested() {
        let Ok(path) = std::env::var("WM_WEEKLY_FIXTURE_PATH") else {
            return;
        };
        let raw = std::fs::read_to_string(path).expect("weekly fixture");
        let rows = parse_weekly_rows(&raw).expect("official weekly parser");
        assert_eq!(rows.len(), 811);
        validate_weekly_rows(&rows).expect("official weekly validation");
        let torid = rows
            .iter()
            .filter(|r| {
                r.compatibility
                    .as_deref()
                    .map(|x| x.eq_ignore_ascii_case("Torid"))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        assert_eq!(torid.len(), 2);
        assert!(torid.iter().any(|r| !r.rerolled) && torid.iter().any(|r| r.rerolled));
    }
}
