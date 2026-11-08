# 外部资料索引

更新时间：2026-11-08（Asia/Shanghai）

2026-11-07 补充核验：目录缓存实现使用 SQLite 与 Tauri 本地数据目录 API；相关官方入口见 [rusqlite 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/) 与 [Tauri path API](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html)。

2026-11-06 为历史在线文档、仓库和入口核查；2026-11-07 已完成真实 API 请求、构建和桌面启动验证，结果记录在 `docs/market-api-validation.md`。下列 P0–P4 是本项目的查阅与评估优先级，不等于数据官方权威；“官方权威”和“项目相关性”分别标注，需结合核验状态判断。

## 推荐查阅顺序

| 优先级 | 查阅重点 | 排序依据 |
| --- | --- | --- |
| P0 | 核心接口与开发约束 | 直接影响市场接入、实时能力和当前 UI 开发规范 |
| P1 | 官方/增强物品、翻译与模拟基础数据 | 支撑名称、字段、版本和模拟校验 |
| P2 | 社区 API、世界状态与解析工具 | 辅助数据和解析，不能替代市场挂单源 |
| P3 | 其他语言客户端和实现参考 | 比较封装与流程，当前未选作项目依赖 |
| P4 | 官网、Wiki、MCP 和探索性资料 | 补充背景，非客户端必需接入 |

## P0 核心接口与开发约束

### 2026-11-07 filtered riven API verification

- Direct API validation covered `/v2/riven/weapons`, `/v2/riven/attributes`, and `/v1/auctions/search` with server-side positive/negative stats, polarity, mastery rank, reroll range, and price sort parameters. The client records this as a read-only experiment; it does not claim WebSocket riven subscriptions or automatic polling.

本层按项目相关性最高排序；市场文档属于 Warframe.market 生态来源，`wf-market` 是第三方 Rust 客户端库，不等同市场自有接口。

### Hugeicons 官方图标库

