# CLAUDE-CN.md

本文件为 Claude Code (claude.ai/code) 提供代码库工作指南。

## 项目概述

`Subagent-mcp` 是一个 Rust 项目，通过 MCP（Model Context Protocol）和**基于 trait 的 provider 架构**提供 AI 能力。库层定义能力接口，各供应商（如 MiniMax）实现这些接口。MCP server 和 CLI 都通过 trait 消费 provider。

### 能力 Trait（9 个接口）

| Trait | 覆盖工具 | 文件 |
|---|---|---|
| `TtsProvider` | `text_to_audio`, `text_to_audio_stream`, `generate_audio_async`, `query_audio_task` | `src/tools/tts/mod.rs` |
| `VoiceProvider` | `list_voices`, `voice_clone`, `voice_design`, `delete_voice` | `src/tools/tts/mod.rs` |
| `VideoProvider` | `generate_video`, `query_video`, `generate_video_agent`, `query_video_agent` | `src/tools/video/mod.rs` |
| `ImageProvider` | `generate_image` | `src/tools/image/mod.rs` |
| `MusicProvider` | `generate_music`, `generate_lyrics`, `generate_music_cover` | `src/tools/music/mod.rs` |
| `ChatProvider` | `chat` | `src/tools/chat/mod.rs` |
| `SearchProvider` | `web_search`, `understand_image` | `src/tools/search/mod.rs` |
| `FileProvider` | `list_files`, `retrieve_file`, `delete_file`, `upload_file` | `src/tools/files/mod.rs` |
| `UsageProvider` | `query_usage` | `src/tools/usage/mod.rs` |

### Subagent 工具（独立，不走 provider）

- `run_subagent` / `list_subagents` / `get_subagent` — subagent 管理（直接使用 MiniMaxClient）

## Provider 配置

### provider.toml（项目根目录）

每个能力独立选择供应商。文件缺失时全部默认 `minimax`。

```toml
[providers]
tts = "minimax"
voice = "minimax"
video = "minimax"
image = "minimax"
music = "minimax"
chat = "minimax"
search = "minimax"
files = "minimax"
usage = "minimax"

[provider_config.minimax]
api_key_env = "MINIMAX_API_KEY"
api_host = "https://api.minimaxi.com"
```

- **API key 不在配置文件中** — `api_key_env` 告诉 factory 从哪个环境变量读取
- 无 `provider.toml` → 全部默认 `minimax` + `MINIMAX_API_KEY` / `MINIMAX_API_HOST` 环境变量

## 环境变量

```bash
export MINIMAX_API_KEY=your_key          # 中国区 (api.minimaxi.com)
export MINIMAX_API_HOST=https://api.minimax.io  # 可选，默认中国区
```

## Rust 工具链

- **主要语言**：Rust
- **编译**：`cargo build --release` — MCP server: `target/release/Subagent-mcp`，CLI: `target/release/Subagent_cli`
- **调试构建**：`cargo build` → `target/debug/`

## 架构

```
src/
├── bin/main_cli.rs              # CLI 入口 (./Subagent_cli <command>)
├── main.rs                       # MCP server (stdio transport, #[tool_router])
├── subagent_impl.rs              # McpToolDispatcher (subagent 工具路由)
├── client.rs                     # MiniMaxClient (HTTP API, MiniMaxProvider 内部使用)
├── consts.rs                     # API 端点、默认模型常量
├── error.rs                      # MiniMaxError
├── lib.rs                        # 库入口 (crate: minimax_api)
├── mcp_params.rs                 # MCP 工具参数结构体 (serde + schemars)
├── types.rs                      # MiniMax API 请求/响应类型 (内部)
├── utils.rs                      # Hex 解码、文件保存、图片 URL 处理
├── ws_client.rs                  # WebSocket 客户端 (流式 TTS)
│
├── tools/                        # Trait 定义 + handler 函数
│   ├── mod.rs
│   ├── tts/mod.rs                # TtsProvider + VoiceProvider + 8 handler
│   ├── video/mod.rs              # VideoProvider + 4 handler
│   ├── image/mod.rs              # ImageProvider + handler
│   ├── music/mod.rs              # MusicProvider + 3 handler
│   ├── chat/mod.rs               # ChatProvider + handler
│   ├── search/mod.rs             # SearchProvider + 2 handler
│   ├── files/mod.rs              # FileProvider + 3 handler
│   ├── usage/mod.rs              # UsageProvider + handler
│   └── subagent.rs               # Subagent handler (binary-only)
│
├── providers/                    # 通用类型 + 各供应商实现
│   ├── mod.rs                    # MediaOutput, AsyncTaskHandle, ProviderError, factory
│   └── minimax/
│       ├── mod.rs                # MiniMaxProvider 结构体
│       ├── tts.rs                # impl TtsProvider + VoiceProvider
│       ├── video.rs              # impl VideoProvider
│       ├── image.rs              # impl ImageProvider
│       ├── music.rs              # impl MusicProvider
│       ├── chat.rs               # impl ChatProvider
│       ├── search.rs             # impl SearchProvider
│       ├── files.rs              # impl FileProvider
│       └── usage.rs              # impl UsageProvider
│
└── subagent/                     # 通用 agent loop 框架 (库)
    ├── types.rs
    ├── registry.rs
    ├── loop_runner.rs
    ├── dispatcher.rs
    └── tool_catalog.rs
```

