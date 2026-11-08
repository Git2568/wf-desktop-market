# Market API validation

验证日期：2026-11-07（Asia/Shanghai）。用户认可现有 UI 大体样式；打包已完成，完整视觉 GUI 与安装实测仍待完成。

## 首次直连通路基线

- v2 `/riven/weapons` with `Language: zh-hans`, `Platform: pc`: HTTP 200，观察到 420 个武器目录项。
- v2 `/riven/attributes`: HTTP 200，观察到 50 个词条目录项。
- v1 `/auctions/search?type=riven&sort_by=price_asc` without weapon: HTTP 400 (`requirements_not_met`)。
- v1 search with `weapon_url_name=torid`: HTTP 200，观察到 100 条记录；这是请求批次，不是全市场总量。
- 独立 `soma` 样本：100 条唯一、可见且未关闭记录，均为 `pc`；其中 44 条含负面词条。这同样只是样本。

历史首次通路使用直接 API，因为审计过的 `wf-market` 0.3.3 源码提供紫卡目录和普通订单 WebSocket 订阅，但没有带筛选的紫卡挂单搜索封装。紫卡 WebSocket 能力仍未验证，客户端没有声明已接通 WebSocket 或自动轮询。返回记录在 UI 中标为批次，价格标为卖家报价。

## 筛选修订验证

目录和搜索现在共用结构化 `MarketFilters` DTO。正面词条最多三项，负面词条单项；武器、词条方向、exclusiveTo、极性、MR、重洗次数范围和排序值均在发起网络搜索前校验。通过校验的所有筛选条件都会发送给 v1 服务端，客户端不会把筛选误做成对已取得 100 条记录的本地截断。

已验证的历史 Torid 组合为：正面 `critical_chance,critical_damage`，负面 `damage_vs_infested`，极性 `madurai`，MR `10..16`，重洗 `1..200`，价格降序。该请求 HTTP 200，返回 10 条记录。MR 最小值不属于当前服务端合同，当前请求只发送 MR 最大值；上述 `10..16` 仅保留为历史验证样本。展示价格按一口价优先、否则起拍价排序，无报价始终排在最后，同价按稳定 ID 排序。这个顺序只描述当前返回批次，不冒充全市场排序或总量。

已验证边界：非法极性和 MR 超出 `7..16` 返回 HTTP 400；合法边界 MR `7..16`、重洗 `0..999`、极性 `zenurik` 返回 HTTP 200 空数组。`re_rolls` 没有擅自设置 999 上限，`1000` 在本地 DTO 校验中保持合法。

### 强化等级搜索参数核验

- 不发送 `mod_rank`：Torid 请求 HTTP 200，100 条样本中同时出现 `item.mod_rank=0` 和 `8`。
- 发送 `mod_rank=maxed`：HTTP 200，100 条样本全部为 `item.mod_rank=8`；项目当前仅实现并验证“不限”和“已满级”两种服务端筛选，不据此断言上游不存在其它枚举。
- `mod_rank=0`、`mod_rank=8`、`mod_rank=unranked`、`mod_rank=unmaxed` 均被接口拒绝；其中 `unmaxed` 响应为 `{"error":{"mod_rank":["app.form.invalid"]}}`。不能据此伪造“未满级 0..7”筛选。
- 当前 `MarketFilters` 只允许 `mod_rank` 为空或 `maxed`，MR 只发送 `mastery_rank_max`，绝不发送 `mastery_rank_min`。挂单 DTO 的 `Listing.mod_rank` 仍是返回的整数等级，不与搜索枚举混用。

### 正面词条排序参数核验（2026-11-08）

- 官方 v1 `/auctions/search` 对 `weapon_url_name=torid` 接受 `sort_by=positive_attr_asc` 与 `sort_by=positive_attr_desc`；两种请求在不带 `positive_stats` 时均 HTTP 200、批次 100；分别带单项 `positive_stats=critical_chance` 时也均 HTTP 200、批次 100。
- 少量匿名样本中，单项 `critical_chance` 的升序与降序请求首尾均观察到 `182.6` 与 `160.2`，样本序列也未呈简单单调关系。因此本次只确认参数被接受和条件组合可请求，不宣称服务端已按原始暴击率值排序，也不推断多词条聚合算法。
- 合计 9 次第一方 Torid 请求（先 4 次、后 5 次）中，`price_asc` 的 ID 顺序 hash 为 `660ACFEA905E7727C66FF69BC7B03F0E72D7CF877C51AFDDBC856ED8BD9C85B1`；`price_desc`、`positive_attr_asc`、`positive_attr_desc` 在单 `critical_chance` 样本中的 ID 顺序 hash 均为 `EC272468B77993F8EB59FC81FA35A41AAB58C32A4994CAA7092ADFFF7BE3A82C`。`price_asc` 前三价格为 `299/300/300`，后三者前三均无 `buyout`，正面暴击率为 `182.6/179.9/179.9`；这些只是当前批次对比证据，不证明服务端正面排序效果。非法排序值返回 HTTP 400，错误字段为 `sort_by`、`app.form.invalid`。
- `MarketFilters.sort_order` 现接受 `asc`、`desc`、`positive_attr_asc`、`positive_attr_desc`；前两者映射为 `price_asc/price_desc` 并保留现有本地报价排序，新两者原样发送后，再对当前返回批次按 `positive_stats` 的选择顺序逐键比较原始正面数值，升/降序仅作用于当前批次。缺失或非有限值始终排最后，同值按稳定 ID；多词条不计算综合分、不归一化、不推断收益方向。
- 正面词条排序若未选择任何 `positive_stats`，本地校验会返回“正面词条排序至少选择一个正面词条”；价格排序仍允许无正面词条。

## 当前验收证据

- Rust `cargo test --offline`：7 passed，0 failed，1 ignored；覆盖新增 `mod_rank=maxed` 请求和非法枚举校验。
- `cargo fmt --check`：通过。
- 本轮正面排序补充：`cargo check --offline --tests`、`cargo fmt --check`、定向 `cargo test --offline --lib positive_sort`（1 passed）和 `git diff --check` 均通过；未运行完整 build 或全套测试。
- ignored `live_filtered_torid_smoke`：主 Agent 在升权网络环境执行通过（exit 0）；返回批次 22，耗时约 1130ms，`mod_rank=8`、MR `<=16`、词条、极性、重洗次数和展示价格顺序断言全部通过。输出未包含卖家明细。
- 完整视觉 GUI、键盘交互和安装实测：待完成，最终构建证据见下方。

## 最终构建与桌面验收状态

- 最终 `pnpm build`（包含 `vue-tsc`）通过。
- Windows Release 构建通过（约 25.23 秒）；NSIS 安装包构建通过（约 2.18 MiB）。
- 导出目录：`local-dist/0.1.0-20261107-124518-908`；两份导出文件 SHA-256 校验通过。目标为 x64 Windows，GUI subsystem 为 2。
- 新版桌面程序已启动；可访问性检查确认初始无查询结果，标题右侧存在“高级筛选”和“查询”，并显示 4 项简单筛选。
- 截图 `FrameArrived` 超时，随后出现连续用户输入保护，主 Agent 停止继续发送输入。因此完整视觉布局、键盘操作和实际组合框交互仍 pending；NSIS 安装程序尚未安装验证。

本轮仍未实现 WebSocket 或自动轮询。Retry-After 当前支持数字秒，HTTP-date 形式尚未支持。
