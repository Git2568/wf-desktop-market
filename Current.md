# 当前进度

更新时间：2026-11-09（Asia/Shanghai）

最新完成：洗卡 core/planner、service 与 IPC 已接入；776 熔接 oracle、真实分析 API 2+1/2+0、core/planner 定向测试均通过。
最新验收：root 已完成真实 Rust fixture Web/mock IPC；常规规划+保留新卡、免费熔接、1280×720 light/900×760 dark 均通过，console 无新增 warn/error。
边界：PlagueKeewar、Kripath PvP、`vinquibus_melee` 无准确映射；2M 工作预算与上游概率假设保留。native GUI/完整构建未做。

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
- （历史记录，2026-11-06）需求更新为比较两条市场信息接入路径；当时尚未实现任一路径。当前阶段已选择直接 API 进行试验接入。
- 本轮架构方案已由用户确认、Luna 完成记录，主 Agent 已完成一致性与范围审阅。
- 文档基线已提交为 `c76d4eb docs: record project plan and reference baseline`，并保留初始提交 `5c07335 chore: initialize repository`。
- 用户已确认当前 UI 规范：WM 文字标识、可收拢/拉开的侧边栏导航、透明无边框设置触发器及“设置/关于”菜单、建议的亮色/暗色调色盘、窗口宽高拖动适配和简洁界面要求。
- 已将 Hugeicons 官方图标库与 Vue 3 官方集成方式列为强制 UI 规范；应用界面图标不得使用其他图标库、Emoji、Unicode 符号或临时手绘 SVG。本轮界面外壳与用户反馈由 Luna 实施，主 Agent 已完成源码与范围审阅。
- 本轮已记录收拢/展开控件的新位置与自适应要求：使用小型 Hugeicons 图标按钮，位于侧栏与主界面分隔线的纵向中点，并随侧栏边界和窗口高度变化保持垂直居中；Luna 已完成修改，主 Agent 源码与范围审阅通过，前端构建通过。依赖链接按现有锁文件恢复，版本未改。
- 本轮追加收拢侧栏按钮样式：按钮保持 30px 尺寸与 Hugeicons 图标，改为无边框、透明背景、无阴影，悬停仅改变图标颜色；已运行 `pnpm build` 与 `git diff --check`，未做 native GUI、打包或网络测试。
- 本轮微调高级筛选弹窗 CSS 高度，为负面下拉留空间并受视口高度限制；未进行 GUI 验收。
- 上一条 UI 提交说明已英文化；已建立 Git 提交标题和正文必须使用英文、准确描述实际改动并记录必要验证的规范，后续提交继续遵守。

## 当前状态

- 用户已认可当前 UI 大体样式；本阶段开始真实接口请求、市场信息获取与显示测试。
- 主题过渡独立 Web 实现已完成源码与测试验收：前端构建通过，`themeTransition.test.mjs` 8/8 通过；真实浏览器已验证顶栏与 Teleport 设置弹窗统一渐变、快速反向切换和无 inline 残留。原生 Tauri GUI、全尺寸/键盘与安装验证仍待完成。
- 已实现直接 Warframe.market API 的首个 Rust/Tauri/Vue 纵向通路，使用武器/词条目录和按武器挂单搜索，显示中文词条、原始数值单位、报价类别、卖家状态、更新时间和本批数量。选择直接 API 是因为已审计的 `wf-market` 0.3.3 未提供按武器紫卡挂单搜索封装；紫卡 WebSocket 尚未验证。

