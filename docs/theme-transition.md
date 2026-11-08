# Theme transition study

核验日期：2026-11-07（Asia/Shanghai）。本文件记录可迁移的动效机制和本项目的独立 Web 实现边界；上游 Qt 资料仅作为一般机制参考。

## 从 Qt 资料学到的机制

已核对的 [FluentQtWidgets](https://github.com/txp666/FluentQtWidgets) `main` 分支提供了几个可复用的行为原则：

- `CardWidget.cpp` 的 `QVariantAnimation` 约 120ms 是卡片背景动画时长，不是全窗口主题切换时长。
- 卡片背景更新会从当前动画值或上一次结束值继续，先停止旧动画，再启动到新颜色，避免切换时跳回旧起点。
- `ThemeManager.setTheme` 统一应用主题并发出信号；可见控件立即刷新，隐藏控件不被强行重绘。
- `FluentWindow` 的 Windows Mica crossfade、兼容性刷新定时器和 `themeChangedFinished` 合并刷新属于系统背景处理，Web 实现不复用 Mica 机制。
- Gallery 入口只是调用 `setTheme`，没有圆形全屏遮罩或揭幕动画。

项目采用的是这些机制的独立映射：切换从当前已显示颜色连续过渡，重复切换合并为一个动画，主题入口共享同一状态；动画表现本身由 Web API 实现，不复制 Qt 源码、图标或依赖。

## Web 实现映射

`src/ui/themeTransition.ts` 负责注册现有根元素颜色 custom properties，并以根元素 `Element.animate` 播放 240ms、`cubic-bezier(.2,0,0,1)` 的颜色过渡。切换流程应满足：

1. 从当前根变量计算出的颜色开始，反向切换也从当前中间值继续。
2. 同一时刻只保留一个动画；新的主题请求合并并取消旧动画的竞争。
3. 主题变量挂在根元素，因此 Teleport 到 `body` 的设置/关于对话框也参与同一过渡。
4. 启动首帧直接应用主题，不播放入场动画。
5. 不依赖原生 Qt、系统 Mica 或网络接口；`prefers-reduced-motion: reduce`、无 `Element.animate`/无 `CSS.registerProperty` 时直接切换。
6. 主题状态仍由现有两个入口共享，隐藏的直接切换 API 不绕过统一逻辑。

## 验收要点

- 亮色和暗色两入口都使用同一动画逻辑，重复点击不会叠加多个动画。
- 连续快速反向切换没有明显闪烁、跳色或回到旧主题起点。
- 根内容、Teleport 对话框、侧栏和状态颜色同步过渡。
- 首次启动无动画；减少动态效果偏好下无动画。
- 宽、窄、矮窗口都保持可读性；键盘操作和焦点状态不因动画丢失。
- 需要单独进行浏览器/原生 GUI 视觉验收和构建验证；本说明不代替这些验证。

## 本轮验证记录

- `node vue-tsc --noEmit` 与 Vite production build 通过。
- `node --experimental-strip-types --test tests/themeTransition.test.mjs`：8/8 通过，覆盖正常/反向切换、同 tick 合并、减少动态效果、隐藏元素、动画故障、inline 清理和只取消自身动画。
- 在 `http://127.0.0.1:1421` 的真实浏览器中，顶栏和设置弹窗均通过统一动画切换；快速双击最终主题正确，Teleport 对话框背景和文字同步过渡。
- DOM 采到非端点中间值：切暗时 canvas `rgb(23,21,28)`、dialog `rgb(37,34,44)`、text `rgb(251,239,207)`；切亮时 canvas `rgb(251,251,253)`，证明颜色确实渐变。root style 始终为 `null`，无 inline 残留；console error logs 为空。
- 本轮未运行原生 Tauri GUI，不据此宣称全尺寸、键盘和安装验证完成；旧导出未因本轮主题验证重新打包。

## 资料边界

上游仓库许可为 GPL-3.0-or-later。本项目只借鉴一般动效机制，采用独立 TypeScript 实现，不拷贝 Qt 源码、图标或依赖。相关来源与 MDN Web API 资料见 [ExternalLinks.md](../ExternalLinks.md)。
