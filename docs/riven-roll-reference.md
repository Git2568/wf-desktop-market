# riven-roll 第三方源码参考记录

核验日期：2026-11-08（Asia/Shanghai）。本记录来自 root 的 CUA 实际阅读，项目仅作为第三方参考，不将其模型假设或公式称为官方规则，也未在本轮实现洗卡模块。

## 初核范围

已阅读仓库中的 [README.md](https://github.com/wxhn1225/riven-roll/blob/main/README.md)、[DESIGN.md](https://github.com/wxhn1225/riven-roll/blob/main/DESIGN.md)、[src/engine/data.ts](https://github.com/wxhn1225/riven-roll/blob/main/src/engine/data.ts)、[src/data/game.json](https://github.com/wxhn1225/riven-roll/blob/main/src/data/game.json)、[src/engine/formula.ts](https://github.com/wxhn1225/riven-roll/blob/main/src/engine/formula.ts) 前 120 行、[src/engine/solver.ts](https://github.com/wxhn1225/riven-roll/blob/main/src/engine/solver.ts) 前 120 行，以及 [`scripts/`](https://github.com/wxhn1225/riven-roll/tree/main/scripts) 目录。仓库 commit：[`986f37c7ae0057be4a4c3e8251cb03080507241a`](https://github.com/wxhn1225/riven-roll/commit/986f37c7ae0057be4a4c3e8251cb03080507241a)。链接使用仓库 main 展示路径，commit 独立记录核验版本。

`src/engine/data.ts` 直接导入随包 `src/data/game.json`，未见运行时下载词条请求。随包游戏版本为 intl `2026.10.27.13.12`、cn `2026.10.22.14.34`，包含 7 类、632 件武器。`classes.stats` 含 `base`、`buff/curse`、`spliced`、`compat`、`buffRestr/curseRestr`、`fmt`、`smaller`、`reverse`、`roundTo`、`roundingMode`；`classes.splices` 含 `a`、`b`、`result`。武器字段包含 `id`、`cls`、`tags`、`physical`、`sentinel`、`grimoire`、`variant`、`icon`。

README 的术语说明来自游戏字典；DESIGN 记录带 hash 的 Public Export 图像。`scripts/` 当前看到 `shot.mjs`、`smoke.mjs`、`ui-regression.py`，尚未找到可复现的游戏数据提取/更新脚本，因此不能声称自动官方同步。

## 公式与求解器边界

已核实的 `formula.ts` 片段：`rawValue = b × 10 × (rank + 1) × D × L × W`，其中 `W` 为 0.9–1.1；使用 binary32 并保持运算顺序，支持多种舍入模式。Ammo Efficiency 的基数为负，但展示时翻转符号。除此之外未读源码不作推断。

`solver.ts` 前言明确：四种布局均按 25%，词条无放回等概率，W 均匀分布，均为模型假设而非实测概率。锁定词条冻结布局并使赤毒成本翻倍；熔接免费锁定，并取两份材料中较高的 W。目标支持 OR 词条、多组目标、最小 W 或按布局阈值；状态按目标角色/阈值桶压缩，以 Dijkstra-like 扫描求期望赤毒或次数并保留新旧状态中更优者。README 称分布模拟可得到中位数/90%分位数。

未来若移植到 Rust，应沿用 reference/version/cache 边界，明确 tag/game_ref 映射与 pool 限制，并补充精确 rounding 测试。当前 Rust 已接入源作者完整求解/模拟/Planner 与比较逻辑，并完成 776 JS oracle 熔接范围对照；历史“unsupported”仅表示早期阶段。正式实现保留 gameRef/版本边界、2M 工作预算及上游概率模型假设；PlagueKeewar、Kripath PvP、`vinquibus_melee` 无法准确映射。
