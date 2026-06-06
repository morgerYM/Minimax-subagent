# Subagents

Subagent 是**带预设 system prompt 的 LLM agent**,运行时进入 agent loop:
- 可以调用任意 MCP 工具
- 可以**递归**调用其他 subagent(包括自己,自行处理深度上限)
- 通过 `run_subagent` MCP 工具暴露给 Claude Code / 其他 MCP 客户端

Subagent 定义在**本目录**下,每个 subagent 一个 JSON 文件。文件名不影响加载,只跟 `name` 字段有关(必须全局唯一)。

## 启动时加载

MCP server 启动时:
1. 读取本目录下所有 `*.json` 文件
2. 解析为 `SubagentDef`
3. 校验字段(name / description / system 必填)
4. 加载到内存中的 `SubagentRegistry`
5. 缺失目录不报错,只 warn

**热加载不支持** — 改完 JSON 后需要重启 MCP server。

## JSON Schema

| 字段 | 类型 | 必填 | 默认 | 说明 |
|---|---|---|---|---|
| `name` | string | ✅ | — | 唯一标识,通过 `run_subagent(name=...)` 调用 |
| `description` | string | ✅ | — | 一句话描述,`list_subagents` 返回 |
| `system` | string | ✅ | — | System prompt 全文(用户写) |
| `model` | string | ❌ | `MiniMax-M3` | LLM 模型 |
| `max_tokens` | int | ❌ | 16384 | 单次 LLM 调用上限 |
| `temperature` | float | ❌ | API 默认 | 0-1 |
| `max_iterations` | int | ❌ | 10 | Agent loop 最多循环次数 |
| `allowed_tools` | string[] | ❌ | 全部 | 工具白名单;`run_subagent` 始终隐式包含 |
| `max_depth` | int | ❌ | 3 | 递归调用 `run_subagent` 时允许的最大深度 |

## 完整示例

`subagents/video-creator.json`:

```json
{
  "name": "video-creator",
  "description": "短视频创作专家,编排图像 + 视频 + 配音",
  "system": "你是一个短视频创作专家,擅长把一个主题变成 30 秒短视频。\n\n工作流:\n1. 用 generate_image 画一张主视觉\n2. 用 generate_video 把主视觉变成 6 秒动态片段\n3. 用 text_to_audio 用 happy 情感合成旁白\n4. 总结三件产出物的 URL\n\n每次只调用一个工具,看完结果再决定下一步。",
  "model": "MiniMax-M3",
  "max_tokens": 16384,
  "max_iterations": 10,
  "allowed_tools": ["generate_image", "generate_video", "text_to_audio"]
}
```

## 递归示例

`subagents/orchestrator.json`(委派给 worker):

```json
{
  "name": "orchestrator",
  "description": "顶层 orchestrator,委派给 worker",
  "system": "用户的任务你应该委托给 worker subagent 完成:\nrun_subagent(name=\"worker\", task=<用户原始任务>)\n拿到结果后,直接把它说给用户。"
}
```

`subagents/worker.json`(实际干活):

```json
{
  "name": "worker",
  "description": "通用任务执行者",
  "system": "你收到一个任务,就用 web_search / generate_image / chat 等工具组合完成。完成后返回 'task done: <one-line summary>'。"
}
```

调用 `run_subagent({name: "orchestrator", task: "搜索 Rust 2024 生态"})` 时:
1. orchestrator LLM 决定调 `run_subagent({name: "worker", task: "..."})`
2. dispatcher 路由到 worker 的 agent loop(depth=1)
3. worker 完成,final_output 回给 orchestrator
4. orchestrator LLM 看到 worker 结果,再调一次 end_turn
5. 整个调用返回的 `tool_calls` 数组里能看到完整的 orchestrator → worker 调用链

## 限制

- **Token 成本**:每次 LLM 调用都花钱,递归会指数级放大
- **执行时间**:长链 agent loop 跑完可能要 1-2 分钟,Claude Code 等待不友好(暂不支持流式)
- **错误传播**:子 subagent 失败时,LLM 看到的是错误字符串,需要自己决定重试或放弃
- **无状态**:每次 `run_subagent` 调用是独立的,subagent 之间无 shared memory

## 调试

调 `get_subagent({name: "..."})` 查看实际生效的:
- `system` 是否被正确读取
- `effective_tools` 是 LLM 实际看到的工具列表(按 `allowed_tools` 过滤)
- `max_depth` / `max_iterations` 是否符合预期
