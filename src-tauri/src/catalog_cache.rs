use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub const SEED_JSON: &str = include_str!("../data/catalog-seed.json");
const SOURCE_BASE: &str = "https://api.warframe.market";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogSeed {
    pub schema_version: u32,
    pub source: String,
    pub language: String,
    pub platform: String,
    pub downloaded_at: i64,
    pub collection_version: String,
    pub api_version: String,
    #[serde(default)]
    pub game_version: Option<String>,
    pub source_updated_at: Option<String>,
    pub weapons: Vec<serde_json::Value>,
    pub attributes: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogDataStatus {
    pub collection_version: Option<String>,
    pub game_version: Option<String>,
    pub api_version: Option<String>,
    pub source: String,
    pub language: String,
    pub platform: String,
    pub downloaded_at: i64,
    pub last_checked_at: Option<i64>,
    pub weapons_count: usize,
    pub attributes_count: usize,
    pub storage: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StoredCatalog {
    pub seed: CatalogSeed,
    pub raw: Vec<u8>,
    pub status: CatalogDataStatus,
    pub recovered_from_previous: bool,
}

#[derive(Debug, Clone)]
pub struct CatalogCache {
    path: Option<PathBuf>,
}

impl CatalogCache {
    pub fn memory() -> Self {
        Self { path: None }
    }

    pub fn sqlite(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    pub fn seed() -> Result<StoredCatalog, String> {
        let raw = SEED_JSON.as_bytes().to_vec();
        let seed: CatalogSeed =
            serde_json::from_slice(&raw).map_err(|e| format!("内置目录 seed 无效：{e}"))?;
        validate_seed(&seed)?;
        Ok(StoredCatalog {
            status: status_for(&seed, "memory", None),
            seed,
            raw,
            recovered_from_previous: false,
        })
    }

    pub fn load(&self) -> Result<Option<StoredCatalog>, String> {
        let Some(path) = &self.path else {
            return Ok(None);
        };
        if !path.exists() {
            return Ok(None);
        }
        let conn = Connection::open(path).map_err(|e| format!("打开目录数据库失败：{e}"))?;
        let mut stmt = conn.prepare("SELECT id, payload, sha256, collection_version, api_version, game_version, source, language, platform, downloaded_at, last_checked_at FROM catalog_snapshot WHERE id IN (1,2) ORDER BY id").map_err(|e| format!("读取目录数据库失败：{e}"))?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, Vec<u8>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, String>(8)?,
                    r.get::<_, i64>(9)?,
                    r.get::<_, Option<i64>>(10)?,
                ))
            })
            .map_err(|e| format!("读取目录数据库失败：{e}"))?;
        let mut first_error = None;
        for row in rows {
            let (
                id,
                raw,
                digest,
                collection_version,
                api_version,
                game_version,
                source,
                language,
                platform,
                downloaded_at,
                last_checked_at,
            ) = match row {
                Ok(value) => value,
                Err(error) => {
                    first_error = Some(format!("读取目录数据库行失败：{error}"));
                    continue;
                }
            };
            let candidate = (|| -> Result<StoredCatalog, String> {
                if sha256(&raw) != digest {
                    return Err("目录数据库摘要校验失败".into());
                }
                let seed: CatalogSeed = serde_json::from_slice(&raw)
                    .map_err(|e| format!("目录数据库 JSON 无效：{e}"))?;
                validate_seed(&seed)?;
                if seed.collection_version != collection_version
                    || seed.api_version != api_version
                    || seed.game_version != game_version
                    || seed.source != source
                    || seed.language != language
                    || seed.platform != platform
                    || seed.downloaded_at != downloaded_at
                {
                    return Err("目录数据库元信息与快照不一致".into());
                }
                Ok(StoredCatalog {
                    status: CatalogDataStatus {
                        collection_version: Some(collection_version),
                        game_version,
                        api_version: Some(api_version),
                        source,
                        language,
                        platform,
                        downloaded_at,
                        last_checked_at,
                        weapons_count: seed.weapons.len(),
                        attributes_count: seed.attributes.len(),
                        storage: "sqlite".into(),
                        last_error: None,
                    },
                    seed,
                    raw,
                    recovered_from_previous: id == 2,
                })
            })();
            match candidate {
                Ok(v) => return Ok(Some(v)),
                Err(e) => first_error = Some(e),
            }
        }
        match first_error {
            Some(e) => Err(e),
            None => Ok(None),
        }
    }

    pub fn save(&self, stored: &StoredCatalog) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        validate_seed(&stored.seed)?;
        let parsed_raw: CatalogSeed = serde_json::from_slice(&stored.raw)
            .map_err(|e| format!("待保存目录 JSON 无效：{e}"))?;
        validate_seed(&parsed_raw)?;
        if parsed_raw != stored.seed {
            return Err("待保存目录原始数据与 typed seed 不一致".into());
        }
        if stored.status.collection_version.as_deref()
            != Some(stored.seed.collection_version.as_str())
            || stored.status.api_version.as_deref() != Some(stored.seed.api_version.as_str())
            || stored.status.game_version != stored.seed.game_version
            || stored.status.source != stored.seed.source
            || stored.status.language != stored.seed.language
            || stored.status.platform != stored.seed.platform
            || stored.status.downloaded_at != stored.seed.downloaded_at
        {
            return Err("待保存目录元信息与 seed 不一致".into());
        }
        let previous = self.load().ok().flatten();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录缓存目录失败：{e}"))?;
        }
        let conn = Connection::open(path).map_err(|e| format!("打开目录数据库失败：{e}"))?;
        conn.execute_batch("CREATE TABLE IF NOT EXISTS catalog_snapshot (id INTEGER PRIMARY KEY, payload BLOB NOT NULL, sha256 TEXT NOT NULL, collection_version TEXT NOT NULL, api_version TEXT NOT NULL, game_version TEXT, source TEXT NOT NULL, language TEXT NOT NULL, platform TEXT NOT NULL, downloaded_at INTEGER NOT NULL, last_checked_at INTEGER)")
            .map_err(|e| format!("创建目录数据库失败：{e}"))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| format!("开启目录事务失败：{e}"))?;
        if let Some(old) = previous {
            tx.execute("INSERT OR REPLACE INTO catalog_snapshot (id,payload,sha256,collection_version,api_version,game_version,source,language,platform,downloaded_at,last_checked_at) VALUES (2,?,?,?,?,?,?,?,?,?,?)", params![old.raw, sha256(&old.raw), old.seed.collection_version, old.seed.api_version, old.seed.game_version, old.seed.source, old.seed.language, old.seed.platform, old.seed.downloaded_at, old.status.last_checked_at]).map_err(|e| format!("备份目录快照失败：{e}"))?;
        }
        tx.execute("INSERT OR REPLACE INTO catalog_snapshot (id,payload,sha256,collection_version,api_version,game_version,source,language,platform,downloaded_at,last_checked_at) VALUES (1,?,?,?,?,?,?,?,?,?,?)", params![stored.raw, sha256(&stored.raw), stored.seed.collection_version, stored.seed.api_version, stored.seed.game_version, stored.seed.source, stored.seed.language, stored.seed.platform, stored.seed.downloaded_at, stored.status.last_checked_at]).map_err(|e| format!("写入目录数据库失败：{e}"))?;
        tx.commit().map_err(|e| format!("提交目录事务失败：{e}"))?;
        Ok(())
    }

    pub fn touch_checked(&self, status: &CatalogDataStatus) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let conn = Connection::open(path).map_err(|e| format!("打开目录数据库失败：{e}"))?;
        let changed = conn
            .execute(
                "UPDATE catalog_snapshot SET last_checked_at=? WHERE collection_version=?",
                params![status.last_checked_at, status.collection_version],
            )
            .map_err(|e| format!("更新目录检查时间失败：{e}"))?;
        if changed == 0 {
            return Err("未找到匹配的目录版本，未更新检查时间".into());
        }
        Ok(())
    }
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub fn sha256(raw: &[u8]) -> String {
    format!("{:x}", Sha256::digest(raw))
}

