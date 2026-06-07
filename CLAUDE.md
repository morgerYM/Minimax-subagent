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

## Provider Configuration

### provider.toml (project root)

Each capability independently selects its provider. Missing file defaults everything to `minimax`.

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

- **API key never in config** — `api_key_env` tells the factory which env var to read
- No `provider.toml` → all defaults to `minimax` with `MINIMAX_API_KEY` / `MINIMAX_API_HOST` env vars

## Development Setup

### Environment Variables

```bash
export MINIMAX_API_KEY=your_key          # China region (api.minimaxi.com)
export MINIMAX_API_HOST=https://api.minimax.io  # optional, defaults to China
```

### Rust Toolchain

- **Primary**: Rust
- **Build**: `cargo build --release` — MCP server at `target/release/Subagent-mcp`, CLI at `target/release/Subagent_cli`
- **Debug binary**: `cargo build` → `target/debug/`

### Adding the MCP Server to Claude Code

- **Do NOT enter an API key** — the server reads `MINIMAX_API_KEY` from the shell environment automatically
- Add via binary path: `/path/to/Subagent-mcp/target/release/Subagent-mcp`
- After code changes: `pkill -f Subagent-mcp`, then restart Claude Code

## Architecture

```
src/
├── bin/main_cli.rs              # CLI entry point (./Subagent_cli <command>)
├── main.rs                       # MCP server (stdio transport, #[tool_router])
├── subagent_impl.rs              # McpToolDispatcher (subagent tool → handler routing)
├── client.rs                     # MiniMaxClient (HTTP API calls, internal to MiniMaxProvider)
├── consts.rs                     # API endpoints, default model names, polling constants
├── error.rs                      # MiniMaxError
├── lib.rs                        # Library root (crate: minimax_api), re-exports
├── mcp_params.rs                 # MCP tool parameter structs (serde + schemars)
├── types.rs                      # MiniMax API request/response types (internal)
├── utils.rs                      # Hex decode, file save, image URL processing
├── ws_client.rs                  # WebSocket client for streaming TTS
│
├── tools/                        # Trait definitions + handler functions
│   ├── mod.rs
│   ├── tts/mod.rs                # TtsProvider + VoiceProvider traits + 8 handlers
│   ├── video/mod.rs              # VideoProvider trait + 4 handlers
│   ├── image/mod.rs              # ImageProvider trait + handler
│   ├── music/mod.rs              # MusicProvider trait + 3 handlers
│   ├── chat/mod.rs               # ChatProvider trait + handler
│   ├── search/mod.rs             # SearchProvider trait + 2 handlers
│   ├── files/mod.rs              # FileProvider trait + 3 handlers
│   ├── usage/mod.rs              # UsageProvider trait + handler
│   └── subagent.rs               # Subagent handlers (binary-only, not part of library)
│
├── providers/                    # Generic types + provider implementations
│   ├── mod.rs                    # MediaOutput, AsyncTaskHandle, ProviderError, factory
│   └── minimax/
│       ├── mod.rs                # MiniMaxProvider struct
│       ├── tts.rs                # impl TtsProvider + VoiceProvider for MiniMaxProvider
│       ├── video.rs              # impl VideoProvider
│       ├── image.rs              # impl ImageProvider
│       ├── music.rs              # impl MusicProvider
│       ├── chat.rs               # impl ChatProvider
│       ├── search.rs             # impl SearchProvider
│       ├── files.rs              # impl FileProvider
│       └── usage.rs              # impl UsageProvider
│
└── subagent/                     # Generic agent loop framework (library)
    ├── types.rs                  # SubagentDef, LoopResult
    ├── registry.rs               # Load subagents/*.json
    ├── loop_runner.rs            # run_agent_loop()
    ├── dispatcher.rs             # ToolDispatcher trait
    └── tool_catalog.rs           # 24 static tool specs
```

### Data Flow

```
MCP Client → stdio → main.rs #[tool] macro
  → tools/xxx/mod.rs handler(&dyn Trait, params)
  → provider trait method (MediaOutput / generic types)
  → MiniMaxProvider internal code (MiniMaxClient + types.rs)
  → MiniMax API

CLI → main_cli.rs command parsing
  → MiniMaxClient directly (not yet migrated to traits)
  → MiniMax API
```

### Key Design Principles

