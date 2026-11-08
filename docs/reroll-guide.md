# Reroll planner implementation

The Rust reroll core is based on `wxhn1225/riven-roll` commit `986f37c7ae0057be4a4c3e8251cb03080507241a` and uses the embedded game snapshot for gameRef/alias and slug↔tag mapping. `RerollService` serializes catalog-backed requests, keeps a bounded planner graph cache, and runs planning in `spawn_blocking`; stale requests are rejected.

The planner validates display values through the signed display/quality conversion, supports current/new card comparison, splicing, lock routes, quantiles and budget probabilities. Unknown or unmapped game references (including PlagueKeewar/Kripath PvP and `vinquibus_melee`) return explicit input errors. Native GUI acceptance remains pending.

Validation: `cargo test --offline --manifest-path src-tauri/Cargo.toml --lib reroll_` passed 12 tests (4 core boundary, 1 nine-case core golden loop, 7 planner/UI fixture scenarios). Full build and native UI validation were not run.

## 2026-11-09 completion status

熔接范围已完成 776 项 JS oracle 对照，并有真实 API 层 2+1/2+0 测试。完整 core/planner 定向测试已通过（9 组上游案例循环、4 边界、7 planner 场景）；已接入源作者完整求解、模拟、方案与比较实现，不再待设计。Planner 与分析共用 gameRef/数据版本。root 已完成真实 Rust fixture 的 Web/mock IPC 验收：常规规划+保留新卡只记一次、免费熔接不扣次数赤毒，1280x720 light/900x760 dark 无横溢且 console 无新增 warn/error；不等同原生全流程。PlagueKeewar、Kripath PvP gameRef、`vinquibus_melee` 无法准确映射；单模型工作预算为 2,000,000，概率采用上游无放回等概率与均匀 W 假设。未执行完整构建。
