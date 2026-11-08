use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedAnalysisData {
    pub payload: Vec<u8>,
    pub fetched_at: i64,
    pub source_updated_at: Option<String>,
    pub source: String,
    pub recovered: bool,
}

pub struct AnalysisCache {
    path: PathBuf,
}

impl AnalysisCache {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self, kind: &str, key: &str) -> Result<Option<CachedAnalysisData>, String> {
        if !self.path.exists() {
            return Ok(None);
        }
        let conn = Connection::open(&self.path).map_err(|e| format!("无法打开分析缓存：{e}"))?;
        if !table_exists(&conn)? {
            return Ok(None);
        }
        let current = read_slot(&conn, kind, key, 1);
        match current {
            Ok(Some(row)) if verify_row(&row).is_ok() => Ok(Some(to_data(row, false))),
            Err(_) | Ok(Some(_)) => match read_slot(&conn, kind, key, 2) {
                Err(_) => Err("分析缓存当前与上一份快照均已损坏".into()),
                Ok(previous) => match previous {
                    Some(previous) if verify_row(&previous).is_ok() => {
                        Ok(Some(to_data(previous, true)))
                    }
                    Some(_) => Err("分析缓存当前与上一份快照均已损坏".into()),
                    None => Err("分析缓存当前快照已损坏，且没有可回退快照".into()),
                },
            },
            Ok(None) => match read_slot(&conn, kind, key, 2) {
                Err(_) => Err("分析缓存上一份快照已损坏".into()),
                Ok(previous) => match previous {
                    Some(previous) if verify_row(&previous).is_ok() => {
                        Ok(Some(to_data(previous, true)))
                    }
                    Some(_) => Err("分析缓存上一份快照已损坏".into()),
                    None => Ok(None),
                },
            },
        }
    }

    pub fn save(&self, kind: &str, key: &str, data: &CachedAnalysisData) -> Result<(), String> {
        if kind.is_empty() || key.is_empty() {
            return Err("分析缓存 kind/key 不能为空".into());
        }
        if data.payload.is_empty() {
            return Err("分析缓存内容不能为空".into());
        }
        if serde_json::from_slice::<serde_json::Value>(&data.payload).is_err() {
            return Err("分析缓存内容不是有效 JSON".into());
        }
        if data.fetched_at <= 0 {
            return Err("分析缓存时间必须大于 0".into());
        }
        if data.source.is_empty() {
            return Err("分析缓存来源不能为空".into());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("无法创建分析缓存目录：{e}"))?;
        }
        let mut conn =
            Connection::open(&self.path).map_err(|e| format!("无法打开分析缓存：{e}"))?;
        create_table(&conn)?;
        let tx = conn
            .transaction()
            .map_err(|e| format!("无法开始分析缓存事务：{e}"))?;
        let new_sha = sha256(&data.payload);
        let existing = read_slot(&tx, kind, key, 1).ok().flatten();
        if let Some(row) = existing {
            if verify_row(&row).is_ok() && row.sha256 == new_sha {
                tx.execute("UPDATE analysis_snapshots SET fetched_at=?4, source_updated_at=?5, source=?6 WHERE kind=?1 AND key=?2 AND slot=1", params![kind, key, 1i64, data.fetched_at, data.source_updated_at, data.source])
                    .map_err(|e| format!("无法更新分析缓存元信息：{e}"))?;
            } else if verify_row(&row).is_ok() {
                tx.execute(
                    "DELETE FROM analysis_snapshots WHERE kind=?1 AND key=?2 AND slot=2",
                    params![kind, key],
                )
                .map_err(|e| format!("无法轮换分析缓存：{e}"))?;
                tx.execute("INSERT INTO analysis_snapshots(kind,key,slot,payload,sha256,fetched_at,source_updated_at,source) SELECT kind,key,2,payload,sha256,fetched_at,source_updated_at,source FROM analysis_snapshots WHERE kind=?1 AND key=?2 AND slot=1", params![kind, key]).map_err(|e| format!("无法备份分析缓存：{e}"))?;
                upsert_current(&tx, kind, key, data, &new_sha)?;
            } else {
                upsert_current(&tx, kind, key, data, &new_sha)?;
            }
        } else {
            upsert_current(&tx, kind, key, data, &new_sha)?;
        }
        tx.commit().map_err(|e| format!("无法提交分析缓存：{e}"))
    }
}