- Vue 3、TypeScript、Vite、Pinia 前端脚手架和最小 Tauri 2 壳已落地。
- WM 双色 Logo、四模块导航与侧栏折叠、亮暗主题、底部菜单、设置/关于对话框和空状态已实现；UI 偏好当前暂存于 `localStorage`，SQLite 配装业务持久化尚未接入（目录缓存已接入）。
- 尚未接入 `wf-market` Rust 库。
- `wf-market` Rust 库未用于挂单搜索；已完成其源码能力核对并选择直接 API 作为当前试验通路。
- （历史首次通路）Rust/Tauri IPC、前端显示和 Torid 100 条实际挂单查询已通过；本轮筛选通路在此基础上继续扩展，完整视觉 GUI 验收仍未完成。
- 本轮已实现市场页基本筛选和高级筛选：启动只加载目录，用户点击“查询”后才请求挂单；简单/高级筛选共享同一份条件，支持武器、正负面词条、价格排序、极性、MR 与 Re-rolls 范围。
- 本轮 5 项 UI 改动已实现并完成源码审查：正面绿色/负面红色且不显示方向文字、表头改为用户状态、Polarity 替代武器副行、末列操作菜单含 3 项禁用占位，并修复 SearchSelect 内框与焦点样式。菜单实际交互及颜色/双边框视觉复核仍未完成。
- 本轮前端构建已通过：本地 `node node_modules/vue-tsc/bin/vue-tsc.js --noEmit` 与 `node node_modules/vite/bin/vite.js build` 均成功，极性 PNG 作为本地资源随 `dist` 构建加载；`git diff --check` 通过。GUI 尚未实际验收，不宣称视觉或交互验收完成。
- 本轮高级筛选范围 UI 已更新为紫卡等级/洗卡次数分组、同一行最小/最大值和 ASCII 数字长度约束；待本地构建与 GUI 验证。
- （历史布局）本轮高级筛选布局曾为整行名称/正面特性/范围/价格排序，负面特性与极性同排；现已由下方最新布局替代。
- 本轮前后端已完成并通过主流程验证：高级筛选使用最高段位等级与已确认的 `maxed`（满级 8 级）强化枚举，列表副行显示极性、段位、强化、洗卡次数；市场模块三页规则已记录，Dock 已冻结。前端最终构建通过；Rust 7 个 offline 测试与独立 `live_filtered_torid_smoke` 通过（22 条、1130ms，包含 rank8、MR<=16、正负词条、极性、洗卡次数和排序断言）。
- 本轮已完成静态目录缓存实现：首次从随包真实 seed 导入 420 武器/50 词条，之后从 `app_local_data_dir/market-catalog.sqlite3` 本地读取；共享 Pinia 目录状态与设置页手动更新/force 入口已接入，启动不自动联网更新。SQLite 保留原始两目录 JSON、SHA-256、collections 版本与 nullable `game_version`；当前/previous 快照事务切换已通过离线回归验证。
- 目录缓存最终证据：Rust fmt/offline 检查 20 passed/0 failed/2 ignored，独立 live probe 1 passed；前端 5 项 store 回归、typecheck/Vite build 与 native build 通过。live probe 首装本地 53ms、重开 24ms、搜索 6348ms、重开至结果 6373ms、总计 6426ms，目录 420/50、挂单 100，request gate 未变化。优化前历史冷目录 7144ms、目录至首次结果 8975ms；旧挂单 1830ms 不作为当前网络基准。原生 GUI 仅完成可访问树读取，点击/截图环境故障，设置交互与安装包未验收。
- 本轮高级筛选布局已调整为名称/极性同排、正负特性整行、强化/最高段位同排、洗卡次数与价格排序整行；仅改变布局，typecheck/build/diffcheck 已通过，本轮未进行 GUI 验收。
- 本轮样式已调整：禁用按钮不再显示等待光标，主滚动区、设置/筛选弹窗和下拉列表统一主题滚动条；待本轮浏览器预览验收。
- 本轮浏览器验收已确认：初始查询按钮 disabled 时为 `cursor:not-allowed` 且无 hover filter；主滚动区与高级筛选弹窗 Chromium 滚动条宽 8px、圆角 999px、轨道透明，亮暗主题滑块均随 `text-muted` token 派生，console errors 为空。Web 非 Tauri 无真实长行情，本轮不宣称原生 GUI 或所有尺寸验收。
- 本轮 UI 版本已完成脚本打包：script exit 0，beforeBuild 的 vue-tsc/Vite 通过，Release 编译约 17.79 秒，NSIS 安装包 2.18 MiB；导出目录为 `local-dist/0.1.0-20261107-125722-599`，exe 10,592,768 bytes、installer 2,286,717 bytes。两份 SHA256 已核对，exe 为 x64 PE、Windows GUI subsystem 2；脚本与导出仍由 Git local exclude 排除，安装程序尚未实际安装。
- （历史记录）词条分析、洗卡指导、配装模拟和保存仍未实现；实时紫卡市场已进入接口试验阶段。
- [历史阶段记录，已由 2026-11-09 状态替代] 当前首版已接通常规紫卡词条分析与市场参考请求；洗卡指导、配装模拟/保存、评分和图片/名称识别仍待实现。未知或新增系数返回 `unsupported`，不伪造结果。
- 前端 `pnpm typecheck`、`pnpm build` 通过；筛选版本 Rust 测试为 7 个离线测试通过，另有 1 个独立真实筛选 smoke（Torid 10 条、约 1044ms）通过。此前 4 个测试与 Torid 100 条 smoke 属于历史首次通路验证；其余 Rust 构建证据保持记录。
- `src-tauri/target/debug/wm-market.exe` 已生成；补充 Tauri Windows manifest 与 ICO 后，WM Market 桌面窗口已成功启动。
- 已新增不纳入 Git 管理的 `build-local.ps1` 本地打包脚本；`build-local.ps1` 与 `local-dist/` 已由 Git `info/exclude` 排除。Windows PowerShell 5.1 通过 `powershell.exe -NoProfile -ExecutionPolicy Bypass -File build-local.ps1 -Offline` 实际成功导出 Release exe 和 NSIS 安装包；仅本进程绕过执行策略，未修改全局策略。CLI 版本校验已通过将 `@tauri-apps/api` 精确固定为 2.11.1，与 Rust Tauri 2.11 对齐。（历史首次打包）beforeBuild 的生产前端 typecheck/build 通过，Release 编译约 1 分 09 秒，NSIS 安装包约 2.15 MiB，并已复制到独立导出目录；安装包尚未在用户机器上实际安装测试。
- 已将本地打包脚本改为仅依赖 Node/Cargo/Git 和已安装项目依赖，移除全局 pnpm 前提；在当前进程隔离 pnpm shim 后离线打包成功，导出目录为 `local-dist/0.1.0-20261107-133303-182`，exe 10,592,768 bytes、installer 2,286,669 bytes，SHA256 已核对，exe 为 x64 PE、Windows GUI subsystem 2。该证据仅覆盖脚本打包链路，不代表 GUI 或安装实测通过。
- 已将 Release Windows GUI 子系统属性移至实际 bin 入口 `src-tauri/src/main.rs`；最终重新导出的 exe 已静态核对为 x64、Windows GUI 子系统 2，SHA256SUMS 校验通过。安装包尚未在用户机器上实际安装测试。

