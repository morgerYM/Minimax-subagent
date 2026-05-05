# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Preferences

### Programming Languages
- **Primary**: Rust
- **Secondary**: Python (use `uv` package manager)
- **Python Version**: 3.13 (when creating new Python projects)

### Code Style
- Use **React-style** architecture (component-based, hooks mindset)

### Network Operations
- **Primary**: `playwright`, `agent-browser` for web information retrieval

## Project Overview

`minimax_agent` is a Rust-based CLI agent that uses MiniMax's API to provide AI capabilities via MCP (Model Context Protocol). The project is currently in early development.

## Current State

- `MiniMax_API_Reference.md` - API documentation for MiniMax platform
- No Rust project structure yet (no `Cargo.toml`)

## Development Goals

Build a Rust MCP Server CLI that provides MiniMax tools:
- Text generation and chat
- Speech synthesis (T2A)
- Video generation
- Image generation
- Music generation
- File management

## MiniMax API Reference

**Base URLs:**
- China: `https://api.minimaxi.com`
- International: `https://api.minimax.io`

**API Endpoints:**
- Text: `POST /v1/chat/completions` (OpenAI compatible)
- Anthropic: `POST /v1/messages` (Anthropic compatible)
- Speech: `POST /v1/t2a_v2`
- Video: `POST /v1/video_generation`
- Image: `POST /v1/image_generation`
- Music: `POST /v1/music_generation`
- File: `POST /v1/files/upload`, `GET /v1/files`

**Environment Variables:**
- `MINIMAX_API_KEY` - API key for authentication

## MCP Server Transport

- **Stdio** (primary) - for Claude Desktop integration
- **SSE** - HTTP server-sent events support

## Related Projects

MiniMax Rust CLI (separate project): `/Users/yyurk/github_project/minimax-code/`

##git 保存更改
在完成计划，并修改完成后，要存档，避免反复修改。保证不会丢失

## error[备注]
当前文件夹minimax agent 与 minimax-code没有任何关系