- URL：[https://hugeicons.com/](https://hugeicons.com/)
- 用途：客户端统一 UI 图标来源；禁止以其他图标库、Emoji、Unicode 符号或临时手绘 SVG 替代。
- 来源/核验状态：官方图标库入口，作为当前 UI 开发约束；具体图标导出和许可范围需在接入时按官方资料核对。
- 核验日期：2026-11-06（Asia/Shanghai）

### Hugeicons Vue 3 快速入门

- URL：[https://hugeicons.com/docs/integrations/vue/quick-start](https://hugeicons.com/docs/integrations/vue/quick-start)
- 用途：Vue 3 集成 `@hugeicons/vue`、`@hugeicons/core-free-icons` 与 `HugeiconsIcon` 的官方用法参考。
- 来源/核验状态：官方 Vue 集成文档已在线核实包名及 `:icon` 用法；实际使用的图标名称和导出仍需逐项核对。
- 核验日期：2026-11-06（Asia/Shanghai）

### Warframe.market API 概览

- URL：[https://docs.warframe.market/docs/api/overview/](https://docs.warframe.market/docs/api/overview/)
- 用途：HTTP/REST API 概览入口。
- 来源/核验状态：来自 Warframe.market 文档站导航，正文尚未核实；不能据此确认紫卡实时推送、鉴权或限流。
- 核验日期：2026-11-06（Asia/Shanghai）

2026-11-07 实测 v2 武器目录 420 项、词条目录 50 项；v1 Torid 挂单批次 100 项；无武器条件 HTTP 400。详细结果见 `docs/market-api-validation.md`。相关入口：[manifests](https://github.com/42bytes-team/wfm-docs/blob/master/docs/api/manifests.mdx)。同日核验 [versions](https://api.warframe.market/v2/versions) 返回 `apiVersion=0.25.0`、`collections.rivens` 和 `data.updatedAt`；用于缓存版本检查，不当作游戏版本。manifests 文档说明 collections hash 可用于缓存更新，详见 `docs/startup-data-cache-analysis.md`。

2026-11-07 追加核验 v1 Torid `/auctions/search` 的强化等级参数：不发送 `mod_rank` 返回 rank 0/8 混合样本；`mod_rank=maxed` 返回样本全部为 `item.mod_rank=8`；整数 `0/8` 与 `unranked/unmaxed` 被接口拒绝，`unmaxed` 响应为 `app.form.invalid`。因此项目只采用“不限/已满级”，不宣称存在未满级枚举；详见 `docs/market-api-validation.md`。该参数核验为真实匿名请求记录，非当前 v2 文档合同。

2026-11-08 追加核验 v1 Torid `/auctions/search` 的 `sort_by=positive_attr_asc/positive_attr_desc`：不带 `positive_stats` 及带 `critical_chance` 的请求均 HTTP 200、批次 100；服务端排序效果未被证明。当前用户批准的实现会在当前返回批次内按 `positive_stats` 选择顺序逐键比较正面原始数值，缺失/非有限值末位，同值按 ID 稳定排序；不宣称全市场服务端排序或多词条综合算法。详见 `docs/market-api-validation.md`。

### Tauri Windows installer

- URL：[https://v2.tauri.app/distribute/windows-installer/](https://v2.tauri.app/distribute/windows-installer/)
- 用途：Tauri 官方 Windows 安装包与 NSIS 分发说明；用于本地 Release/NSIS 打包脚本的文档核对。
- 来源/核验状态：官方 Tauri v2 文档，2026-11-07 已核验入口；本项目安装包已生成，但尚未在用户机器上实际安装测试。
- 核验日期：2026-11-07（Asia/Shanghai）

### MDN scrollbar pseudo-elements

- URL：[https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Selectors/::-webkit-scrollbar](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Selectors/::-webkit-scrollbar)
- 用途：核对 Chromium 滚动条伪元素与标准 `scrollbar-color`/`scrollbar-width` 的兼容回退边界；项目仅用于样式实现参考。
- 来源/核验状态：MDN 文档，2026-11-07 已核验；`::-webkit-scrollbar` 为非标准伪元素，标准属性非 `auto` 时可能覆盖伪元素规则，因此项目在 `@supports selector(::-webkit-scrollbar)` 中显式回退。
- 核验日期：2026-11-07（Asia/Shanghai）

### Warframe.market WebSocket 概览

- URL：[https://docs.warframe.market/docs/websockets/overview/](https://docs.warframe.market/docs/websockets/overview/)
- 用途：市场 WebSocket 协议和连接方式资料。
- 来源/核验状态：页面可读并已初核入口；紫卡事件覆盖、实际连通和鉴权仍未验证。
- 核验日期：2026-11-06（Asia/Shanghai）

### wfm-docs API 文档仓库

- URL：[https://github.com/42bytes-team/wfm-docs](https://github.com/42bytes-team/wfm-docs)
- 用途：Warframe.market 公共 API、OAuth 和 WebSocket 文档候选来源。
- 来源/核验状态：仓库可读，默认分支为 `master`；README 自述为 Warframe.market 公共文档，属于市场自有文档来源，不等同 Digital Extremes 官方。接口覆盖、鉴权、限流和紫卡实时能力仍待核实。
- 核验日期：2026-11-06（Asia/Shanghai）

### wfm-docs API overview（核心文件）

- URL：[https://github.com/42bytes-team/wfm-docs/blob/master/docs/api/overview.mdx](https://github.com/42bytes-team/wfm-docs/blob/master/docs/api/overview.mdx)
- 用途：API 总览核心入口。
- 来源/核验状态：仓库目录和核心正文已读取；2026-11-07 已用其文档核对 v2 目录、响应封套和限流说明，并完成真实请求样本。紫卡 WebSocket、鉴权完整语义仍待复核。
- 核验日期：2026-11-07（Asia/Shanghai）

### wf-market Rust 客户端库

- 主仓库：[https://github.com/KibbeWater/wf-market](https://github.com/KibbeWater/wf-market)
- 辅助入口：[crates.io](https://crates.io/crates/wf-market)、[docs.rs](https://docs.rs/wf-market/latest/wf_market/)
- 用途：候选 Rust 市场客户端库及 API/WebSocket 封装参考。
- 来源/核验状态：第三方 Rust 客户端库；此前 README/API 文档层初核显示 `wf_market 0.3.3`、可选 `websocket` feature 和订单实时示例。crates.io 最新发布版本未独立核验，实际版本未锁定；源码、依赖构建、紫卡事件覆盖、协议和联调仍待核实，订单示例不等于紫卡实时挂单支持。
- 核验日期：2026-11-06（Asia/Shanghai）

本轮源码核验：0.3.3 的 `src/api/rivens.rs` 提供紫卡武器和词条目录；`src/api/v1/mod.rs` 仅挂接旧版统计模块，未发现按武器筛选紫卡挂单的封装。其 WebSocket 文档覆盖普通新订单，不能视为紫卡行情推送。源码入口：[rivens.rs](https://github.com/KibbeWater/wf-market/blob/main/src/api/rivens.rs)、[v1/mod.rs](https://github.com/KibbeWater/wf-market/blob/main/src/api/v1/mod.rs)。

官方订阅与规则入口：[WebSocket subscriptions](https://github.com/42bytes-team/wfm-docs/blob/master/docs/websockets/subscriptions.mdx)、[market rules](https://github.com/42bytes-team/wfm-docs/blob/master/docs/rules/overview.md)。

## P1 物品、翻译与模拟基础

本层按官方原始来源优先、社区增强数据次之排序；官方权威不代表已具备市场价格或挂单数据。

### Digital Extremes Public Export

- URL：[https://content.warframe.com/PublicExport/](https://content.warframe.com/PublicExport/)
- 用途：官方物品、语言和游戏基础数据校验来源，与市场价格/挂单数据分开。
- 来源/核验状态：用户提供的 Digital Extremes 官方入口，官方权威性明确；本轮未能读取官方入口，索引获取方式、认证、JSON 覆盖与解析待样本验证。
- 核验日期：2026-11-06（Asia/Shanghai）

### calamity-inc/warframe-public-export-plus

- URL：[https://github.com/calamity-inc/warframe-public-export-plus](https://github.com/calamity-inc/warframe-public-export-plus)
- 用途：研究增强物品数据、`Export*.json` 与 `dict.en.json`/`dict.zh.json` 翻译映射。
- 来源/核验状态：社区增强数据项目，默认分支 `senpai`；README/仓库树已确认相关文件，项目自述“更完整”未做对比验证，不等同官方原始接口。
- 核验日期：2026-11-06（Asia/Shanghai）

### browse.wf

- URL：[https://browse.wf/](https://browse.wf/)、[About](https://browse.wf/about)
- 用途：浏览游戏内部路径、语言标签、物品和图片的社区查询工具。
- 来源/核验状态：社区工具；About 确认使用上述 `warframe-public-export-plus` 数据，适合作为辅助浏览入口，不冒充官方 API。
- 核验日期：2026-11-06（Asia/Shanghai）

### Polarity icon asset verification (2026-11-07)

- [`ExportTextIcons.json`](https://github.com/calamity-inc/warframe-public-export-plus/blob/senpai/ExportTextIcons.json) maps `POLARITY_ATTACK` to `/Lotus/Interface/PolarityTriangle.png`, `POLARITY_DEFENSE` to `PolarityPoint.png`, `POLARITY_TACTIC` to `PolarityCircle.png`, and `POLARITY_POWER` to `PolarityMark.png`. [`dict.en.json`](https://github.com/calamity-inc/warframe-public-export-plus/blob/senpai/dict.en.json) maps the corresponding names to Madurai, Vazarin, Naramon, and Zenurik. The repository [`README.md`](https://github.com/calamity-inc/warframe-public-export-plus/blob/senpai/README.md) and browse.wf documentation identify this as a community mirror of game assets; it does not make Digital Extremes the hosting source.
- Verified asset URLs: [Madurai](https://browse.wf/Lotus/Interface/PolarityTriangle.png), [Vazarin](https://browse.wf/Lotus/Interface/PolarityPoint.png), [Naramon](https://browse.wf/Lotus/Interface/PolarityCircle.png), [Zenurik](https://browse.wf/Lotus/Interface/PolarityMark.png). Each returned HTTP 200 with a PNG signature on 2026-11-07 (Asia/Shanghai).
- The repository stores the downloaded bytes under `src/assets/polarities/`; `sources.json` records the local file, original in-game path, mapping token, source URL, dimensions, byte count, and SHA-256. Copyright remains with the original rights holders; licensing was not inferred.

### calamity-inc/warframe-public-export

- URL：[https://github.com/calamity-inc/warframe-public-export](https://github.com/calamity-inc/warframe-public-export)
- 用途：Public Export 清洗格式和 corrections 处理参考。
- 来源/核验状态：社区格式镜像；README 说明数据本身不改、需要部分 corrections，仅作格式辅助，不推断所有官方 JSON 均有问题。
- 核验日期：2026-11-06（Asia/Shanghai）

### WFCD/warframe-items（含 @wfcd/items）

- 主仓库：[https://github.com/WFCD/warframe-items](https://github.com/WFCD/warframe-items)
- 辅助路径：[物品数据目录](https://github.com/WFCD/warframe-items/tree/master/data/json)、[i18n](https://github.com/WFCD/warframe-items/blob/master/data/json/i18n.json)、[WFCD 组织](https://github.com/WFCD)、[物品 CDN](https://cdn.warframestat.us/)
- 用途：候选中英文名称、字段映射、物品基础资料和图片资源参考；`@wfcd/items` 与该项目合并记录。
- 来源/核验状态：WFCD 社区维护，与 Digital Extremes 无隶属关系；README 初核确认含 `zh` i18n，辅助路径来自 README。文件正文、字段覆盖、数据新鲜度、市场标识对齐及 CDN 图片用法未逐项验证。
- 核验日期：2026-11-06（Asia/Shanghai）

### WFCD mod-generator / genesis-assets mod frames

- URL：[WFCD/mod-generator](https://github.com/WFCD/mod-generator)、[@wfcd/mod-generator](https://www.npmjs.com/package/@wfcd/mod-generator)、[genesis-assets/modFrames](https://github.com/WFCD/genesis-assets/tree/master/modFrames)
- 用途：按用户确认使用 WFCD 社区镜像组件组合紫卡空白卡与本地预览资源，链接 [riven-card-assets.md](docs/riven-card-assets.md)。
- 来源/核验状态：WFCD 社区原组件与资源，已记录为社区来源；不冒称 Digital Extremes 官方直接提供完整紫卡图片。本地已保存 7 个 PNG 并接入 `RivenCardPreview` 组件。
- 核验日期：2026-11-08（Asia/Shanghai）

### 2026-11-09 项目实现核验补充

- 用途：记录本项目洗卡 core/planner 接入状态与验证边界。
- 来源/核验状态：基于上方 riven-roll commit `986f37c7ae0057be4a4c3e8251cb03080507241a`、776 项熔接 oracle 对照及正式 Rust 定向测试；不是官方概率声明。概率采用上游无放回/均匀 W 模型，保留 2M 工作预算。PlagueKeewar、Kripath PvP、`vinquibus_melee` 无准确 gameRef 映射。
- 核验日期：2026-11-09（Asia/Shanghai）

## P2 社区 API 与解析工具

本层用于世界状态、社区统计和解析辅助；世界状态或社区紫卡统计不等于 Warframe.market 实时挂单。

### warframestat.us / WFCD 世界状态 API

- URL：[https://docs.warframestat.us/](https://docs.warframestat.us/)、[warframe-status](https://github.com/WFCD/warframe-status)、[warframestat.us](https://warframestat.us/)
- 用途：世界状态 REST-ish API、`/socket` 更新，以及 items/drops/rivens 等社区接口参考。
- 来源/核验状态：WFCD 社区项目，与 Digital Extremes 无隶属关系；源码 README 提供 REST-ish 与 `/socket` 世界状态更新证据，统计细节来自文档搜索索引；文档正文及主页本轮未获取可读内容。不要将其当作 Warframe.market 实时挂单源或记录未经核验的固定限流数值；历史 `WFCD/api-spec` 已归档，文档生成迁移至 `warframe-status`。
- 核验日期：2026-11-06（Asia/Shanghai）

### WFCD/warframe-worldstate-parser

- URL：[https://github.com/WFCD/warframe-worldstate-parser](https://github.com/WFCD/warframe-worldstate-parser)
- 用途：将世界状态响应解析为 JavaScript 对象。
- 来源/核验状态：社区解析工具；README 示例使用 `api.warframe.com/cdn/worldState.php`，属于世界状态解析，不是市场客户端。
- 核验日期：2026-11-06（Asia/Shanghai）

### warframe-info-api（名称匹配候选）

- URL：[https://github.com/WsureWarframe/warframe-info-api](https://github.com/WsureWarframe/warframe-info-api)
- 用途：研究世界状态、Market 物品/紫卡与 riven.market 查询及中英文模糊名称匹配。
- 来源/核验状态：按名称检索到的匹配候选，用户未指定确切仓库；README 描述 Express/Node 能力，Wiki 查询仅拼接搜索链接，`/wf/电波` 示例未核实；在线服务、当前协议兼容性、示例端点和活跃度待核验。
- 核验日期：2026-11-06（Asia/Shanghai）

### Warframe.js

- URL：[https://github.com/WFCD/Warframe.js](https://github.com/WFCD/Warframe.js)
- 用途：Node/浏览器世界状态封装，研究警报、裂缝和突击等状态数据处理。
- 来源/核验状态：社区 npm 包 README 已核对；不是 Warframe.market 客户端或挂单数据源。
- 核验日期：2026-11-06（Asia/Shanghai）

## P3 其他语言客户端与实现参考

本层用于比较接口封装和流程，当前未选作项目依赖。

### pywmapi

- URL：[https://github.com/leonardodalinky/pywmapi](https://github.com/leonardodalinky/pywmapi)、[PyPI](https://pypi.org/project/pywmapi/)
- 用途：Python Market API 封装、items/orders/riven 模板与属性参考。
- 来源/核验状态：项目身份可确认；当前 GitHub README 说明创建拍卖不可用，检索/详情/竞价未实现；PyPI 旧描述与当前 README 有差异。README 转述的每秒 3 次限制未经现行官方规则独立确认。
- 核验日期：2026-11-06（Asia/Shanghai）

### warframe-market.py

- URL：[https://github.com/aldi-f/warframe-market.py](https://github.com/aldi-f/warframe-market.py)、[PyPI](https://pypi.org/project/warframe-market.py/)
- 用途：异步 Market API v2 Python 客户端、物品/多语言和最优买卖单示例参考。
- 来源/核验状态：项目身份可确认；自述仍处开发阶段，riven 与 WebSocket 覆盖未核实。
- 核验日期：2026-11-06（Asia/Shanghai）

### Riven-Finder

- URL：[https://github.com/tvojk13/Riven-Finder](https://github.com/tvojk13/Riven-Finder)、[README](https://github.com/tvojk13/Riven-Finder/blob/master/README.md)
- 用途：自定义紫卡查询、定期 API 轮询、拍卖结果处理、任务管理和 Market 登录流程参考。
- 来源/核验状态：第三方 C# 项目；README/仓库元信息初核显示 C#/.NET 6.0、按武器/最高一口价/卖家状态过滤和 Market 凭据登录。源码认证流程、交易操作范围、现行 API 兼容性和 WebSocket 能力仍待核实。
- 核验日期：2026-11-06（Asia/Shanghai）

### Warframe Market 物品名称中英对照显示脚本

- 主要入口：[Greasy Fork 用户脚本源码](https://update.greasyfork.org/scripts/558234/Warframe%20Market%20%E7%89%A9%E5%93%81%E5%90%8D%E7%A7%B0%E4%B8%AD%E8%8B%B1%E5%AF%B9%E7%85%A7%E6%98%BE%E7%A4%BA%20%E6%94%AF%E6%8C%81%E6%90%9C%E7%B4%A2.user.js)
- 补充来源：用户本机 Tampermonkey 安装页 `chrome-extension://dhdgffkkebhmkfjojejmpbldmpobfkfo/ask.html?aid=a3262605-0bee-4468-b449-aa6a64f4fe3d`（不作为可共享源码或数据接口）。
- 用途：候选中英文展示方法参考；用户描述支持“英文（中文）”、搜索、紫卡和玄骸幻纹翻译。
- 来源/核验状态：第三方用户脚本；已尝试抓取源码但本次工具未获取正文，方法、覆盖和数据源待核实。
- 核验日期：2026-11-06（Asia/Shanghai）

## P4 辅助资料与探索

本层补充官方背景、社区资料和工具探索，不是当前客户端数据接入必需项。

### FluentQtWidgets 主题机制参考

- URL：[项目仓库](https://github.com/txp666/FluentQtWidgets)、[CardWidget.cpp](https://github.com/txp666/FluentQtWidgets/blob/main/src/widgets/CardWidget.cpp)、[Theme.cpp](https://github.com/txp666/FluentQtWidgets/blob/main/src/core/Theme.cpp)、[FluentWindow.cpp](https://github.com/txp666/FluentQtWidgets/blob/main/src/window/FluentWindow.cpp)、[Gallery interface](https://github.com/txp666/FluentQtWidgets/blob/main/examples/gallery/app/view/gallery_interface.cpp)、[LICENSE](https://github.com/txp666/FluentQtWidgets/blob/main/LICENSE)
- 用途：学习主题统一应用、从当前动画值连续过渡、重复切换合并和系统背景 crossfade 的一般机制，作为本项目 Web 主题过渡的设计参考。
- 来源/核验状态：第三方 Qt 项目，用户指定仓库并已核对 `main` 分支相关源码入口；GitHub API 返回受限，未记录 commit SHA。README/LICENSE 标示 GPL-3.0-or-later。本项目仅独立实现一般动效逻辑，不复制 Qt 源码、图标或依赖；上游没有被核实为圆形全屏揭幕实现。
- 核验日期：2026-11-07（Asia/Shanghai）

### Web 主题过渡 API

- URL：[CSS.registerProperty](https://developer.mozilla.org/en-US/docs/Web/API/CSS/registerProperty_static)、[Element.animate](https://developer.mozilla.org/en-US/docs/Web/API/Element/animate)、[prefers-reduced-motion](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion)
- 用途：本项目独立 TypeScript 主题过渡的 Web API、兼容性回退和减少动态效果依据。
- 来源/核验状态：MDN 文档已浏览核对；不属于 Qt 项目来源。
- 核验日期：2026-11-07（Asia/Shanghai）

### Warframe.market 交易市场站

- URL：[https://warframe.market/](https://warframe.market/)
- 用途：第三方交易市场站入口。
- 来源/核验状态：页面可访问但无可读正文；首页访问初核，业务内容未核实。
- 核验日期：2026-11-06（Asia/Shanghai）

### Warframe 官方网站

- URL：[https://www.warframe.com/](https://www.warframe.com/)
- 用途：Digital Extremes 官方网站和资料导航。
- 来源/核验状态：页面可访问并跳转 `/en`，官方归属明确；不构成市场接口或模拟数据已核实。
- 核验日期：2026-11-06（Asia/Shanghai）

### Warframe Wiki 社区资料站

- URL：[https://wiki.warframe.com/](https://wiki.warframe.com/)
- 用途：官网导航的社区资料站入口。
- 来源/核验状态：官网导航确认入口，正文未核实；不得将 Wiki 内容称作开发商官方数据或已确认模拟数据源。
- 核验日期：2026-11-06（Asia/Shanghai）

### 灰机 Wiki 与腾讯国服官网

- URL：[灰机 Wiki](https://warframe.huijiwiki.com/wiki/Mainpage)、[灰机 API 内容页](https://warframe.huijiwiki.com/wiki/Api%E5%86%85%E5%AE%B9)、[腾讯国服官网](https://wf.qq.com/)
- 用途：中文资料、国服背景和版本对齐研究参考。
- 来源/核验状态：灰机页面本轮返回 403；搜索摘要提及 DE Public Export 与 WFCD/warframe-items，但不足以证明整体来源关系。腾讯官网可读，但不是已确认的结构化数据库/API；国服与国际服对齐待研究。
- 核验日期：2026-11-06（Asia/Shanghai）

### MCP 服务器与包（用户指定）


1. [mpeciakk/warframe-mcp](https://github.com/mpeciakk/warframe-mcp)：README 自述覆盖世界状态、物品、市场、掉落和配装，可本地 stdio 或自行启动 HTTP；用户评价“最全面”仅作用户评价，未作客观比较。
2. [pipeworx-io/mcp-warframe](https://github.com/pipeworx-io/mcp-warframe)：README 提供托管入口 [gateway.pipeworx.io/warframe/mcp](https://gateway.pipeworx.io/warframe/mcp) 和本地包 `@pipeworx/mcp-warframe`，涵盖世界状态、裂缝、入侵和物品搜索；用户评价“稳定易上手”，未连接或验证托管稳定性。
3. [bbauti/warframe-mcp](https://github.com/bbauti/warframe-mcp) 及 [@bbauti/warframe-mcp](https://www.npmjs.com/package/@bbauti/warframe-mcp)：README 展示物品、武器、MOD、市场、掉落、配装、Wiki 和世界状态工具；用户评价“查询丰富”，npm 正文 403；未确认公共托管入口。
4. [无作用域 warframe-mcp](https://www.npmjs.com/package/warframe-mcp)：用户不推荐，暂不选用；npm 正文 403，源码关联未确认，不猜测作者。
- 用途：探索 MCP 工具封装形式。
- 来源/核验状态：以上按用户偏好顺序记录；本轮仅文档核查，未连接或验证查询结果，作为辅助工具研究参考。
- 核验日期：2026-11-06（Asia/Shanghai）

### 其他规范候选

- 用途：用户提到的 Market OpenAPI 或非官方 Google Doc 规范候选。
- 来源/核验状态：用户未提供确切链接，待补充后核实；当前以 P0 的 Warframe.market 自有文档为准，不猜测 URL。
- 核验日期：2026-11-06（Asia/Shanghai）

### 用户指定视觉参考

- [Angelo Dock 参考视频](https://x.com/angeloldesigns/status/2100163930643423338/video/1)：用户指定的 Dock 视觉参考；用于观察干净、平整的底部导航方向，不复制其弹出窗口或实现细节。
- 核验日期：2026-11-08（Asia/Shanghai）
### Tauri 自定义窗口标题栏

- [Tauri Window Customization](https://v2.tauri.app/learn/window-customization/)：无系统装饰窗口、拖动区域和窗口权限配置参考；本项目仅独立实现标题栏与 Hugeicons 控件，不复制示例图标。
- 来源/核验状态：Tauri 官方文档，用户指定并于 2026-11-08 核验；原生按钮行为待本机 GUI 验收。
- 核验日期：2026-11-08（Asia/Shanghai）

### Digital Extremes weekly Riven PC aggregate

- URL：[weeklyRivensPC.json](https://www-static.warframe.com/repos/weeklyRivensPC.json)
- 用途：紫卡分析页官方周统计来源；`pop` 仅按官方发布语义展示为相对热度指数，`rerolled` 分组保留，不推导成交频率或估价。
- 来源/核验状态：Digital Extremes 静态数据入口；2026-11-08 已核验 JSON、Torid 聚合与 HTTP `Last-Modified` 元数据。实际数据含裸键、单双引号和 `null` 等非 strict JSON 情形，本项目使用受限安全 parser 处理，不执行 eval。字段语义同时参考 [Riven Trading Toolbuilders: Phase 1](https://forums.warframe.com/topic/1077490-riven-trading-toolbuilders-phase-1/)；`pop` 仅按官方发布语义展示为相对热度，不推导成交频率或估价。
- 核验日期：2026-11-08（Asia/Shanghai）

### Riven analysis rules and community reference provenance

- 本地研究记录：[riven-analysis-rules.md](docs/riven-analysis-rules.md)、[riven-analysis-market.md](docs/riven-analysis-market.md)
- 用途：记录 Warframe 44.1.3 / D1.2 规则边界、历史倍率模式、已核实的社区系数与属性别名，以及周统计/挂单比较的数据边界。
- 来源/核验状态：规则系数来自社区维护的 [calamity-inc/warframe-riven-info](https://github.com/calamity-inc/warframe-riven-info)；直接使用的原始资料为 [`riven_tags.json`](https://raw.githubusercontent.com/calamity-inc/warframe-riven-info/senpai/riven_tags.json) 与 [`RivenParser.js`](https://raw.githubusercontent.com/calamity-inc/warframe-riven-info/senpai/RivenParser.js)，不等同 Digital Extremes 官方规则接口。公式仅用于已核实常规词条，未知或缺失系数返回 unsupported，不推断新词条。
- 核验日期：2026-11-08（Asia/Shanghai）

### Warframe PC 44.1.3 官方补丁说明

- URL：[PC 44.1.3 Patch Notes](https://www.warframe.com/en/patch-notes/pc/44-1-3)
- 用途：核对现行 Riven disposition 统一为等效 D1.2 的版本边界。
- 来源/核验状态：Digital Extremes 官方补丁入口；2026-11-08（Asia/Shanghai）由 root 核验。官方来源，不等同社区系数实现。
- 核验日期：2026-11-08（Asia/Shanghai）

### Riven Disposition Retirement 官方开发者公告

- URL：[Dev Workshop](https://forums.warframe.com/topic/1529475-dev-workshop-ammo-efficiency-changes-riven-disposition-retirement/)
- 用途：记录 Riven disposition 退休及统一 D1.2 的规则变更背景。
- 来源/核验状态：Digital Extremes 官方论坛公告；2026-11-08（Asia/Shanghai）由 root 核验。
- 核验日期：2026-11-08（Asia/Shanghai）

### riven-roll 词条范围与分类参考

- URL：[riven-roll ranges](https://wxhn1225.github.io/riven-roll/#/ranges)、[源码仓库](https://github.com/wxhn1225/riven-roll)
- 用途：用户提供的第三方紫卡范围/分类展示与源码参考；页面展示步枪和近战的核心、元素、物理、派系、其他、熔接分组及 D1.2 范围布局。
- 来源/核验状态：第三方项目，非 Digital Extremes 官方入口。网站分类于 2026-11-08 由 CUA 核验；源码仓库 README 与 commit `986f37c7ae0057be4a4c3e8251cb03080507241a` 已由用户/ root 核验。源码初核范围与限制见 [riven-roll 参考记录](docs/riven-roll-reference.md)。README 所述游戏字典、Public Export 图片、洗卡 solver 与分布模拟仅作项目能力记录，未据此宣称本项目公式或数据已官方核实。
- 核验日期：2026-11-08（Asia/Shanghai）
