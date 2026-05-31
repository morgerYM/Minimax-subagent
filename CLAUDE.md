# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`minimax_agent` is a Rust-based CLI agent that uses MiniMax's API to provide AI capabilities via MCP (Model Context Protocol). It wraps MiniMax's text, speech, video, image, music, and file management APIs as MCP tools.

**Implemented MCP tools:**
- `text_to_audio` / `text_to_audio_stream` / `generate_audio_async` / `query_audio_task` — TTS
- `list_voices` / `voice_clone` / `voice_design` / `delete_voice` — voice management
- `generate_image` / `understand_image` — image
- `generate_video` / `generate_video_agent` / `query_video` / `query_video_agent` — video
- `generate_music` / `generate_music_cover` / `generate_lyrics` — music
- `chat` / `web_search` — text
- `query_usage` — account
- `list_files` / `retrieve_file` / `delete_file` — file management

## Development Setup

### Environment Variables

```bash
export MINIMAX_API_KEY=your_key          # China region (api.minimaxi.com)
export MINIMAX_API_KEY=your_key          # Global  (api.minimax.io)
```

### Rust Toolchain

- **Primary**: Rust
- **Build**: `cargo build --release` — the MCP server runs from `target/release/minimax-mcp`
- **Debug binary**: `cargo build` → `target/debug/minimax-mcp`

### Adding the MCP Server to Claude Code

- **Do NOT enter an API key** — the server reads `MINIMAX_API_KEY` from the shell environment automatically
- Add via binary path: `/Users/yyurk/my_project/minimax_agent/target/release/minimax-mcp`
- After code changes: `pkill -f minimax-mcp`, then restart Claude Code

## Architecture

```
src/
├── bin/main_cli.rs     # CLI entry point (./minimax <command>)
├── client.rs           # MiniMaxClient (API calls)
├── consts.rs           # API endpoints, constants
├── error.rs            # MiniMaxError
├── lib.rs              # Library root, re-exports types
├── mcp_params.rs      # MCP tool parameters (serde)
├── types.rs            # Request/response types
├── utils.rs            # process_image_url, helpers
├── ws_client.rs        # WebSocket client (streaming TTS)
└── tools/
    ├── chat.rs         # chat, web_search
    ├── files.rs        # list/retrieve/delete files
    ├── image.rs        # generate_image, understand_image
    ├── music.rs        # generate_music, generate_music_cover, generate_lyrics
    ├── search.rs       # web_search
    ├── tts.rs          # text_to_audio, voice_clone, voice_design, etc.
    ├── usage.rs       # query_usage
    └── video.rs        # generate_video, generate_video_agent
```

### MCP Transport

- **Stdio** (primary) — for Claude Desktop integration
- **SSE** — HTTP server-sent events (when configured)

## Development Guide

### Rust Compilation Rules

In tool handlers, `params.xxx` fields are moved when constructing the `req` object. After that, access the value via `req.xxx` (not `params.xxx`). Example:

```rust
// params.text is moved into req here
let req = T2ARequest { text: params.text, .. };
// use req.text, not params.text
```

### API Authentication

- Coding Plan endpoints (`/v1/coding_plan/search`, `/v1/coding_plan/vlm`) require header: `MM-API-Source: Minimax-MCP`
- API key region must match the base URL:
  - China: `https://api.minimaxi.com`
  - Global: `https://api.minimax.io`
- Image understanding API (`understand_image`) requires base64 data URL format — use `utils::process_image_url()` for local files

### Testing

```bash
cargo run --bin minimax -- list_voices
cargo run --bin minimax -- query_usage
cargo run --bin minimax -- text_to_audio "你好"
cargo run --bin minimax -- web_search "关键词"
cargo run --bin minimax -- understand_image "描述" /path/to/image.png
```

## User Preferences

### Preferred Voices

**Female (priority):**
- `Portuguese_LovelyLady` — Lovely Lady
- `female-yujie` — 御姐音色
- `Japanese_KindLady` — Kind Lady
- `Japanese_CalmLady` — Calm Lady

**Male:**
- `Japanese_GentleButler` — Gentle Butler

### Audio Playback

After generating audio, play with:

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

- `/v1/get_voice` rejects empty JSON — always pass `voice_type` parameter; `list_voices` defaults to `"system"`
- `voice_design` and `voice_clone` require sufficient account balance; insufficient balance returns API error 1008

### Related Projects

- MiniMax Rust CLI (separate project): `/Users/yyurk/github_project/minimax-code/` — **not related** to this repo