fn table_exists(conn: &Connection) -> Result<bool, String> {
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='analysis_snapshots'",
        [],
        |_| Ok(1i32),
    )
    .optional()
    .map(|x| x.is_some())
    .map_err(|e| format!("无法检查分析缓存表：{e}"))
}
fn create_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS analysis_snapshots (kind TEXT NOT NULL, key TEXT NOT NULL, slot INTEGER NOT NULL CHECK(slot IN (1,2)), payload BLOB NOT NULL, sha256 TEXT NOT NULL, fetched_at INTEGER NOT NULL, source_updated_at TEXT, source TEXT NOT NULL, PRIMARY KEY(kind,key,slot));").map_err(|e| format!("无法创建分析缓存表：{e}"))
}
#[derive(Debug)]
struct Row {
    payload: Vec<u8>,
    sha256: String,
    fetched_at: i64,
    source_updated_at: Option<String>,
    source: String,
}
fn read_slot(conn: &Connection, kind: &str, key: &str, slot: i64) -> Result<Option<Row>, String> {
    conn.query_row("SELECT payload,sha256,fetched_at,source_updated_at,source FROM analysis_snapshots WHERE kind=?1 AND key=?2 AND slot=?3", params![kind,key,slot], |r| Ok(Row { payload:r.get(0)?, sha256:r.get(1)?, fetched_at:r.get(2)?, source_updated_at:r.get(3)?, source:r.get(4)? })).optional().map_err(|e| format!("无法读取分析缓存：{e}"))
}
fn verify_row(row: &Row) -> Result<(), String> {
    if row.payload.is_empty()
        || row.fetched_at <= 0
        || row.source.is_empty()
        || serde_json::from_slice::<serde_json::Value>(&row.payload).is_err()
        || sha256(&row.payload) != row.sha256
    {
        return Err("分析缓存校验失败".into());
    }
    Ok(())
}
fn to_data(row: Row, recovered: bool) -> CachedAnalysisData {
    CachedAnalysisData {
        payload: row.payload,
        fetched_at: row.fetched_at,
        source_updated_at: row.source_updated_at,
        source: row.source,
        recovered,
    }
}
fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}
fn upsert_current(
    conn: &Connection,
    kind: &str,
    key: &str,
    data: &CachedAnalysisData,
    digest: &str,
) -> Result<(), String> {
    conn.execute("INSERT INTO analysis_snapshots(kind,key,slot,payload,sha256,fetched_at,source_updated_at,source) VALUES(?1,?2,1,?3,?4,?5,?6,?7) ON CONFLICT(kind,key,slot) DO UPDATE SET payload=excluded.payload,sha256=excluded.sha256,fetched_at=excluded.fetched_at,source_updated_at=excluded.source_updated_at,source=excluded.source", params![kind,key,data.payload,digest,data.fetched_at,data.source_updated_at,data.source]).map(|_| ()).map_err(|e| format!("无法写入分析缓存：{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    fn temp() -> PathBuf {
        std::env::temp_dir().join(format!(
            "wm-analysis-cache-{}-{}.sqlite",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    fn data(payload: &[u8], n: i64) -> CachedAnalysisData {
        CachedAnalysisData {
            payload: payload.to_vec(),
            fetched_at: n,
            source_updated_at: Some("v1".into()),
            source: "test".into(),
            recovered: false,
        }
    }
    #[test]
    fn round_trip_and_metadata() {
        let p = temp();
        let c = AnalysisCache::new(p.clone());
        c.save("riven", "x", &data(br#"{"x":1}"#, 1)).unwrap();
        let x = c.load("riven", "x").unwrap().unwrap();
        assert_eq!(x.fetched_at, 1);
        assert!(!x.recovered);
        let _ = fs::remove_file(p);
    }
    #[test]
    fn deduplicates_same_payload() {
        let p = temp();
        let c = AnalysisCache::new(p.clone());
        c.save("riven", "x", &data(br#"{"x":1}"#, 1)).unwrap();
        c.save("riven", "x", &data(br#"{"x":1}"#, 2)).unwrap();
        let conn = Connection::open(&p).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT count(*) FROM analysis_snapshots WHERE kind='riven' AND key='x'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            1
        );
        let _ = fs::remove_file(p);
    }
    #[test]
    fn corrupt_current_recovers_previous() {
        let p = temp();
        let c = AnalysisCache::new(p.clone());
        c.save("riven", "x", &data(br#"{"x":1}"#, 1)).unwrap();
        c.save("riven", "x", &data(br#"{"x":2}"#, 2)).unwrap();
        let conn = Connection::open(&p).unwrap();
        conn.execute(
            "UPDATE analysis_snapshots SET payload='bad' WHERE slot=1",
            [],
        )
        .unwrap();
        let x = c.load("riven", "x").unwrap().unwrap();
        assert!(x.recovered);
        assert_eq!(x.payload, br#"{"x":1}"#);
        let _ = fs::remove_file(p);
    }

    #[test]
    fn corrupt_both_returns_error_and_read_missing_does_not_create_db() {
        let p = temp();
        let c = AnalysisCache::new(p.clone());
        assert!(c.load("riven", "x").unwrap().is_none());
        assert!(!p.exists());
        c.save("riven", "x", &data(br#"{"x":1}"#, 1)).unwrap();
        c.save("riven", "x", &data(br#"{"x":2}"#, 2)).unwrap();
        let conn = Connection::open(&p).unwrap();
        conn.execute("UPDATE analysis_snapshots SET payload='bad'", [])
            .unwrap();
        assert!(c.load("riven", "x").is_err());
        let _ = fs::remove_file(p);
    }

    #[test]
    fn corrupt_current_without_previous_errors_and_bad_metadata_recovers() {
        let p = temp();
        let c = AnalysisCache::new(p.clone());
        c.save("riven", "x", &data(br#"{"x":1}"#, 1)).unwrap();
        let conn = Connection::open(&p).unwrap();
        conn.execute(
            "UPDATE analysis_snapshots SET payload='bad' WHERE slot=1",
            [],
        )
        .unwrap();
        assert!(c.load("riven", "x").is_err());
        c.save("riven", "x", &data(br#"{"x":2}"#, 2)).unwrap();
        c.save("riven", "x", &data(br#"{"x":3}"#, 3)).unwrap();
        conn.execute(
            "UPDATE analysis_snapshots SET fetched_at=0 WHERE slot=1",
            [],
        )
        .unwrap();
        let recovered = c.load("riven", "x").unwrap().unwrap();
        assert!(recovered.recovered);
        assert_eq!(recovered.payload, br#"{"x":2}"#);
        let _ = fs::remove_file(p);
    }

    #[test]
    fn rejects_invalid_input() {
        let p = temp();
        let c = AnalysisCache::new(p.clone());
        assert!(c.save("riven", "x", &data(b"not-json", 1)).is_err());
        assert!(c.save("riven", "x", &data(br#"{"x":1}"#, 0)).is_err());
        let _ = fs::remove_file(p);
    }
}
