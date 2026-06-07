# Subagent 调用时动态工具白名单

## 问题

当前 `RunSubagentParams` 只有 `name` + `task` 两个字段，subagent 能用哪些工具完全由
`subagents/<name>.json` 中的 `allowed_tools` 静态决定。调用方无法在运行时控制
subagent 的工具使用范围。

## 方案

在 `RunSubagentParams` 上新增 `allowed_tools: Option<Vec<String>>` 字段。
调用方可选择性传入，覆盖 JSON 配置中的默认值。不传则完全向后兼容。

## 改动文件（3 个，~15 行）

### 1. `src/mcp_params.rs`

```rust
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct RunSubagentParams {
    pub name: String,
    pub task: String,
    #[schemars(description = "工具白名单（可选）。覆盖 subagent JSON 配置中的 allowed_tools。不传则使用 JSON 中的默认值。")]
    pub allowed_tools: Option<Vec<String>>,
}
```

### 2. `src/tools/subagent.rs` — `handle_run_subagent`

```rust
let sub = registry.get(&params.name)...;

let sub_tools = if let Some(tool_override) = &params.allowed_tools {
    let mut effective = sub.clone();
    effective.allowed_tools = Some(tool_override.clone());
    tools_for_subagent(all_tools, &effective)
} else {
    tools_for_subagent(all_tools, sub)
};
```

### 3. `src/subagent_impl.rs` — `build_run_subagent_tool` 中的 execute 闭包

同上逻辑，在闭包内 `parse_input::<RunSubagentParams>()` 之后同样处理。

### 不改的文件
- `src/subagent/factory.rs` — `tools_for_subagent()` 签名不变
- `src/subagent/types.rs` — 不改
- `src/subagent/loop_runner.rs` — 不改
- `subagents/*.json` — 不改

## 关键设计决策

| 决策 | 选择 | 原因 |
|------|------|------|
| override 实现方式 | 调用处克隆 SubagentDef + 改 allowed_tools | 签名不变，零测试影响 |
| 递归渗透 | **不渗透** — 子 subagent 用自己的配置 | 行为最清晰可预测 |
| 空列表 `[]` | 只能调 `run_subagent`（与 JSON 语义一致） | 已有 `tools_for_subagent` 行为 |
| 不存在的工具名 | 静默忽略 | 已有 `tools_for_subagent` 行为 |

## 验证

```bash
cargo build                                           # 零 warning
cargo test --lib subagent::factory::tests             # 已有测试全过
cargo run --bin Subagent_cli -- run_subagent '...'    # 3 组手动测试
```

## 递归行为

当 subagent 内部再 `run_subagent` 到另一个子代理时，新子代理的工具有自己的 JSON 配置决定。
调用时传入的 `allowed_tools` 只影响当前这一层——不向下渗透。