## 本次验证

- 检查范围为 `Program.md`、`Current.md`、`Agent.md`、`ExternalLinks.md`。
- 四个文件均通过 UTF-8 严格解码。
- 已检查行尾空格、制表符、文件末尾换行和一级标题；Markdown 基础格式检查通过。
- 已检查 Markdown 本地链接目标；HTTPS 链接不作为本地文件检查。
- 已完成匿名真实请求样本：v2 武器目录 420 项、词条目录 50 项、Torid 挂单批次 100 项；无武器条件返回 HTTP 400。另有 soma 小样本 100 项可见开放唯一记录，均为 PC，其中 44 项含负面词条；样本不代表市场总量。
- 已完成此前筛选版本的解析、过滤、去重、单位和报价类别测试；本轮浏览器验收确认最高段位单值、强化不限/满级可选，选择最高16与满级后切换 WM 用户/我的上架再回购买页仍保持筛选，空页仅显示暂无内容且无登录控件；Dock 约 78% 透明、blur16px，dockTop783/contentBottom771 不遮盖，console errors 为空。原生 Tauri GUI 与安装程序本轮未实测，不宣称完整 GUI 验收。未实现紫卡 WebSocket、自动轮询、评分、洗卡或配装功能。

## 下一步

1. 按后续变更影响范围或用户明确要求，定向安排筛选/设置 GUI 与安装程序验证；宽窄矮窗口、亮暗主题和键盘路径的既有待验收项不是每轮强制检查。
2. 继续核实紫卡实时更新能力、限流和 WebSocket 边界。

