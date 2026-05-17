# fix-list-voices-2013-error

## 问题描述

`list_voices` 调用 `/v1/get_voice` API 时返回 `API error 2013: invalid params`。

## 根因分析

1. 当 `voice_type=None` 时，`serde` 生成空 JSON `{}`
2. MiniMax `/v1/get_voice` API 拒绝空 JSON，必须传 `voice_type` 参数
3. API 测试结果：
   - `{}` → 2013 invalid params
   - `{"voice_type":"system"}` → 成功，返回 303 个系统音色

## 修复方案

修改 `src/client.rs` 第 148-156 行，`list_voices` 函数默认使用 `"system"` 而非 `None`：

```rust
let req = VoiceListRequest {
    voice_type: voice_type.map(String::from).or_else(|| Some("system".to_string())),
};
```

## 修改文件

- `src/client.rs` — 修改 `list_voices` 函数

## 验证

```bash
cargo build --release --bin test_mcp
cargo run --bin test_mcp -- list_voices
# 成功列出 303+ 系统音色
```

## 记录时间

2026-05-17