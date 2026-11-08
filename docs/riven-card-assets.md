# Riven card asset research

核验日期：2026-11-08（Asia/Shanghai）

## 结论

已找到并保存一组可用于“空白紫卡背景”组合的原始 PNG 组件。来源是 WFCD 的社区 npm 包 `@wfcd/mod-generator`（1.0.10），包内随 `genesis-assets` 发布的 `modFrames` 图片；不是 Digital Extremes 官方托管 URL，也不是社区重绘模板。不要把 WFCD 或 npm 包称作 DE 官方来源。组件仅保留图层本身，未抹字、未从截图编辑或 AI 生成。

`mod-generator` 的公开 README 说明其用途是使用 `warframe-items` 组装完整/折叠 Mod 图片并支持 Rivens；发布代码中的 `getBackground("Omega")` 明确选择 `LegendaryBackground.png`、`RivenTopRightBacker.png`、`RivenLowerTab.png`，`getFrame("Omega")` 选择 Omega 框架组件。由此可验证这些文件是该社区生成器所依赖的游戏 Mod 图层候选，但不能单独证明每个字节由 DE 官方 Public Export 直接托管。

来源入口：

- https://github.com/WFCD/mod-generator
- https://www.npmjs.com/package/@wfcd/mod-generator
- 下载包（核验时使用）：https://registry.npmjs.org/@wfcd/mod-generator/-/mod-generator-1.0.10.tgz
- WFCD 组织自述其与 Digital Extremes 无隶属关系：https://github.com/WFCD

## 已保存组件

| 本地文件 | 类型/尺寸 | SHA-256 | 来源包路径 | 用途/证据 |
|---|---:|---|---|---|
| `src/assets/riven/LegendaryBackground.png` | PNG 256x512 | `B48423D7B09951F63094037067821292B72D215DC109DA120AA82707EAE608C4` | `package/dist/genesis-assets/modFrames/LegendaryBackground.png` | 生成器将 Omega/Riven 背景指定为该文件；检视为无文字、无数值的暗纹背景 |
| `src/assets/riven/OmegaFrameTop.png` | PNG 281x117 | `D53F27961B4A173622B4F1A193B4576E3B4114C45EBC94F70C50202C913E3643` | `package/dist/genesis-assets/modFrames/OmegaFrameTop.png` | Omega 顶部框线图层；无文字、无数值 |
| `src/assets/riven/OmegaFrameBottom.png` | PNG 292x120 | `122FD6DD8D45CECD8706C70F2721AFDFBA8DAFD8CEDFEDF14AE46167E69B536E` | `package/dist/genesis-assets/modFrames/OmegaFrameBottom.png` | Omega 底部框线图层；无文字、无数值 |
| `src/assets/riven/OmegaSideLight.png` | PNG 16x256 | `378734AAF9A45040372601676E07730C5D48F51977F7B0348857DC66255C4240` | `package/dist/genesis-assets/modFrames/OmegaSideLight.png` | Omega 侧光图层；无文字、无数值 |
| `src/assets/riven/OmegaCornerLights.png` | PNG 64x64 | `2F8C92ADC3D4FE0EF828F3207BDC3D694FF88C8B1BC54DBE20E30E0D6A17633E` | `package/dist/genesis-assets/modFrames/OmegaCornerLights.png` | Omega 角光图层；无文字、无数值 |
| `src/assets/riven/RivenTopRightBacker.png` | PNG 45x26 | `B52DC8BB3EFF7A014FA81BE61498C217BB8E6D2BEFBF44FCA01F8F171F75D38A` | `package/dist/genesis-assets/modFrames/RivenTopRightBacker.png` | 生成器的 Riven 专用右上背板；无文字、无数值 |
| `src/assets/riven/RivenLowerTab.png` | PNG 210x26 | `A2D119019518725FC7E04D1F6D4F21EB64889F905DE434B84C448FA87084CA35` | `package/dist/genesis-assets/modFrames/RivenLowerTab.png` | 生成器的 Riven 专用底部标签图层；无文字、无数值 |

