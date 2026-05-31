# MiniMax Rust MCP Server

Rust 实现的 MiniMax API MCP 服务器，通过 Model Context Protocol 提供 AI 工具。

## 功能

- **chat** — Anthropic 兼容接口文本对话
- **web_search** — 网络搜索 (Token Plan)
- **understand_image** — 图片理解 (Token Plan)
- **text_to_audio** — 文本转语音
- **text_to_audio_stream** — WebSocket 流式文本转语音
- **generate_audio_async** — 异步文本转语音（最长5万字符）
- **query_audio_task** — 查询异步 TTS 任务状态
- **list_voices** — 列出可用音色
- **voice_clone** — 音色克隆
- **voice_design** — 音色设计
- **delete_voice** — 删除指定音色
- **query_usage** — 查询账户余额
- **generate_image** — 图像生成
- **generate_video** — 视频生成
- **query_video** — 查询视频任务状态
- **generate_music** — 音乐生成
- **generate_lyrics** — 歌词生成
- **generate_music_cover** — 音乐翻唱
- **generate_video_agent** — 视频Agent任务
- **query_video_agent** — 查询视频Agent任务状态
- **list_files** — 列出平台文件
- **retrieve_file** — 获取文件信息
- **delete_file** — 删除平台文件

## 快速开始

### 1. 环境变量

确保 `~/.zshrc` 中已设置：

```bash
export MINIMAX_API_KEY="sk-cp-xxxxx"  # 你的 Token Plan API Key
export MINIMAX_API_HOST="https://api.minimaxi.com"  # 国内
```

加载环境变量：

```bash
source ~/.zshrc
```

### 2. 构建

```bash
cargo build --release
```

二进制文件位置：`target/release/minimax-mcp`

### 3. CLI 测试

```bash
source ~/.zshrc

# 测试 MCP 服务器（发送 JSON-RPC 消息）
./target/release/minimax-mcp
```

交互式测试需要发送 JSON-RPC 消息。例如，使用 `socat` 或 `nc`：

```bash
# 初始化 + 调用工具
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {"name": "test", "version": "1.0"}
  }
}
{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}
```

### 4. Claude Code 集成

Claude Code 会自动从 `~/.claude/settings.local.json` 加载 MCP 配置。

#### 方式一：自动加载（当前已配置）

查看当前配置：

```bash
claude mcp list
```

#### 方式二：手动配置

编辑 `~/.claude/settings.local.json`：

```json
{
  "mcpServers": {
    "minimax": {
      "command": "/Users/yyurk/my_project/minimax_agent/target/release/minimax-mcp"
    }
  }
}
```

**注意**：`MINIMAX_API_KEY` 环境变量会自动从 shell 环境继承，无需在配置中指定。

### 5. 重启 MCP 服务器

修改代码后需要重新编译，然后重启 Claude Code：

```bash
cargo build --release
# 退出 Claude Code 并重新进入
```

## API Key 说明

Token Plan API Key (`sk-cp-xxxxx`) 支持：

- `/v1/coding_plan/search` — 搜索
- `/v1/coding_plan/vlm` — 图片理解
- `/v1/messages` — 文本对话 (通过 `/anthropic/v1/messages`)
- 其他 MiniMax API 端点

API Key 通过 `MiniMaxClient::from_env()` 自动从环境变量读取，**不需要**在配置文件中明文存储。

## 目录结构

```
minimax_agent/
├── src/
│   ├── main.rs        # MCP 服务器入口
│   ├── client.rs      # API 客户端
│   ├── types.rs       # 请求/响应类型
│   ├── consts.rs      # 常量定义
│   ├── utils.rs       # 工具函数
│   ├── error.rs       # 错误类型
│   ├── lib.rs         # 库入口
│   ├── mcp_params.rs  # MCP 工具参数定义
│   ├── ws_client.rs   # WebSocket 客户端
│   └── tools/         # 工具实现
│       ├── chat.rs    # 文本对话
│       ├── search.rs  # 搜索和图片理解
│       ├── tts.rs     # 语音合成
│       ├── video.rs   # 视频生成
│       ├── image.rs   # 图像生成
│       ├── music.rs   # 音乐生成
│       ├── files.rs   # 文件管理
│       └── usage.rs   # 账户查询
├── scripts/
│   └── bump.sh        # 版本号递增脚本
├── docs/              # MiniMax API 文档
├── Cargo.toml         # 项目配置
├── Makefile           # 构建/发布命令
└── VERSION            # 版本号文件
```

## 故障排除

### MCP 服务器无响应

确保环境变量已加载：

```bash
source ~/.zshrc
echo $MINIMAX_API_KEY  # 确认有值
```

### chat 返回 404

检查是否使用了正确的端点。Token Plan key 需要 `/anthropic` 前缀（代码中已配置）。

### 余额不足 (error 1008)

`voice_clone` 和 `voice_design` 需要较大账户余额，不建议使用。