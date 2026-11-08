# Riven analysis core

The independent Rust core is `src-tauri/src/riven_analysis.rs`; its embedded seed is `src-tauri/data/riven-analysis-reference.json`. The seed contains 7 internal Riven types and 165 source coefficient records, plus structured Market attribute `gameRef`/unit metadata and the verified melee aliases.

The current rule mode is Warframe 44.1.3: all Rivens use the equivalent of disposition `D=1.2`. Legacy mode accepts `D` from the request or weapon catalog and validates `0.5..1.55`.

For a verified coefficient `b`, rank `r`, disposition `D`, and normalized roll `u`, the core computes:

`v = b * 10 * (r + 1) * D * L * (0.9 + 0.2u)`

with `r=0..8`; `L` is `0.99`, `1.2375`, `0.75`, or `0.9375` for the four positive layouts, and `-0.495`/`-0.75` for a negative stat. It returns a theoretical numeric interval and explicitly labels position as an interval position, not a probability percentile.

`percent` values are shown as `v*100`; `multiply` values are shown as `1+v` (so `1.5x` is a `+50%` delta, not `+150%`). Source negative coefficients, including recoil reduction, are preserved; no second sign flip is applied. Unverified `reverseValueSymbol`, spliced traits, or missing coefficients return an `unsupported` row rather than an invented value.

范围提示通过本地 `riven_analysis_range_hints` IPC 提供理论区间，不请求市场 HTTP。它与 `analyze` 共用 `layout_factors`、`theoretical_value` 和同源 reference；新增/删除正负槽位时批量重算已选项，单独修改词条只重算该项，value 输入不会触发范围请求或覆盖用户值。空输入显示真实本地范围，未知 metadata、方向不匹配和未核实 `reverseValueSymbol` 保持 unsupported。UI 约束为左栏 480px、数值输入 145px、上传按钮居中，窗口宽度 ≤1100px 时单列；稳定 id/context 与 row generation 用于丢弃旧响应。新接口已用真实 catalog/reference 与 `analyze` 做一致性定向测试，不将此表述为完整 helper 重构。

The source is a community-maintained Calamity repository, not Digital Extremes. URLs and SHA-256 values are embedded in the seed and duplicated in the local research cache under `local-dist/riven-analysis-research/`.
