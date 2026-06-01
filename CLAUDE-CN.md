# CLAUDE-CN.md

本文件为 Claude Code (claude.ai/code) 提供代码库工作指南。

## 项目概述

`minimax_agent` 是一个基于 Rust 的 CLI 工具，通过 MCP（Model Context Protocol）提供 MiniMax API 能力。封装了 MiniMax 的文本对话、语音合成、视频生成、图像生成、音乐生成和文件管理接口。

**已实现的 MCP 工具：**
- `text_to_audio` / `text_to_audio_stream` / `generate_audio_async` / `query_audio_task` — 语音合成
- `list_voices` / `voice_clone` / `voice_design` / `delete_voice` — 音色管理
- `generate_image` / `understand_image` — 图像
- `generate_video` / `generate_video_agent` / `query_video` / `query_video_agent` — 视频
- `generate_music` / `generate_music_cover` / `generate_lyrics` — 音乐
- `chat` / `web_search` — 文本对话
- `query_usage` — 账户用量
- `list_files` / `retrieve_file` / `delete_file` — 文件管理

## 开发环境配置

### 环境变量

```bash
export MINIMAX_API_KEY=your_key          # 中国区 (api.minimaxi.com)
export MINIMAX_API_KEY=your_key          # 国际区 (api.minimax.io)
```

### Rust 工具链

- **主要语言**：Rust
- **编译**：`cargo build --release` — MCP server 从 `target/release/minimax-mcp` 运行
- **调试构建**：`cargo build` → `target/debug/minimax-mcp`

### 添加 MCP Server 到 Claude Code

- **无需输入 API key** — server 启动时自动从 shell 环境变量读取 `MINIMAX_API_KEY`
- 通过二进制路径添加：`/path/to/minimax_agent/target/release/minimax-mcp`
- 代码修改后需重启：先执行 `pkill -f minimax-mcp`，然后退出并重新进入 Claude Code

## 架构

```
src/
├── bin/main_cli.rs     # CLI 入口（./minimax <command>）
├── client.rs           # MiniMaxClient（API 调用）
├── consts.rs          # API 端点、常量
├── error.rs           # MiniMaxError
├── lib.rs             # 库入口，导出类型
├── mcp_params.rs     # MCP 工具参数（serde）
├── types.rs           # 请求/响应类型
├── utils.rs           # process_image_url 及其他辅助函数
├── ws_client.rs       # WebSocket 客户端（流式 TTS）
└── tools/
    ├── chat.rs        # chat、web_search
    ├── files.rs       # 文件列表/获取/删除
    ├── image.rs       # 生成图像、图片理解
    ├── music.rs       # 生成音乐、音乐翻唱、歌词生成
    ├── search.rs      # 网络搜索
    ├── tts.rs          # 文字转语音、音色克隆、音色设计等
    ├── usage.rs       # 查询账户用量
    └── video.rs       # 视频生成、视频Agent
```

### MCP 传输方式

- **Stdio**（主要）— 用于 Claude Desktop 集成
- **SSE** — HTTP Server-Sent Events（配置后启用）

## 默认模型（最新）

| 能力 | 默认模型 | 说明 |
|------|----------|------|
| 语音合成（同步/流式/异步） | `speech-2.8-hd` | 9 种 emotion：happy/sad/angry/fearful/disgusted/surprised/calm/fluent/whisper |
| 视频生成 | `MiniMax-Hailuo-2.3` | 02 模型支持 6/10 秒时长 + 768P/1080P 分辨率 |
| 图像生成 | `image-01` | `image-01-live` 支持 style_type（cartoon/vitality 等） |
| 音乐生成 | `music-2.6` | 支持 `is_instrumental` 和 `lyrics_optimizer` |
| 文本对话 | `MiniMax-M3` | 1M 上下文窗口，最大输出 16,384 tokens |

## 开发指南

### Rust 编译规则

在 tool handler 中，`params.xxx` 字段在构造 `req` 对象时会被 move。此后的访问需通过 `req.xxx`（而非 `params.xxx`）。示例：

```rust
// params.text 在此被 move 到 req 中
let req = T2ARequest { text: params.text, .. };
// 之后使用 req.text，而非 params.text
```

### API 认证

- 编程计划接口（`/v1/coding_plan/search`、`/v1/coding_plan/vlm`）需要添加 header：`MM-API-Source: Minimax-MCP`
- API key 区域必须与 base URL 匹配：
  - 中国区：`https://api.minimaxi.com`
  - 国际区：`https://api.minimax.io`
- 图片理解接口（`understand_image`）需要 base64 data URL 格式——本地文件可通过 `utils::process_image_url()` 转换

### 测试命令

```bash
cargo run --bin minimax -- list_voices
cargo run --bin minimax -- query_usage
cargo run --bin minimax -- text_to_audio "你好"
cargo run --bin minimax -- web_search "关键词"
cargo run --bin minimax -- understand_image "描述" 项目路径/image.png
```

## 用户偏好

### 音色选择

**女性（优先）：**
- `Portuguese_LovelyLady` — Lovely Lady
- `female-yujie` — 御姐音色
- `Japanese_KindLady` — Kind Lady
- `Japanese_CalmLady` — Calm Lady

**男性：**
- `Japanese_GentleButler` — Gentle Butler

### 音频播放

生成音频后，使用以下命令播放：

```bash
afplay <file_path>
```

## 附录

### API 基础地址

| 区域   | Base URL                |
|--------|-------------------------|
| 中国区 | `https://api.minimaxi.com` |
| 国际区 | `https://api.minimax.io`    |

### 已知问题

- `/v1/get_voice` 拒绝空 JSON — 必须传递 `voice_type` 参数；`list_voices` 默认使用 `"system"`
- `voice_design` 和 `voice_clone` 需要账户有足够余额；余额不足会返回 API error 1008

### 相关项目

- MiniMax Rust CLI（独立项目） — **与本项目无关**

### 验证习惯

如果觉得有问题，先查阅相关文档/代码/测试结果后再判断，再询问用户。