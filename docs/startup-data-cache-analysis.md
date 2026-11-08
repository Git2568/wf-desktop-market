# 启动目录等待与静态数据缓存分析

更新时间：2026-11-07（Asia/Shanghai）

## 当前启动链路

`MarketPage.onMounted` 先调用 `loadCatalog`，通过 `market_catalog` 串行取得武器目录和词条目录；两次目录 HTTP 都成功后才设置 `catalogReady`，用户点击查询才会请求挂单。启动不会下载完整游戏数据，也不会自动查询挂单。首次无缓存时，目录请求和限速会直接反映在首次可查询时间。

当前实现使用共享 Pinia 目录状态，并在 `app_local_data_dir/market-catalog.sqlite3` 保留本地目录；首次启动从随包真实 seed（420 武器、50 词条）导入，之后优先本地读取。设置页提供手动数据更新入口与 force 操作；不会在启动时自动联网更新，也没有后台更新任务。

所有 HTTP 共用至少 1 秒起始间隔，单次超时 15 秒；6 秒搜索间隔约束后续搜索，首次没有固定 6 秒延迟。`elapsed_ms` 覆盖挂单前 request gate 等待、HTTP 请求、解析归一化与排序，不包含目录和 search gate 等待。

## 已执行证据

Rust ignored 诊断 `live_startup_latency_probe` 已执行成功（仅同一后端 Service 链路，不含 GUI 或人工操作）：冷目录约 7144ms，热目录 0ms（毫秒精度下小于 1ms，request gate 未变化，证明未发 HTTP），从目录开始到首次 100 条挂单结果约 8975ms，snapshot `elapsed_ms` 1830ms；目录为 420 件武器、50 条词条。该样本只执行一次，不作为平均性能承诺；旧输出字段 `first_search_wall_ms` 后续改名为 `startup_to_results_ms`，无需重复测量。

版本接口样本：`GET https://api.warframe.market/v2/versions` 返回 HTTP 200，`apiVersion=0.25.0`，`collections.rivens='MjAyNi0xMC0yMVQxNjowMDoxMA=='`，`data.updatedAt=2026-10-28T03:52:09Z`。`apiVersion` 不是游戏版本；collections 值不在本项目文档中解码为唯一语义。wfm-docs manifests 文档说明 collections 版本 hash 可用于缓存更新；本次样本未见 ETag/Last-Modified，不据此断言接口整体不支持它们。

## 建议的后续实现

先在 SQLite/Rust 层缓存当前查询真正需要的静态目录：武器中英文名、id、slug、type，以及词条定义、翻译、单位和适用限制；已随包的四种极性图标不再重复下载。不缓存挂单价格或用户在线状态，它们仍应实时请求。

缓存元数据至少区分 app/schema 版本、数据源、`collections.rivens` 版本、语言、平台、`downloaded_at`、`last_checked_at` 和校验摘要；`game_version` 保留 nullable，只有取得可靠游戏版本来源时才填写，不能用 Market API 版本冒充。

以下是历史设计建议：启动优先读取有效旧缓存，不因更新阻断查询；更新入口检查 versions，无变化不下载目录，有变化才下载两目录，并共享限速。当前实现仅支持设置页手动更新/force，启动无联网；新目录完整校验后用事务切换活动版本，失败保留旧版本。

SQLite 保留原始两目录 JSON、SHA-256 与 `collections.rivens` 版本元数据，`game_version` 当前为 nullable，未用 Market API 版本冒充游戏版本。当前/previous 快照切换与事务更新已通过离线回归验证；失败保留旧快照与重试路径已纳入实现。实现进展详见 [catalog-cache.md](catalog-cache.md)。

## 实现验证

前端 typecheck/Vite build、Rust fmt/offline 检查与 native build 已通过；5 项真实目录 store 回归覆盖去重、并发加载/更新、旧状态晚回、force 与重复更新、失败保留/重试及无 Tauri。live startup probe 通过：首装本地目录 53ms、重开 24ms、搜索 6348ms、重开至结果 6373ms、总 probe 6426ms；目录 420/50、挂单 100，且 request gate 前后未变化，证明启动未发目录 HTTP。

优化前历史单次冷目录约 7144ms、目录至首次结果约 8975ms；旧挂单 1830ms 不与当前 6348ms 混作稳定网络基准。当前主要等待位于挂单阶段，不承诺固定耗时或提升比例。原生 GUI 仅完成可访问树读取，点击与截图环境故障，设置交互及安装包仍待验收。
