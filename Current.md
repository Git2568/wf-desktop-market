# 当前进度

更新时间：2026-11-06（Asia/Shanghai）

## 已完成

- Git 仓库已初始化。
- 当前分支为 `main`。
- 初始提交为 `5c07335 chore: initialize repository`。
- 根目录已有基础 `.gitignore`。
- `ExternalLinks.md` 已统一记录 `wf-market` 外部资料；此前在线文档初核显示 docs 文档为 `wf_market 0.3.3` 并提供可选 `websocket` 功能，仅代表文档层初核。
- 已创建 [ExternalLinks.md](ExternalLinks.md) 统一记录外部库、市场 API 文档、官方网站与资料站链接；`wfm-docs` 仓库及 API 总览路径已完成入口核实，单文件正文和业务能力仍待复核。
- 已收录 WFCD/warframe-items 主仓库、物品数据目录和 i18n 数据链接；README 初核确认包含 `zh`，翻译接入尚未完成，名称/字段/市场标识映射仍待样本核对。
- 已收录用户提供的第三方 Tampermonkey 翻译脚本源码入口；已尝试获取源码但本次工具未获取正文，功能范围仅来自用户描述，尚未安装、运行或接入翻译业务。
- 已收录 Riven-Finder，实现参考的 README 初核描述 C#/.NET 6.0、API 轮询和 Market 登录；源码及交易范围待验证，客户端尚未接入或运行。
- Market 账号登录与在线交易相关已记录为后续候选方向，不列入当期必做。
- 外部资料索引已按 P0–P4 重组并补充官方 Public Export、社区物品/翻译、世界状态解析、其他语言客户端及辅助资料；排序是本项目查阅/评估优先级，不等于官方权威或已决定采用。
- 用户已确认 Windows 优先及 Tauri 2、Vue 3 + TypeScript + Vite + Pinia、Rust + Tokio、独立 Rust 计算核心、SQLite、本地 IPC 分层与数据流架构，已记录至 `Program.md`。
- 需求已更新为比较两条市场信息接入路径：`wf-market` Rust 库或直接调用市场 API；目前仅纳入评估，未实现任一路径。
- 本轮架构方案已由用户确认、Luna 完成记录，主 Agent 已完成一致性与范围审阅。
- 文档基线已提交为 `c76d4eb docs: record project plan and reference baseline`，并保留初始提交 `5c07335 chore: initialize repository`。
- 用户已确认当前 UI 规范：WM 文字标识、可收拢/拉开的侧边栏导航、透明无边框设置触发器及“设置/关于”菜单、建议的亮色/暗色调色盘、窗口宽高拖动适配和简洁界面要求。
- 已将 Hugeicons 官方图标库与 Vue 3 官方集成方式列为强制 UI 规范；应用界面图标不得使用其他图标库、Emoji、Unicode 符号或临时手绘 SVG。本轮界面外壳与用户反馈由 Luna 实施，主 Agent 已完成源码与范围审阅。
- 本轮已记录收拢/展开控件的新位置与自适应要求：使用小型 Hugeicons 图标按钮，位于侧栏与主界面分隔线的纵向中点，并随侧栏边界和窗口高度变化保持垂直居中；Luna 已完成修改，主 Agent 源码与范围审阅通过，前端构建通过。依赖链接按现有锁文件恢复，版本未改。
- 上一条 UI 提交说明已英文化；已建立 Git 提交标题和正文必须使用英文、准确描述实际改动并记录必要验证的规范，后续提交继续遵守。

## 当前状态

- Vue 3、TypeScript、Vite、Pinia 前端脚手架和最小 Tauri 2 壳已落地。
- WM 双色 Logo、四模块导航与侧栏折叠、亮暗主题、底部菜单、设置/关于对话框和空状态已实现；UI 偏好当前暂存于 `localStorage`，SQLite 业务持久化尚未接入。
- 尚未接入 `wf-market` Rust 库。
- 尚未接入直接市场 API；两条接入路径均未实现。
- 尚未完成对应版本源码/API 审计、依赖构建、紫卡实际数据接口与 WebSocket 覆盖验证，也未核实完整的上游协议、授权、限流和数据规则。
- 实时紫卡市场、紫卡词条分析、紫卡洗卡指导、配装模拟和保存均未实现。
- 四个模块的业务功能与真实数据仍未实现；亮暗主题、窗口拖动适配和响应式布局仍待用户预览细审及 GUI 验收。
- 原 codeLuna 已验证 `pnpm typecheck`、`pnpm build`、`cargo check`、`cargo build` 成功；本轮紧凑布局与去暖黄修订后，refinement Luna 再次验证 `pnpm typecheck` 与 `pnpm build` 成功。
- Debug 可执行文件已生成于 `src-tauri/target/debug/wm-market.exe`；Vite `127.0.0.1:1420` 已由用户实际预览。

## 本次验证

- 检查范围为 `Program.md`、`Current.md`、`Agent.md`、`ExternalLinks.md`。
- 四个文件均通过 UTF-8 严格解码。
- 已检查行尾空格、制表符、文件末尾换行和一级标题；Markdown 基础格式检查通过。
- 已检查 Markdown 本地链接目标；HTTPS 链接不作为本地文件检查。
- 业务测试与真实数据联调尚未开展。
- 本轮仅记录已确认的 UI 设计和图标规范；未据此宣称真实市场数据、WebSocket、评分、洗卡或配装功能已实现。
- 主 Agent 已完成源码、Hugeicons 图标导出、色彩对比和静态布局审计；GUI 自动化连接与原生启动验收未成功，因此不宣称全尺寸、键盘及两主题交互 GUI 测试通过。

## 下一步

1. 按已定架构验证真实紫卡数据获取、实际可用的实时更新能力和市场适配通路。
2. 核对 P1 原始/增强物品数据与 P2 社区 API 的字段、语言、版本和市场标识边界。
3. 优先完成 UI 预览复核与后续 GUI 验收，再按已定架构接入市场真实数据。
4. 核实 `wf-market` 与直接市场 API 的实际能力后再确定接入路径，不擅自选定接口或数据源。

## 阻塞与待确认

当前没有代码层阻塞；两条市场信息接入路径均尚无实现，且尚无 `wf-market` 源码审计、直接 API 核验、联网联调或紫卡实时订阅验证。WebSocket、评分、洗卡指导、配装计算与 SQLite 业务持久化仍未实现。仍待确认市场接入方式与上游协议、热门成品口径、评分模型、洗卡规则、配装模拟规则、SQLite 数据结构/迁移和具体缓存策略。本轮实现与用户反馈由 Luna 实施，主 Agent 已完成源码与范围审阅。