2026-11-08 估价状态更新（历史阶段记录，洗卡待办已由 2026-11-09 状态替代）：用户已授权并实现 `sample-heuristic-v1` 首版，具体模型见 [riven-valuation.md](docs/riven-valuation.md)。

## 阻塞与待确认

当前主要待完成项是本轮筛选 GUI 与设置交互实际验收、安装程序实际安装验证和紫卡 WebSocket 能力核实；这些既有待办不构成每轮强制检查，也不自动阻塞无关改动。Release/NSIS 导出与接口验证已完成，但完整 GUI 与安装包实际安装测试尚未通过，不能宣称完成。Retry-After 当前解析数字秒，HTTP-date 尚未支持；真实 429/509 未刻意触发。[历史阶段记录，已由 2026-11-09 状态替代] 评分模型、洗卡规则、配装计算与配装 SQLite 结构仍待设计；市场静态目录缓存已实现；缓存目录不等于完整游戏数据。

本轮生效的验证政策：除非用户明确要求完整构建/打包/导出，否则按变更影响范围选择轻量定向检查；历史验证记录保留，不自动扩展为全应用回归。

本轮底部 Dock 已定向验收：1280x720 下主体 288x36、bottomGap=0、底部圆角 0、四边无边框，三列各 96px，磨砂 blur18/saturate1.15；按钮覆盖槽位 95%，图标文字间距 2px，前两槽有细分隔线。靠边点击可切换 WM 用户页，`aria-current` 正确跟随；未选外凸/选中内凹、亮暗主题和 console errors 均已核对，内容与 footer 无重叠。已恢复购买页与亮色；本轮仅执行 typecheck/diffcheck 与 Dock 定向浏览器验收，不宣称原生、全功能或完整构建。

2026-11-08 用户根据 Angelo Dock 参考视频纠正视觉方向：当前 Dock 内部按钮已改为透明平整，中性灰文字，选中仅保留极淡中性状态底色；移除项级背景、投影、凹凸和紫色块，玻璃材质与柔和阴影集中在整块 Dock 底板。浏览器验收确认内部默认背景透明、`box-shadow: none`、`border: 0`，选中仅中性灰 6% 且无阴影；整体高度 36px、下沿与窗口底部间距 0。切至 WM 用户后页面及选中项对应，再切回紫卡购买正常；控制台 `error=[]`。已恢复购买页与亮色并重置临时 viewport。旧的未选外凸/选中内凹记录保留为历史验收，不代表当前样式。

本轮紫卡词条分析页已完成定向浏览器验收：1280x900 亮色左右布局、760x720 暗色单列均无横向溢出；临时入口加载已有真实 420 武器/50 词条 seed，不代表 Tauri/API 验收。初始“开始分析”禁用；Torid/托里德中英文搜索、暴击率 123.4% 与暴击伤害 98.2% 后按钮启用，点击仅显示“分析功能暂未开放”。3 正 1 负时两个加号禁用；新增空行或输入 `abc` 时按钮禁用；删除负面和第三正面后恢复可用，减至 1 正面后再次禁用；重复 Critical Chance 被过滤；卡片三正绿/一负红与有符号小数 -12.3% 同步；模块切换保留草稿（AX 复核 123.4），console error=[]。已恢复亮色并重置 viewport。图片/名称识别、真实分析、原生打包未验收，未运行完整构建。本轮 `vue-tsc --noEmit` 与 `git diff --check` 已通过。

