# Subagent MCP Server

Rust 实现的 MCP 服务器，通过 Model Context Protocol 提供 AI 工具。采用 **trait-based provider 架构**，支持按能力选择不同的 LLM 供应商，默认集成 MiniMax API。

## 架构

```
Tools (trait 定义 + handler)          Providers (供应商实现)
─────────────────────────────      ─────────────────────────
src/tools/tts/mod.rs               src/providers/minimax/tts.rs
  ├── TtsProvider trait                ├── impl TtsProvider
  ├── VoiceProvider trait              ├── impl VoiceProvider
  └── handle_* 函数                    └── (MiniMaxClient + types.rs)
─────────────────────────────      ─────────────────────────
src/tools/video/mod.rs             src/providers/minimax/video.rs
  ├── VideoProvider trait              └── impl VideoProvider
  └── handle_* 函数
─────────────────────────────      ─────────────────────────
        ... (9 个能力 trait)                    ...
```

- **接口与 handler 在一起**（`tools/xxx/mod.rs`），改接口时不用跨目录跳
- **供应商在独立目录**（`providers/<name>/`），新增供应商只需开目录 + 实现 trait
- **`provider.toml`** 按能力独立选择供应商，API key 只在环境变量

## 功能

### AI 生成

- **text_to_audio** — 文本转语音（同步）
- **text_to_audio_stream** — WebSocket 流式文本转语音
- **generate_audio_async** — 异步文本转语音（最长 5 万字符）
- **query_audio_task** — 查询异步 TTS 任务状态
- **generate_image** — 图像生成
- **generate_video** — 视频生成（异步/同步两种模式）
- **query_video** — 查询视频任务状态
- **generate_video_agent** — 视频 Agent 模板生成
- **query_video_agent** — 查询视频 Agent 任务状态
- **generate_music** — 音乐生成
- **generate_lyrics** — 歌词生成
- **generate_music_cover** — 音乐翻唱

### 音色管理

- **list_voices** — 列出可用音色（系统/克隆/AI 设计）
- **voice_clone** — 音色克隆
- **voice_design** — 通过文字 prompt 设计新音色
- **delete_voice** — 删除指定音色

### 对话与搜索

- **chat** — 文本对话（Anthropic 兼容接口，支持 coding-plan-vlm/search 等模型）
- **web_search** — 网络搜索
- **understand_image** — 图片理解

### 账户与文件

- **query_usage** — 查询账户余额
- **list_files** — 列出平台文件
- **retrieve_file** — 获取文件信息与下载链接
- **delete_file** — 删除平台文件

### Subagent（子智能体）

- **run_subagent** — 运行具名 subagent，支持递归组合。可通过 `allowed_tools` 参数
  运行时覆盖工具白名单（不传则用 JSON 配置）
- **list_subagents** — 列出所有已加载 subagent
- **get_subagent** — 查看 subagent 完整配置

Subagent 定义在 `subagents/<name>.json` 中，启动时加载。

## 快速开始

### 1. 环境变量

```bash
export MINIMAX_API_KEY="your-api-key"
export MINIMAX_API_HOST="https://api.minimaxi.com"  # 可选，默认中国区
```

### 2. Provider 配置（可选）

项目根目录创建 `provider.toml`（不创建则全部默认 minimax）：

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

- **API key 不在配置文件** — `api_key_env` 告知从哪个环境变量读取
- 每个能力可独立选择供应商（如 `tts = "minimax"`, `chat = "openai"`）

### 3. 构建

```bash
cargo build --release
```

构建产物：
| 产物 | 路径 |
|------|------|
| 库 | `minimax_api`（lib） |
| MCP server | `target/release/Subagent-mcp` |
| CLI | `target/release/Subagent_cli` |

### 4. CLI 测试

```bash
./target/release/Subagent_cli list_voices
./target/release/Subagent_cli query_usage
./target/release/Subagent_cli text_to_audio "你好"
./target/release/Subagent_cli web_search "关键词"
```

### 5. Claude Code 集成

编辑 `~/.claude/settings.local.json`：

```json
{
  "mcpServers": {
    "Subagent-mcp": {
      "command": "项目路径/target/release/Subagent-mcp"
    }
  }
}
```

`MINIMAX_API_KEY` 环境变量自动从 shell 继承，无需在配置中指定。

修改代码后重新编译并重启 Claude Code：

```bash
cargo build --release
pkill -f Subagent-mcp
# 重启 Claude Code
```

## 添加新供应商

1. 创建 `src/providers/<name>/mod.rs`，封装供应商客户端
2. 实现需要的能力 trait（`src/providers/<name>/{tts,video,...}.rs`）
3. 在 `provider.toml` 添加 `[provider_config.<name>]`
4. 更新 `src/providers/mod.rs` 的 `create_provider_set()` factory

## 目录结构

```
Subagent_tools/
├── src/
│   ├── bin/main_cli.rs       # CLI 入口
│   ├── main.rs                # MCP 服务器入口
│   ├── subagent_impl.rs       # 工具工厂注册 + run_subagent 构建器
│   ├── client.rs              # MiniMaxClient（MiniMaxProvider 内部使用）
│   ├── consts.rs              # 常量定义
│   ├── types.rs               # MiniMax API 请求/响应类型
│   ├── utils.rs               # Hex 解码、文件保存、图片处理
│   ├── ws_client.rs           # WebSocket 客户端
│   ├── error.rs               # 错误类型
│   ├── lib.rs                 # 库入口 (crate: minimax_api)
│   ├── mcp_params.rs          # MCP 工具参数定义
│   ├── tools/                 # Trait 定义 + handler 函数
│   │   ├── tts/mod.rs         #   TtsProvider + VoiceProvider
│   │   ├── video/mod.rs       #   VideoProvider
│   │   ├── image/mod.rs       #   ImageProvider
│   │   ├── music/mod.rs       #   MusicProvider
│   │   ├── chat/mod.rs        #   ChatProvider
│   │   ├── search/mod.rs      #   SearchProvider
│   │   ├── files/mod.rs       #   FileProvider
│   │   ├── usage/mod.rs       #   UsageProvider
│   │   └── subagent.rs        #   Subagent handler（binary-only）
│   ├── providers/             # 通用类型 + 供应商实现
│   │   ├── mod.rs             #   MediaOutput, ProviderError, factory
│   │   └── minimax/           #   MiniMax 供应商
│   │       ├── tts.rs, video.rs, image.rs, music.rs
│   │       ├── chat.rs, search.rs, files.rs, usage.rs
│   │       └── mod.rs
│   └── subagent/              # Agent loop 框架（库）
│       ├── types.rs            #   SubagentDef, LoopResult, DispatchResult
│       ├── registry.rs         #   加载 subagents/*.json
│       ├── loop_runner.rs      #   run_agent_loop()
│       ├── agent_tool.rs       #   AgentTool（自包含，schema + execute 一体）
│       └── factory.rs          #   ToolRegistry, tools_for_subagent()
├── subagents/                 # 用户定义的 subagent JSON
├── scripts/
│   ├── bump.sh                # 版本号递增
│   └── sync_docs.sh           # 同步官方 API 文档
├── docs/provider-Minimax/     # MiniMax API 文档缓存
├── Cargo.toml
├── provider.toml              # Provider 配置（可选）
├── Makefile
└── VERSION
```

## 故障排除

### MCP 服务器无响应

```bash
source ~/.zshrc
echo $MINIMAX_API_KEY  # 确认有值
```

### chat 返回 404

Token Plan key 需要 `/anthropic` 前缀（代码中已配置）。

### voice_design / voice_clone 返回 API error 1008

账户余额不足，需要充值。
