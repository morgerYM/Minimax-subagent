# Design: CLI text_to_audio 参数解析修复

## Problem Analysis

### Original Buggy Code

```rust
// 原代码的问题逻辑
while i < args.len() {
    let arg = &args[i];
    if arg == "--voice" && i + 1 < args.len() { ... }
    else if arg == "--speed" && i + 1 < args.len() { ... }
    else if arg.starts_with("--") { exit(1); }
    else {
        text = arg.clone();
        i += 1;
        while i < args.len() && !args[i].starts_with("--") {
            text.push(' ');
            text.push_str(&args[i]);
            i += 1;
        }
        break;  // ← BUG: 跳出循环，--voice 等后续选项被忽略
    }
}
```

### Fixed Code

```rust
let mut found_text = false;
while i < args.len() {
    let arg = &args[i];
    if arg == "--voice" && i + 1 < args.len() { voice_id = args[i + 1].clone(); i += 2; }
    else if arg == "--speed" && i + 1 < args.len() { speed = args[i + 1].parse().ok(); i += 2; }
    else if arg == "--pitch" && i + 1 < args.len() { pitch = args[i + 1].parse().ok(); i += 2; }
    else if arg == "--emotion" && i + 1 < args.len() { emotion = Some(args[i + 1].clone()); i += 2; }
    else if arg.starts_with("--") { exit(1); }
    else {
        // 第一个非选项参数是文本，之后的非选项参数也合并到文本
        if !found_text {
            text = arg.clone();
            found_text = true;
        } else {
            text.push(' ');
            text.push_str(arg);
        }
        i += 1;
    }
}
```

### Key Changes

1. **移除 `break`** — 循环继续处理所有参数
2. **添加 `found_text` 标志** — 区分文本参数和选项参数
3. **第一个非选项参数是文本** — 之后遇到非选项参数也合并到文本（支持多词文本）

## T2A API Parameters

| 参数 | 类型 | 范围/可选值 | 说明 |
|------|------|-------------|------|
| `--voice` | String | 音色ID列表 | 音色选择 |
| `--speed` | f64 | 0.5-2.0 | 语速 |
| `--pitch` | i32 | -12~12 | 音调 |
| `--emotion` | String | happy/sad/angry/... | 情感 |