本轮自定义标题栏已完成浏览器定向验收：1280x820 亮色展开、760x520 暗色折叠均无横向溢出；Sidebar top=0，标题栏 top=0/height=36，右侧控件贴右；旧 `.topbar`/`.topbar-context` 数量为 0。四个窗口按钮均透明、无边框且不在拖动区内；主题切换、侧栏折叠/展开正常，Dock bottomGap=0，console error=[]。已恢复亮色展开与购买页并重置 viewport。浏览器中原生窗口按钮保持禁用，尚未实测原生拖动、最小化、最大化和关闭；不宣称原生窗口动作验收或完整构建。本轮源码修复后的 `vue-tsc`、`cargo check --offline`、`git diff --check` 均已通过。

本轮全局与组件 scoped 紧凑样式已完成定向浏览器验收：1280x820 亮色分析页与高级筛选/负面下拉、760x520 暗色高级筛选/设置菜单和弹窗均无横向溢出；分析 form gap8，数字输入与 SearchSelect 均 32px/12px 字号、r6，卡片 r10 无阴影，SearchSelect 保持外 1px/内 0px 单边框。高级筛选自然高度 499px/r12/gap10，展开负面列表仍在弹窗边界；矮窗弹窗 top16/bottom504/maxheight488，可滚动；设置弹窗 r12/body16，菜单开关正常。真实 Torid/暴击率输入 123.4 后预览同步，console error=[]；已恢复亮色展开侧栏并重置 viewport。实际行情表格与上架操作仅做源码样式审计，不宣称真实数据或原生全功能验收，未运行完整 build/cargo。既有本轮 `vue-tsc --noEmit`/`git diff --check` 已通过，后续纯 CSS 修正仅 `git diff --check` 通过。

本轮前端源码已加入共享 WeaponSelect：简单筛选、高级筛选和分析页均在下拉弹层顶部提供“武器类型”筛选，候选按英文名称 A–Z 排序，未知类型保留原值标签；SearchSelect header 与 listbox 分离并修正内部焦点切换关闭逻辑。简单/高级排序增加正面词条升降序选项，分析预览内容居中。`vue-tsc --noEmit` 与 `git diff --check` 已通过；待 root 进行浏览器验收，不宣称 GUI 验收完成。

词条排序最终语义已确认：选择正面词条升序/降序但未选正面词条时，查询前通过现有错误通路提示“请至少选择一个正面词条后按词条排序。”；已查询结果仅在 snapshot 使用词条排序时显示“词条排序限当前批次”。排序为当前批次内按词条数值多键比较，多选按用户选择顺序依次比较，不宣称全市场排序算法。

本轮前端与排序功能已完成 root 定向验收：1280x820 亮色主简单/高级/分析页、760x520 暗色简单布局均无横向溢出；简单排序字段 126px，宽窗前三列约 286.66px 等分剩余，窄窗严格 245px 两列且排序仍 126px。简单/高级四项排序同步（含 `positive_attr_desc`），类型 header 独立于 listbox；ZAW 11 项按英文 A–Z（Balla→Sepfahn），关键词 Torid+ZAW 无匹配、清词后恢复 11 项，Kitgun 6 项按英文 A–Z（Catchmoon→Vermisplicer）。选择宝拉后切 Kitgun 保留中文已选值，header ArrowUp 切类型不关闭面板；高级类型 header Escape 只关闭内层且 dialog 保持，`aria-expanded=false`；listbox 主题滚动条 8px；分析武器中英名称及正负行居中；console error=[]。已恢复亮色展开购买页并重置 viewport。临时入口使用真实 420/50 seed，不等于 Tauri 真实挂单 GUI 验收。

本轮最终轻检查证据：`cargo check --offline --tests/fmtcheck` 通过，定向 `cargo test --offline --lib positive_sort` 1 passed，前端 `vue-tsc` 与 `git diff --check` 通过；未运行完整 build/全套回归。用户已批准当前批次内按选择顺序多键排序；API 9 次实测详见 docs，不夸大服务端排序支持。上述前端待验收项已改为完成定向验收。