- **Generators** (tts, video, image, music, voice_clone, voice_design) return `MediaOutput::Bytes` or `MediaOutput::Url` — handlers handle output_file save
- **Async tasks** (video, async tts) return `AsyncTaskHandle` on submit, `AsyncTaskResult` on query
- **Query/report tools** (list_voices, search, files, usage) return domain-specific structs
- Handler functions never construct API requests or call MiniMaxClient — that's all in MiniMaxProvider
- Each `#[tool]` method gets its own `Arc<dyn Trait>` field on `MiniMaxMcp`

## Default Models (Latest)

| Capability | Default Model | Notes |
|------------|---------------|-------|
| TTS (sync/stream/async) | `speech-2.8-hd` | 9 emotions: happy/sad/angry/fearful/disgusted/surprised/calm/fluent/whisper |
| Video | `MiniMax-Hailuo-2.3` | 02 model adds 6/10s duration + 768P/1080P resolution |
| Image | `image-01` | `image-01-live` adds style_type (cartoon/vitality/etc.) |
| Music | `music-2.6` | Supports `is_instrumental` and `lyrics_optimizer` |
| Chat | `MiniMax-M3` | 1M context window, max 16,384 output tokens |

### MCP Transport

- **Stdio** (primary) — for Claude Desktop integration

## Adding a New Provider

1. Create `src/providers/<name>/mod.rs` with a struct wrapping the provider's client
2. Implement the traits you want to support in `src/providers/<name>/{tts,video,...}.rs`
3. Add `[provider_config.<name>]` to `provider.toml`
4. Update `create_provider_set()` in `src/providers/mod.rs` to handle the new provider name

## Development Guide

### Provider Trait Pattern

Each handler function signature:
```rust
pub async fn handle_xxx(
    provider: &dyn XxxProvider,
    params: XxxParams,
) -> Result<CallToolResult, ErrorData>
```

Each provider impl method signature:
```rust
async fn xxx(&self, params: &XxxParams) -> Result<OutputType, ProviderError>
```

MiniMaxProvider wraps `MiniMaxClient` (from `src/client.rs`) and all MiniMax-specific types (from `src/types.rs`) are internal.

### API Authentication

- Coding Plan endpoints (`/v1/coding_plan/search`, `/v1/coding_plan/vlm`) require header: `MM-API-Source: Minimax-MCP`
- API key region must match the base URL:
  - China: `https://api.minimaxi.com`
  - Global: `https://api.minimax.io`
- Image understanding API (`understand_image`) requires base64 data URL format — use `utils::process_image_url()` for local files

### Testing

```bash
cargo run --bin Subagent_cli -- list_voices
cargo run --bin Subagent_cli -- query_usage
cargo run --bin Subagent_cli -- text_to_audio "你好"
cargo run --bin Subagent_cli -- web_search "关键词"
cargo run --bin Subagent_cli -- understand_image "描述" 项目路径/image.png
```

### Syncing Official API Docs (docs/)

```bash
./scripts/sync_docs.sh                  # sync all project-relevant docs (default)
./scripts/sync_docs.sh all              # sync every page from llms.txt
./scripts/sync_docs.sh --list           # list available slugs
./scripts/sync_docs.sh <slug> [<slug>]  # sync specific docs only
```

## Git & Publishing

### Privacy Rules (DO NOT COMMIT)

- **`.claude/`** — contains local settings and possibly API keys, use `.git/info/exclude` instead
- **`.env`** — never commit environment files
- **Absolute paths** — never commit paths containing your username, use `项目路径/...` in docs
- **mp3 / media files** — generated outputs should not be committed

### Local-only Ignore Rules

Use `.git/info/exclude`:
```
target/
.claude/
.gitignore
.env
*.mp3
```

## User Preferences

### Preferred Voices

**Female (priority):**
- `Portuguese_LovelyLady`
- `female-yujie`
- `Japanese_KindLady`
- `Japanese_CalmLady`

**Male:**
- `Japanese_GentleButler`

### Audio Playback

```bash
afplay <file_path>
```

## Appendix

### API Base URLs

| Region   | Base URL              |
|----------|-----------------------|
| China    | `https://api.minimaxi.com` |
| Global   | `https://api.minimax.io` |

### Known Issues

- `/v1/get_voice` rejects empty JSON — always pass `voice_type` parameter
- `voice_design` and `voice_clone` require sufficient account balance; insufficient balance returns API error 1008

### Build Output

- Library crate: `minimax_api` (no binary)
- MCP server binary: `Subagent-mcp`
- CLI binary: `Subagent_cli`

## git
- 只能git add，git commit -m,