## 限制与使用边界

- 本地检查确认 `LegendaryBackground.png` 是纯背景纹理，没有可见文字或数值；其余保存文件是透明/装饰框图层，不能单独当完整卡片。
- 本轮没有找到可证明由 Digital Extremes 官方 Public Export 直接托管的“完整空白 Riven 卡框”URL；不得把 `browse.wf`、WFCD、npm 或 `mod-generator` 称为官方托管。
- 这些文件来自社区镜像发布包；组件可作为原始游戏资产候选，版权/再分发许可仍需项目方另行确认。包自身含 MIT 许可证，但不等同于游戏资产权利授权。

## `mod-generator` expanded Omega 布局证据

已从发布包 `dist/generator.mjs` 读取布局代码（无新增网络请求）。Riven 的 `tier` 是 `Omega`，背景仍取 `LegendaryBackground.png`。生成器的 expanded 中间画布为 **292x512**，背景原图为 **256x512**，因此 `centerX = (292-256)/2 = 18`、`centerY = 0`；背景不缩放。

- `LegendaryBackground.png`：绘制到 `(18, 0)`，原尺寸 256x512。
- `OmegaFrameTop.png`：原尺寸 281x117，大于背景宽度时使用 `widthDiff = 281-256-48 = -23`，绘制 x=`11.5`、y=`background.height*0.14 = 71.68`，不缩放。
- `OmegaFrameBottom.png`：原尺寸 292x120，大于背景宽度时使用 `widthDiff = 292-256-40 = -4`，绘制 x=`2`、y=`background.height*0.65 = 332.8`，不缩放。
- `OmegaSideLight.png`：先在 256x512 的 `backgroundImage` 内部绘制右侧原图到 x=`256*0.93 = 238.08`、y=`512*0.21 = 107.52`，再水平翻转副本到 x=0；随后整个 background group 在 expanded 画布上以 x=18 绘制，所以最终 x 为 **256.08** 和 **18**，不缩放。
- `RivenTopRightBacker.png`：先在 background group 内绘制到 x=`256*0.8 = 204.8`、y=`512*0.2 = 102.4`，最终 x 为 **222.8**、y=102.4；其内部 `backerImage` 可能再绘制极性/段位信息，空白背景实现应跳过该动态内容。
- `RivenLowerTab.png`：先在 background group 内绘制到 x=`256*0.09 = 23.04`、y=`512-120-16 = 376`，最终 x 为 **41.04**、y=376；不缩放。
- `bottomImage` 的 Riven 角光若启用，组件内部画布为 292x120，并且整个 bottom group 直接在 expanded 画布以 x=2、y=332.8 绘制（不是 background group，因此不再加 18）：右角最终 x=`2 + 292*0.73 = 215.16`、y=`332.8 + 120*0.27 = 365.2`；左角翻转副本最终 x=`2 + 292*0.04 = 13.68`、y=365.2。随后还会绘制等级槽位。纯空白背景不应自动加入等级槽位。

最后生成器创建 **292x380** 输出画布，将完整 292x512 中间画布绘制到 y=`(380-512)/2 = -66`，即垂直裁切源画布 y=66..446。以上坐标说明为什么不能用 CSS `cover` 把各图层拉伸填满容器：顶部/底部图层是固定像素装饰，且背景有透明/留白区域；应按原尺寸和这些偏移组合。

注意：`OmegaFrameTop` 的 x=11.5 不是几何居中值 5.5。发布代码在 `top.width > background.width` 分支中使用 `widthDiff = top.width - background.width - 48`，再绘制 `-widthDiff/2`，即 `-(-23)/2 = 11.5`；应按代码复现，不自行改成 `(292-281)/2`。`OmegaFrameTop` 直接画到 292x512 expanded 画布；`backgroundGen` 才是先生成 256x512 group 再以 x=18 放置。