本轮已完成真正紫卡分析前端首版源码接入：新增 `src/api/analysis.ts` 固定 snake_case DTO 与四组 Tauri IPC；市场 DTO 增加可选 `group`、`disposition`、`game_ref`、`positive_is_negative` 字段。分析页新增强化等级、现行/历史规则、历史倍率和高价参考挂单数参数，保留 2–3 正面/0–1 负面及有限数校验；词条范围、官方周热度、高价挂单匹配和估价待设计分区独立展示，分析与市场请求并行、分别 loading/error、带 generation 防旧结果覆盖。普通浏览器只提示桌面分析，不伪造结果；设置页仅在打开时读取分析数据本地状态，并与目录更新独立保留错误。本轮已完成定向检查与 mock 展示验收；来源已汇总至 `ExternalLinks.md`，native GUI 与完整功能仍待验收。

分析当前状态修订：已接通已核实常规词条的本地范围与市场参考请求；未知或新增系数返回 `unsupported`，不伪造结果。历史“分析功能暂未开放”浏览器记录保留为当时事实，不代表当前源码状态；评分、战斗模拟与估价公式仍未实现。分析规则更新仅表示校验/重新导入随包已核实表，周数据按手动刷新联网更新，不表示自动获取全部新游戏规则。

2026-11-08 当前首版与定向验收补充：Rust 新增 `riven_analysis`、`analysis_cache`、`analysis_service` 三模块，定向测试共 22 passed，`cargo check --offline --tests` 通过。本地 165 条紫卡基础系数与原始来源逐项比对一致；默认国际服现行规则为 `D=1.2`，legacy 保留 `0.5–1.55`，未核实新增词条返回 `unsupported`。周统计和挂单仅在用户点击分析时按 TTL 获取；设置额外支持周统计刷新；规则表随包校验导入；启动不联网。

历史记录：首版范围分析阶段。root 的 CUA 使用真实 420 武器/50 词条 seed、实际 Rust Torid 范围 JSON 与 mock 市场 IPC 完成 1280x720 亮色及 760x720 暗色单列（492px）验收，无横向溢出：CC 180 范围 160.4–196、位置 55.1；CD 145 范围 128.3–156.8、位置 58.6；`console error=[]`。这是 mock IPC 展示验收，不等同真实 native IPC 或全功能验收。stale 数据保留警告；abc 输入禁用并可清结果；Torid 历史 D 自动读取 1.3。卖家高价样本默认最多 N=20，是当前批次一口价样本，不是成交价或全球 top；DE 周统计 popularity 是相对热度，不反推成交频率；估价公式待设计。未完成完整 build/打包、WFM 实网挂单新增验收。

最终恢复验收：root 已恢复亮色主题、展开侧栏并 reset viewport；重置后默认宽 804px 仍无横向溢出。最后热更新无可见反斜杠，stale 文案与挂单获取时间显示正确；控制台结论为最终稳定版本无新增错误。临时热更新期间出现的格式错误已纠正；本记录仍是 mock/Web 展示验收，不代表真实 native GUI。

2026-11-08 范围提示与分析页定向验收：root 在 mock IPC 下验证 1280x900 亮色左栏 480px、数值输入 145px、上传/form 居中；900x760 暗色单列宽 632px 无横向溢出，console 无新增错误。空输入显示真实本地范围；槽位增删批量重算，单改词条仅单项请求，value 改动不请求且保留 220；稳定 id/context 与 row generation 防旧响应。Torid 2+0 CC/CD、加第三正、加负槽、仅 fire 负、替换为 multishot、再批量更新等场景均符合预期；计数器 0–8、restore CD 后为 11，正式分析不增加 range IPC。背景 250x325.33 的 7 份本地 PNG 资源对应 9 个 img 实例（SideLight 与 Corner 各复用一次），均加载成功，社区镜像来源已记录。估价 80–140 中位 110 来自 Rust 合成 fixture（100/110/120），非实网成交；详情展开 3 references、0% factors/100% coverage 正常。最终恢复亮色、展开侧栏并 reset viewport 后默认宽 1106px 无横向溢出，上传按钮仍精准居中，稳定 console 无新增错误。以上为 mock IPC/Web 定向展示，不代表 native 实网、完整 build 或打包。已有估价 16 tests、range 一致性 5 tests、core 13 tests 通过；root 已完成 cargo check --offline --tests、vue-tsc、fmtcheck、diffcheck 轻检查。

