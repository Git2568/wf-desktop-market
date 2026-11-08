# Catalog seed cache

The bundled catalog seed is sourced from the Warframe.market public API and contains only riven weapon and attribute directory data. It does not contain listings, seller/user information, complete game statistics, or a game-version manifest.

- Seed: [`src-tauri/data/catalog-seed.json`](../src-tauri/data/catalog-seed.json)
- Source record: [`src-tauri/data/catalog-seed-sources.json`](../src-tauri/data/catalog-seed-sources.json)
- Schema: `1`
- Source: `https://api.warframe.market`
- Language/platform: `zh-hans` / `pc`
- Collection: `rivens`, version `MjAyNi0xMC0yMVQxNjowMDoxMA==`
- API version: `0.25.0`
- Counts: 420 weapons, 50 attributes
- Seed size: 266,368 bytes
- SHA-256: `9c33aab0f61e41605526a3ae72a8dfcc6a498f95d10f6fd9e79d17704880aef0`

The seed follows the backend contract `{ schema_version, source, language, platform, downloaded_at, collection_version, api_version, source_updated_at, weapons, attributes }`. The source record and collection version are provenance for the cached directory.

## 当前实现

- SQLite 文件位于 `app_local_data_dir/market-catalog.sqlite3`。
- 首次启动从包内真实 seed 导入，之后优先从本地 SQLite 读取；启动不联网。
- SQLite 保留原始武器/词条 JSON、SHA-256、集合版本元数据与 nullable `game_version`。
- 设置页提供手动检查与更新/重新下载（force）入口：无变化只请求一次 `versions`；发生变化或 force 时请求版本、两个目录并复查版本，共四次请求。
- 两个目录全部校验成功后，以事务写入 active 与最近有效 previous；失败保留旧数据，校验损坏时按 previous 再回退到 seed。
- 前端通过共享 Pinia 目录状态更新，不清空已有挂单数据。

## 验证

20 项 Rust offline 回归、5 项 store 回归、前端 typecheck/Vite build 与 native build 均通过；独立 live probe 通过：首装本地 53ms、重开 24ms、搜索 6348ms、重开至结果 6373ms。该网络样本为单次结果，不是稳定基准。完整 GUI 设置交互因明确的窗口捕获/坐标定位环境错误未验收；debug 实例已关闭。