pub fn status_for(
    seed: &CatalogSeed,
    storage: &str,
    last_error: Option<String>,
) -> CatalogDataStatus {
    CatalogDataStatus {
        collection_version: Some(seed.collection_version.clone()),
        game_version: seed.game_version.clone(),
        api_version: Some(seed.api_version.clone()),
        source: seed.source.clone(),
        language: seed.language.clone(),
        platform: seed.platform.clone(),
        downloaded_at: seed.downloaded_at,
        last_checked_at: None,
        weapons_count: seed.weapons.len(),
        attributes_count: seed.attributes.len(),
        storage: storage.into(),
        last_error,
    }
}

pub fn validate_seed(seed: &CatalogSeed) -> Result<(), String> {
    if seed.schema_version != 1 {
        return Err(format!(
            "目录 schema_version 不支持：{}",
            seed.schema_version
        ));
    }
    if seed.source != SOURCE_BASE || seed.language != "zh-hans" || seed.platform != "pc" {
        return Err("目录来源、语言或平台不符合合同".into());
    }
    if seed.collection_version.trim().is_empty()
        || seed.api_version.trim().is_empty()
        || seed.downloaded_at <= 0
    {
        return Err("目录缺少版本元信息".into());
    }
    if seed.weapons.is_empty() || seed.attributes.is_empty() {
        return Err("目录 weapons/attributes 不能为空".into());
    }
    let mut ids = std::collections::HashSet::new();
    let mut slugs = std::collections::HashSet::new();
    for item in &seed.weapons {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("目录武器缺少 id")?;
        let slug = item
            .get("slug")
            .and_then(|v| v.as_str())
            .ok_or("目录武器缺少 slug")?;
        if id.is_empty()
            || slug.is_empty()
            || !item.get("i18n").map(|v| v.is_object()).unwrap_or(false)
        {
            return Err("目录武器关键字段无效".into());
        }
        let i18n = item
            .get("i18n")
            .and_then(|v| v.as_object())
            .ok_or("目录武器 i18n 无效")?;
        if i18n
            .values()
            .any(|v| v.get("name").and_then(|n| n.as_str()).is_none())
        {
            return Err("目录武器名称字段无效".into());
        }
        if !ids.insert(id) || !slugs.insert(slug) {
            return Err("目录武器 id/slug 重复".into());
        }
    }
    ids.clear();
    slugs.clear();
    for item in &seed.attributes {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("目录词条缺少 id")?;
        let slug = item
            .get("slug")
            .and_then(|v| v.as_str())
            .ok_or("目录词条缺少 slug")?;
        if id.is_empty()
            || slug.is_empty()
            || !item.get("i18n").map(|v| v.is_object()).unwrap_or(false)
        {
            return Err("目录词条关键字段无效".into());
        }
        let i18n = item
            .get("i18n")
            .and_then(|v| v.as_object())
            .ok_or("目录词条 i18n 无效")?;
        if i18n
            .values()
            .any(|v| v.get("name").and_then(|n| n.as_str()).is_none())
        {
            return Err("目录词条名称字段无效".into());
        }
        if !ids.insert(id) || !slugs.insert(slug) {
            return Err("目录词条 id/slug 重复".into());
        }
    }
    crate::market::catalog_from_seed(seed).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "wm-market-{label}-{}-{}-{}.sqlite3",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed),
            now_ms()
        ))
    }

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn stored_with_version(version: &str) -> StoredCatalog {
        let mut stored = CatalogCache::seed().expect("seed");
        stored.seed.collection_version = version.into();
        stored.raw = serde_json::to_vec(&stored.seed).expect("serialize");
        stored.status = status_for(&stored.seed, "sqlite", None);
        stored
    }

    #[test]
    fn seed_validates_and_preserves_raw_fields() {
        let stored = CatalogCache::seed().expect("embedded seed");
        assert_eq!(stored.seed.weapons.len(), 420);
        assert_eq!(stored.seed.attributes.len(), 50);
        assert!(stored.seed.weapons[0].get("i18n").is_some());
        assert_eq!(sha256(&stored.raw).len(), 64);
    }

    #[test]
    fn sqlite_reopens_and_corrupt_active_falls_back_previous() {
        let path = temp_path("fallback");
        let cache = CatalogCache::sqlite(path.clone());
        cache.save(&stored_with_version("one")).expect("first save");
        cache
            .save(&stored_with_version("two"))
            .expect("second save");
        let conn = Connection::open(&path).expect("open");
        conn.execute(
            "UPDATE catalog_snapshot SET payload=? WHERE id=1",
            params![b"broken"],
        )
        .expect("corrupt active");
        let loaded = cache.load().expect("load fallback").expect("previous");
        assert_eq!(loaded.seed.collection_version, "one");
        drop(conn);
        fs::remove_file(path).expect("remove test database");
    }

    #[test]
    fn typed_corruption_with_recomputed_hash_falls_back_previous() {
        let path = temp_path("typed-corrupt");
        let cache = CatalogCache::sqlite(path.clone());
        cache.save(&stored_with_version("one")).expect("first save");
        cache
            .save(&stored_with_version("two"))
            .expect("second save");
        let conn = Connection::open(&path).expect("open");
        let mut broken = stored_with_version("two").seed;
        broken.attributes[0]["positiveOnly"] = serde_json::json!("bad");
        let raw = serde_json::to_vec(&broken).expect("raw");
        conn.execute(
            "UPDATE catalog_snapshot SET payload=?, sha256=? WHERE id=1",
            params![raw.clone(), sha256(&raw)],
        )
        .expect("corrupt active");
        let loaded = cache.load().expect("fallback").expect("previous");
        assert_eq!(loaded.seed.collection_version, "one");
        drop(conn);
        fs::remove_file(path).expect("remove test database");
    }

    #[test]
    fn save_three_times_keeps_last_valid_previous() {
        let path = temp_path("three-save");
        let cache = CatalogCache::sqlite(path.clone());
        cache.save(&stored_with_version("one")).expect("one");
        cache.save(&stored_with_version("two")).expect("two");
        let conn = Connection::open(&path).expect("open");
        conn.execute(
            "UPDATE catalog_snapshot SET payload=? WHERE id=1",
            params![b"bad"],
        )
        .expect("corrupt active");
        drop(conn);
        cache.save(&stored_with_version("three")).expect("three");
        let conn = Connection::open(&path).expect("open newest");
        conn.execute(
            "UPDATE catalog_snapshot SET payload=? WHERE id=1",
            params![b"bad"],
        )
        .expect("corrupt newest active");
        assert_eq!(
            cache
                .load()
                .expect("load")
                .expect("previous")
                .seed
                .collection_version,
            "one"
        );
        drop(conn);
        fs::remove_file(path).expect("remove test database");
    }

    #[test]
    fn metadata_mismatch_and_both_bad_are_errors() {
        let path = temp_path("metadata");
        let cache = CatalogCache::sqlite(path.clone());
        cache.save(&stored_with_version("one")).expect("one");
        cache.save(&stored_with_version("two")).expect("two");
        let conn = Connection::open(&path).expect("open");
        conn.execute(
            "UPDATE catalog_snapshot SET collection_version=? WHERE id=1",
            params!["wrong"],
        )
        .expect("metadata");
        assert_eq!(
            cache
                .load()
                .expect("fallback")
                .expect("previous")
                .seed
                .collection_version,
            "one"
        );
        conn.execute(
            "UPDATE catalog_snapshot SET payload=? WHERE id=2",
            params![b"bad"],
        )
        .expect("bad previous");
        assert!(cache.load().is_err());
        drop(conn);
        fs::remove_file(path).expect("remove test database");
    }

    #[test]
    fn source_schema_and_duplicate_ids_are_rejected() {
        let mut seed = CatalogCache::seed().expect("seed").seed;
        seed.source = "https://example.invalid".into();
        assert!(validate_seed(&seed).is_err());
        let mut seed = CatalogCache::seed().expect("seed").seed;
        seed.schema_version = 2;
        assert!(validate_seed(&seed).is_err());
        let mut seed = CatalogCache::seed().expect("seed").seed;
        seed.weapons[1]["id"] = seed.weapons[0]["id"].clone();
        assert!(validate_seed(&seed).is_err());
    }

    #[test]
    fn extra_raw_fields_survive_reopen_and_touch_requires_match() {
        let path = temp_path("raw-extra");
        let cache = CatalogCache::sqlite(path.clone());
        let mut stored = stored_with_version("extra");
        stored.seed.weapons[0]["test_extra"] = serde_json::json!({"kept": true});
        stored.raw = serde_json::to_vec(&stored.seed).expect("raw");
        cache.save(&stored).expect("save");
        let reopened = cache.load().expect("load").expect("stored");
        assert_eq!(reopened.seed.weapons[0]["test_extra"]["kept"], true);
        let mut status = reopened.status.clone();
        status.collection_version = Some("missing".into());
        assert!(cache.touch_checked(&status).is_err());
        fs::remove_file(path).expect("remove test database");
    }

    #[test]
    fn failed_write_transaction_keeps_previous_snapshot() {
        let path = temp_path("rollback");
        let cache = CatalogCache::sqlite(path.clone());
        cache.save(&stored_with_version("one")).expect("one");
        let conn = Connection::open(&path).expect("open");
        conn.execute_batch("CREATE TRIGGER block_catalog_insert BEFORE INSERT ON catalog_snapshot BEGIN SELECT RAISE(ABORT, 'blocked'); END;").expect("trigger");
        assert!(cache.save(&stored_with_version("two")).is_err());
        assert_eq!(
            cache
                .load()
                .expect("load")
                .expect("previous")
                .seed
                .collection_version,
            "one"
        );
        drop(conn);
        fs::remove_file(path).expect("remove test database");
    }
}
