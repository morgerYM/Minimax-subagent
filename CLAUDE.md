# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`Subagent-mcp` is a Rust-based project that provides AI capabilities via MCP (Model Context Protocol) with a **trait-based provider architecture**. The library defines capability interfaces, and each provider (e.g. MiniMax) implements them. The MCP server and CLI both consume providers through traits.

### Capability Traits (9 interfaces)

| Trait | Covers | File |
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

### Subagent tools (separate, not provider-backed)

- `run_subagent` / `list_subagents` / `get_subagent` — subagent management (uses MiniMaxClient directly)

  `run_subagent` 支持运行时工具白名单覆盖：调用时可传 `allowed_tools` 参数
  （`Option<Vec<String>>`），覆盖 subagent JSON 中的静态配置。不传则回退到 JSON 配置。
  参见 `src/mcp_params.rs` 中的 `RunSubagentParams`。

## Provider Configuration

### provider.toml (project root)

Each capability independently selects its provider. Missing file defaults everything to `minimax`.

```toml
[providers]
tts = "minimax"
voice = "minimax"
video = "minimax"
...
usage = "minimax"

[provider_config.minimax]
api_key_env = "MINIMAX_API_KEY"
api_host = "https://api.minimaxi.com"
```

- **API key never in config** — `api_key_env` tells the factory which env var to read
- No `provider.toml` → all defaults to `minimax` with `MINIMAX_API_KEY` / `MINIMAX_API_HOST` env vars

## Development Setup

- **Env**: `MINIMAX_API_KEY=your_key`, `MINIMAX_API_HOST` (optional, defaults to China)
- **Build**: `cargo build --release` → binaries: `Subagent-mcp`, `Subagent_cli`
- **Claude Code**: Add via binary path, no API key in config. After changes: `pkill -f Subagent-mcp`

## Architecture

```
src/
├── bin/main_cli.rs           # CLI
├── main.rs                    # MCP server (stdio, #[tool_router])
├── subagent_impl.rs           # Tool factories + run_subagent builder
├── client.rs / consts.rs / error.rs / types.rs / utils.rs / ws_client.rs / mcp_params.rs / lib.rs
├── tools/                     # Trait defs + handlers (9 traits, see table above)
│   └── {tts,video,image,music,chat,search,files,usage}/ + subagent.rs
├── providers/minimax/         # MiniMax provider impls
│   └── {tts,video,image,music,chat,search,files,usage}.rs
└── subagent/                  # Agent loop framework
    └── types.rs / registry.rs / loop_runner.rs / agent_tool.rs / factory.rs
```

### Data Flow

```
MCP → main.rs #[tool] → handler(&dyn Trait) → provider trait → MiniMaxClient → API
CLI → main_cli.rs → MiniMaxClient → API
Subagent: registry.get → tools_for_subagent() (params override JSON) → run_agent_loop
  → AgentTool::to_spec() (LLM) → AgentTool::execute() (dispatch)
```

### Key Design

- Handlers never construct API requests — that's all in MiniMaxProvider. Each `#[tool]` has its own `Arc<dyn Trait>`.
- Subagent tools: self-contained `AgentTool` (schema + execute) via `ToolRegistry` factory pattern.
- `run_subagent` supports runtime `allowed_tools` override — params take priority over JSON config, only affects current layer.

## Default Models (Latest)

| Capability | Default Model | Notes |
|------------|---------------|-------|
| TTS (sync/stream/async) | `speech-2.8-hd` | 9 emotions: happy/sad/angry/fearful/disgusted/surprised/calm/fluent/whisper |
| Video | `MiniMax-Hailuo-2.3` | 02 model adds 6/10s duration + 768P/1080P resolution |
| Image | `image-01` | `image-01-live` adds style_type (cartoon/vitality/etc.) |
| Music | `music-2.6` | Supports `is_instrumental` and `lyrics_optimizer` |
| Chat | `MiniMax-M3` | 1M context window, max 16,384 output tokens |

## Development Guide

### Provider trait pattern
- Handler: `handle_xxx(&dyn Trait, Params) -> Result<CallToolResult, ErrorData>`
- Provider impl: `xxx(&self, &Params) -> Result<OutputType, ProviderError>`
- MiniMaxClient + types.rs are internal to MiniMaxProvider

### API notes
- Coding Plan endpoints need `MM-API-Source: Minimax-MCP` header
- Base URLs: China `api.minimaxi.com` / Global `api.minimax.io`; region must match API key
- `understand_image` needs base64 — use `utils::process_image_url()`
- Known issues: `/v1/get_voice` needs `voice_type`; `voice_design`/`voice_clone` need balance

### Adding a provider
1. Create `src/providers/<name>/` with `mod.rs` + trait impls
2. Add `[provider_config.<name>]` to `provider.toml`
3. Update `create_provider_set()` in `src/providers/mod.rs`

### Testing / docs
```bash
cargo run --bin Subagent_cli -- text_to_audio "你好"
./scripts/sync_docs.sh [all|--list|<slug>]
```

## Git & Publishing

- DO NOT COMMIT: `.claude/`, `.env`, absolute paths, `*.mp3`
- Use `.git/info/exclude` for `target/`, `.claude/`, `.gitignore`, `.env`, `*.mp3`
- main 分支：git merge 合并 + 处理冲突 + 上传
- 其他分支：只能 `git add`，`git commit -m`

## User Preferences

- **Voices (female)**: `Portuguese_LovelyLady`, `female-yujie`, `Japanese_KindLady`, `Japanese_CalmLady`
- **Voices (male)**: `Japanese_GentleButler`
- **Playback**: `afplay <file_path>`


