# 外部资料索引

更新时间：2026-11-06（Asia/Shanghai）

本轮仅做在线文档、仓库和入口核查，没有下载依赖、业务请求、构建、运行或 WebSocket 联调。下列 P0–P4 是本项目的查阅与评估优先级，不等于数据官方权威；“官方权威”和“项目相关性”分别标注，需结合核验状态判断。

## 推荐查阅顺序

| 优先级 | 查阅重点 | 排序依据 |
| --- | --- | --- |
| P0 | 市场核心 API、WebSocket 与 Rust 候选库 | 直接影响市场接入和实时能力 |
| P1 | 官方/增强物品、翻译与模拟基础数据 | 支撑名称、字段、版本和模拟校验 |
| P2 | 社区 API、世界状态与解析工具 | 辅助数据和解析，不能替代市场挂单源 |
| P3 | 其他语言客户端和实现参考 | 比较封装与流程，当前未选作项目依赖 |
| P4 | 官网、Wiki、MCP 和探索性资料 | 补充背景，非客户端必需接入 |

## P0 市场核心

本层按项目相关性最高排序；市场文档属于 Warframe.market 生态来源，`wf-market` 是第三方 Rust 客户端库，不等同市场自有接口。

### Warframe.market API 概览

- URL：[https://docs.warframe.market/docs/api/overview/](https://docs.warframe.market/docs/api/overview/)
- 用途：HTTP/REST API 概览入口。
- 来源/核验状态：来自 Warframe.market 文档站导航，正文尚未核实；不能据此确认紫卡实时推送、鉴权或限流。
- 核验日期：2026-11-06（Asia/Shanghai）

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
- 来源/核验状态：仓库目录确认路径存在；单文件正文未能读取，REST 能力、WebSocket 推送、鉴权、限流和紫卡覆盖待复核。
- 核验日期：2026-11-06（Asia/Shanghai）

### wf-market Rust 客户端库

- 主仓库：[https://github.com/KibbeWater/wf-market](https://github.com/KibbeWater/wf-market)
- 辅助入口：[crates.io](https://crates.io/crates/wf-market)、[docs.rs](https://docs.rs/wf-market/latest/wf_market/)
- 用途：候选 Rust 市场客户端库及 API/WebSocket 封装参考。
- 来源/核验状态：第三方 Rust 客户端库；此前 README/API 文档层初核显示 `wf_market 0.3.3`、可选 `websocket` feature 和订单实时示例。crates.io 最新发布版本未独立核验，实际版本未锁定；源码、依赖构建、紫卡事件覆盖、协议和联调仍待核实，订单示例不等于紫卡实时挂单支持。
- 核验日期：2026-11-06（Asia/Shanghai）

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