2026-11-08 紫卡预览控件与范围 placeholder 小修：分析页上传按钮旁新增 Hugeicons `EyeIcon`/`ViewOffIcon` 独立切换按钮，默认显示紫卡预览，使用 `ref(true)` 与 `v-show` 保留表单、范围和结果状态；按钮提供动态 aria-label/title、aria-pressed、aria-controls 与隐藏状态文案。词条数值范围 placeholder 使用全局两主题中性灰 `#737373`（亮色）/`#a3a3a3`（暗色），不改全局 `--text-muted`。root 已完成定向 Web/mock 验收：Torid 2+0 下亮/暗 placeholder 分别为 rgb(115,115,115)/rgb(163,163,163)，点击隐藏后按 Enter 恢复，原值 180 与范围提示 160.4–196/128.3–156.8 保持不变，范围 IPC 保持 2 次；按钮透明且无边框，无新增 console error。已恢复亮色并关闭临时 tab；1442 备用预览已清理，1441 来源未知的现有服务未触碰。以上不代表 native GUI 或完整构建。

2026-11-08 词条分类元数据已落地：新增 `src/data/riven-attribute-presentation.json`，按现有 50 条 seed slug 去重分为核心、元素、物理、派系、其他、熔接六组；近战新增四项位于熔接末尾，common 近战八项保留在其他。该配置随包作为本地展示元数据，不新增 SQL 迁移；SQLite 仍为整包目录缓存。正面/负面下拉的分类列已加宽加高，仅本页启用；root 定向验收通过，详见下一段；本轮未运行完整构建。

2026-11-08 词条分类已完成 root 定向 Web/mock IPC 验收：1442 现有 420/50 seed 在 1280x720 亮色与 900x760 暗色下，正面六组按用户顺序展示，每组三列约 210px，面板宽约 660px；当前视口可用高度约 344/364px，列表自适应，未将 480px 宣称为实测值。边缘 12px 无横向溢出，列表距输入 6px；Weak Point 搜索仅显示熔接 2 项，收起武器搜索仅显示 1 项，短面板约 95px。长中文换行正常，负面保留 `positiveOnly` 过滤，已选暴击率后负面不重复；Esc、ArrowDown+Enter 键盘路径正常。最终 26 次键盘下移后 active 弱点伤害仍在列表内可见，listScroll=277、外层 pageScroll=255 不变；修复测量取消 max-height 导致内部滚动重置问题。console 无新增错误。已恢复亮色、展开侧栏并 reset viewport；这是 Web/mock 验收，不代表 native 或完整构建。`vue-tsc` 与 `git diff --check` 已通过，本轮未运行 cargo。

2026-11-09：熔接范围完成 776 JS oracle 对照；真实分析 API 2+1/2+0 测试通过；core/planner 定向 12 项通过（含 9 组上游案例、4 边界、7 planner）。已接入完整求解/模拟/方案/比较实现，共用 gameRef/数据版本。PlagueKeewar、Kripath PvP、`vinquibus_melee` 无法准确映射；2M 复杂度预算及上游概率假设保留。网页仅真实 Rust fixture mock IPC，UI/native 仍待 root 验收，未运行完整构建。
# 2026-11-09 最新状态

洗卡指导 backend core/planner 已由源作者完整实现并接入 Rust/Tauri，完成 776 项熔接 oracle、真实分析 API 2+1/2+0 与 12 项定向测试。root 已完成真实 Rust fixture Web/mock IPC 验收：常规规划+保留新卡、免费熔接不扣次数赤毒，1280x720 light/900x760 dark 无横溢，最终 console 无新增 warn/error；native 与完整构建未做。以下早期“洗卡待实现/待设计”均为历史阶段记录。