### 数据流

```
MCP Client → stdio → main.rs #[tool] 宏
  → tools/xxx/mod.rs handler(&dyn Trait, params)
  → provider trait 方法 (返回通用类型)
  → MiniMaxProvider 内部 (MiniMaxClient + types.rs)
  → MiniMax API

CLI → main_cli.rs 命令解析
  → MiniMaxClient 直接调用 (暂未迁移到 trait)
  → MiniMax API
```

## 添加新供应商

1. 创建 `src/providers/<name>/mod.rs`，封装供应商客户端
2. 实现需要的能力 trait（不必全部）
3. 在 `provider.toml` 添加 `[provider_config.<name>]`
4. 更新 `src/providers/mod.rs` 的 `create_provider_set()` 处理新供应商名

## 默认模型

| 能力 | 默认模型 | 说明 |
|------|----------|------|
| 语音合成 | `speech-2.8-hd` | 9 种 emotion |
| 视频 | `MiniMax-Hailuo-2.3` | 02 支持 6/10 秒 + 768P/1080P |
| 图像 | `image-01` | `image-01-live` 支持 style_type |
| 音乐 | `music-2.6` | 支持 `is_instrumental` |
| 文本对话 | `MiniMax-M3` | 1M 上下文窗口 |

## 开发指南

### Provider Trait 模式

Handler 函数签名：
```rust
pub async fn handle_xxx(
    provider: &dyn XxxProvider,
    params: XxxParams,
) -> Result<CallToolResult, ErrorData>
```

Provider 方法签名：
```rust
async fn xxx(&self, params: &XxxParams) -> Result<OutputType, ProviderError>
```

MiniMaxProvider 内部封装 `MiniMaxClient`，`types.rs` 全部类型为内部使用。

### API 认证

- Coding Plan 接口需要 header: `MM-API-Source: Minimax-MCP`
- API key 区域必须匹配 base URL
- 图片理解接口需要 base64 data URL — 使用 `utils::process_image_url()`

### 测试命令

```bash
cargo run --bin Subagent_cli -- list_voices
cargo run --bin Subagent_cli -- query_usage
cargo run --bin Subagent_cli -- text_to_audio "你好"
cargo run --bin Subagent_cli -- web_search "关键词"
```

## Git 与发布

### 隐私规则（禁止提交）

- **`.claude/`** — 使用 `.git/info/exclude` 替代
- **`.env`** — 禁止提交
- **绝对路径** — 文档中使用 `项目路径/...`
- **mp3 / 媒体文件** — 禁止提交

### 本地排除规则 (`.git/info/exclude`)

```
target/
.claude/
.gitignore
.env
*.mp3
```

## 用户偏好

### 音色选择

**女性（优先）：**
- `Portuguese_LovelyLady`、`female-yujie`、`Japanese_KindLady`、`Japanese_CalmLady`

**男性：**
- `Japanese_GentleButler`

### 音频播放

```bash
afplay <file_path>
```

## 附录

### API 基础地址

| 区域 | Base URL |
|------|----------|
| 中国区 | `https://api.minimaxi.com` |
| 国际区 | `https://api.minimax.io` |

### 构建产物

| 名称 | 类型 |
|------|------|
| `minimax_api` | 库 crate (无二进制) |
| `Subagent-mcp` | MCP server 二进制 |
| `Subagent_cli` | CLI 二进制 |

## git
- 只能git add，git commit -m,
