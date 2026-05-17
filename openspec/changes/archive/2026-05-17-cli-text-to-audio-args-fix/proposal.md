# Proposal: CLI text_to_audio 参数解析修复

## Problem Statement

CLI `text_to_audio` 命令的参数解析存在 bug：当文本参数先传入时，解析逻辑会在 `break` 处跳出循环，导致 `--voice`、`--speed`、`--pitch`、`--emotion` 等选项参数被完全忽略。

## Root Cause

原解析逻辑：
1. 遇到非选项参数（文本）时，收集文本并 `break` 跳出 while 循环
2. `break` 导致后续所有选项参数未被处理
3. 无论用户传入什么 `--voice`，实际都使用默认值 `female-shaonv`

## Goals

- 修复参数解析逻辑，确保所有选项参数都能被正确处理
- 默认音色改为 `female-yujie`（御姐音色，用户明确要求）
- 支持 `--voice`、`--speed`、`--pitch`、`--emotion` 四个选项

## Scope

### In Scope
- 修复 `src/bin/main_cli.rs` 的 `text_to_audio` 命令参数解析
- 添加 `--pitch` 和 `--emotion` 选项支持

### Out of Scope
- 其他命令的参数解析修改
- MCP server 端的修改

## Success Criteria

- `./minimax text_to_audio "你好" --voice male-qn-qingse` 使用男性音色
- `./minimax text_to_audio "你好" --voice Santa_Claus` 使用圣诞老人音色
- 不同 `--voice` 参数产生明显不同的声音输出