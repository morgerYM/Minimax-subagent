# MiniMax 开放平台 API 参考文档

## 目录

- [概述](#概述)
- [获取 API Key](#获取-api-key)
- [API 端点](#api-端点)
- [文本生成 API](#文本生成-api)
- [文本对话 API](#文本对话-api)
- [Token Plan MCP 工具](#token-plan-mcp-工具)
- [语音合成 API](#语音合成-api)
- [视频生成 API](#视频生成-api)
- [图像生成 API](#图像生成-api)
- [音乐生成 API](#音乐生成-api)
- [歌词生成 API](#歌词生成-api)
- [文件管理 API](#文件管理-api)
- [Prompt 缓存](#prompt-缓存)
- [Anthropic API 兼容](#anthropic-api-兼容)
- [OpenAI API 兼容](#openai-api-兼容)
- [速率限制](#速率限制)
- [错误码](#错误码)
- [常见问题](#常见问题)

---

## 概述

MiniMax 开放平台提供文本、语音、视频、图像、音乐五大方向的 API 服务，支持通过 HTTP 请求、Anthropic SDK（推荐）或 OpenAI SDK 接入。

**API 基础 URL：**
- 国内用户：`https://api.minimaxi.com`
- 国际用户：`https://api.minimax.io`

---

## 获取 API Key

### 按量付费
1. 进入 **账户管理 > 接口密钥**
2. 创建新的 API Key

### Token Plan
1. 进入 **订阅管理 > Token Plan**
2. 创建 Token Plan API Key

> **注意**：API Key 是调用接口的重要凭证，请勿与他人共享或暴露在客户端代码中。

---

## API 端点

### 文本类

| 接口 | 方法 | 端点 | 说明 |
|------|------|------|------|
| 文本对话 | POST | `/v1/chat/completions` | OpenAI 兼容接口 |
| 文本合成 | POST | `/v1/text/chat` | MiniMax 文本对话 |
| Anthropic 兼容 | POST | `/v1/messages` | Anthropic SDK 接口 |

### 语音类

| 接口 | 方法 | 端点 | 说明 |
|------|------|------|------|
| 同步语音合成 | POST | `/v1/t2a_v2` | HTTP 同步语音合成 |
| 同步语音合成 | WSS | `/v1/t2a_v2` | WebSocket 流式输出 |
| 异步语音合成 | POST | `/v1/t2a_async` | 长文本异步语音合成 |
| 查询语音任务 | GET | `/v1/tasks/{task_id}` | 查询异步任务状态 |
| 音色快速复刻 | POST | `/v1/voice_cloning` | 声音克隆 |
| 音色设计 | POST | `/v1/voice_design` | 基于描述生成音色 |

### 视频类

| 接口 | 方法 | 端点 | 说明 |
|------|------|------|------|
| 文生视频 | POST | `/v1/video_generation` | 文本生成视频 |
| 图生视频 | POST | `/v1/video_generation` | 图片生成视频 |
| 首尾帧视频 | POST | `/v1/video_generation` | 首尾帧生成视频 |
| 主体参考视频 | POST | `/v1/video_generation` | 主体参考生成视频 |
| 查询视频任务 | GET | `/v1/tasks/{task_id}` | 查询视频生成状态 |
| 视频下载 | GET | `/v1/files/{file_id}` | 下载生成的视频 |

### 图像类

| 接口 | 方法 | 端点 | 说明 |
|------|------|------|------|
| 文生图 | POST | `/v1/image_generation` | 文本生成图像 |
| 图生图 | POST | `/v1/image_generation` | 参考图生成图像 |

### 音乐类

| 接口 | 方法 | 端点 | 说明 |
|------|------|------|------|
| 音乐生成 | POST | `/v1/music_generation` | 根据描述和歌词生成音乐 |
| 歌词生成 | POST | `/v1/lyrics_generation` | 生成歌曲歌词 |

### 文件类

| 接口 | 方法 | 端点 | 说明 |
|------|------|------|------|
| 上传文件 | POST | `/v1/files/upload` | 上传文件 |
| 列出文件 | GET | `/v1/files` | 列出已上传文件 |
| 检索文件 | GET | `/v1/files/{file_id}` | 获取文件信息 |
| 下载文件 | GET | `/v1/files/{file_id}/download` | 下载文件 |
| 删除文件 | POST | `/v1/files/{file_id}/delete` | 删除文件 |

---

## 文本生成 API

### 支持模型

| 模型名称 | 上下文窗口 | 介绍 | 输出速度 |
|----------|------------|------|----------|
| MiniMax-M2.7 | 204,800 tokens | 开启模型的自我迭代 | ~60 TPS |
| MiniMax-M2.7-highspeed | 204,800 tokens | 与 M2.7 效果不变，速度大幅提升 | ~100 TPS |
| MiniMax-M2.5 | 204,800 tokens | 顶尖性能与极致性价比 | ~60 TPS |
| MiniMax-M2.5-highspeed | 204,800 tokens | 与 M2.5 效果不变，速度大幅提升 | ~100 TPS |
| MiniMax-M2.1 | 204,800 tokens | 强大多语言编程能力 | ~60 TPS |
| MiniMax-M2.1-highspeed | 204,800 tokens | 极速版 | ~100 TPS |
| MiniMax-M2 | 204,800 tokens | 专为高效编码与Agent工作流而生 | - |

### Anthropic SDK 调用（推荐）

```python
# 安装 SDK
pip install anthropic

# 设置环境变量
export ANTHROPIC_BASE_URL=https://api.minimaxi.com/anthropic
export ANTHROPIC_API_KEY=${YOUR_API_KEY}

# 调用示例
import anthropic

client = anthropic.Anthropic()

message = client.messages.create(
    model="MiniMax-M2.7",
    max_tokens=1024,
    system="You are a helpful assistant.",
    messages=[
        {
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": "Hi, how are you?"
                }
            ]
        }
    ]
)

for block in message.content:
    if block.type == "thinking":
        print(f"Thinking:\n{block.thinking}\n")
    elif block.type == "text":
        print(f"Text:\n{block.text}\n")
```

### OpenAI SDK 调用

```python
# 安装 SDK
pip install openai

# 设置环境变量
export OPENAI_BASE_URL=https://api.minimaxi.com/v1
export OPENAI_API_KEY=${YOUR_API_KEY}

# 调用示例
from openai import OpenAI

client = OpenAI()

response = client.chat.completions.create(
    model="MiniMax-M2.7",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"}
    ],
    max_tokens=1024
)

print(response.choices[0].message.content)
```

---

## 文本对话 API

### M2-her 模型

专为对话场景优化的文本模型，支持丰富的角色设定和对话历史管理能力。

**上下文窗口**：64K

### 消息角色类型

#### 基础角色

| 角色类型 | 说明 | 使用场景 |
|----------|------|----------|
| system | 设定模型的角色和行为 | 定义 AI 的身份、性格、知识范围等 |
| user | 用户的输入 | 用户发送的消息 |
| assistant | 模型的历史回复 | AI 之前的回复，用于多轮对话 |

#### 高级角色

| 角色类型 | 说明 | 使用场景 |
|----------|------|----------|
| user_system | 设定用户的角色和人设 | 角色扮演场景中定义用户身份 |
| group | 对话的名称 | 标识对话分组或场景名称 |
| sample_message_user | 示例的用户输入 | 提供用户消息的示例 |
| sample_message_ai | 示例的模型输出 | 提供期望的 AI 回复示例 |

### 调用示例

```python
from openai import OpenAI

client = OpenAI()
client.base_url = "https://api.minimaxi.com/v1"
client.api_key = "${YOUR_API_KEY}"

response = client.chat.completions.create(
    model="M2-her",
    messages=[
        {
            "role": "system",
            "name": "AI助手",
            "content": "你是一个友好、专业的AI助手"
        },
        {
            "role": "user",
            "name": "用户",
            "content": "你好，请介绍一下你自己"
        }
    ],
    temperature=1.0,
    top_p=0.95,
    max_completion_tokens=2048
)

print(response.choices[0].message.content)
```

### 角色扮演示例

```python
messages = [
    {
        "role": "system",
        "content": "你是《三国演义》中的诸葛亮，智慧、沉稳、善于谋略"
    },
    {
        "role": "user_system",
        "content": "你是一位来自现代的穿越者"
    },
    {
        "role": "group",
        "content": "三国时期的隆中对话"
    },
    {
        "role": "user",
        "content": "军师，我有一些现代的想法想和您探讨"
    }
]
```

---
## Token Plan MCP 工具

> Token Plan MCP 是 Token Plan 订阅用户专属的工具集，提供**网络搜索**和**图片理解**两个能力，助力编码过程中快速获取信息和理解图片内容。

### 支持工具

| 工具名称 | 功能 | 说明 |
|----------|------|------|
| `web_search` | 网络搜索 | 根据搜索查询词进行网络搜索，返回搜索结果和相关搜索建议 |
| `understand_image` | 图片理解 | 对图片进行理解和分析，支持多种图片输入方式 |

### 工具参数

#### web_search

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| query | string | ✓ | 搜索查询词 |

#### understand_image

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| prompt | string | ✓ | 对图片的提问或分析要求 |
| image_url | string | ✓ | 图片来源，支持 HTTP/HTTPS URL 或本地文件路径 |

**支持格式**：JPEG、PNG、GIF、WebP（最大 20MB）

### 安装配置

```bash
# 安装 uvx（如果没有安装）
curl -LsSf https://astral.sh/uv/install.sh | sh

# Claude Code 中配置
claude mcp add -s user MiniMax --env MINIMAX_API_KEY=你的API密钥 --env MINIMAX_API_HOST=https://api.minimaxi.com -- uvx minimax-coding-plan-mcp -y
```

### Claude Code 配置示例

编辑 `~/.claude.json`，添加以下 MCP 配置：

```json
{
  "mcpServers": {
    "MiniMax": {
      "command": "uvx",
      "args": ["minimax-coding-plan-mcp", "-y"],
      "env": {
        "MINIMAX_API_KEY": "你的API密钥",
        "MINIMAX_API_HOST": "https://api.minimaxi.com"
      }
    }
  }
}
```

### OpenCode 配置示例

编辑 `~/.config/opencode/opencode.json`：

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "MiniMax": {
      "type": "local",
      "command": ["uvx", "minimax-coding-plan-mcp", "-y"],
      "environment": {
        "MINIMAX_API_KEY": "你的API密钥",
        "MINIMAX_API_HOST": "https://api.minimaxi.com"
      },
      "enabled": true
    }
  }
}
```

### 验证配置

进入 Claude Code 或 OpenCode 后输入 `/mcp`，能看到 `web_search` 和 `understand_image`（Claude Code）或 `MiniMax connected`（OpenCode），说明配置成功。

---


## 语音合成 API

### 支持模型

| 模型 | 特性 |
|------|------|
| speech-2.8-hd | 最新 HD 模型，精准还原真实语气细节，全面提升音色相似度 |
| speech-2.8-turbo | 最新 Turbo 模型，极速响应，语气表达生动自然 |
| speech-2.6-hd | 极致音质与韵律表现，生成更快更自然 |
| speech-2.6-turbo | 音质优异，超低时延，响应更灵敏 |
| speech-02-hd | 出色的韵律和稳定性，复刻相似度和音质表现突出 |
| speech-02-turbo | 小语种能力增强，性能表现出色 |
| speech-01-hd | 语音 HD 模型，复刻相似度和音质表现突出 |
| speech-01-turbo | 语音 Turbo 模型，小语种能力增强 |

### 支持语言（40种）

中文、粤语、英语、西班牙语、法语、俄语、德语、葡萄牙语、阿拉伯语、意大利语、日语、韩语、印尼语、越南语、土耳其语、荷兰语、乌克兰语、泰语、波兰语、罗马尼亚语、希腊语、捷克语、芬兰语、印地语、保加利亚语、丹麦语、希伯来语、马来语、波斯语、斯洛伐克语、瑞典语、克罗地亚语、菲律宾语、匈牙利语、挪威语、斯洛文尼亚语、加泰罗尼亚语、尼诺斯克语、泰米尔语、阿非利卡语

### 同步语音合成

```python
import requests

url = "https://api.minimaxi.com/v1/t2a_v2"
headers = {
    "Authorization": f"Bearer ${YOUR_API_KEY}",
    "Content-Type": "application/json"
}
data = {
    "model": "speech-02-hd",
    "text": "你好，这是语音合成测试",
    "stream": False,
    "voice_setting": {
        "voice_id": "male-qn-qingse",
        "speed": 1.0,
        "volume": 1.0,
        "pitch": 0
    },
    "audio_setting": {
        "sample_rate": 32000,
        "bitrate": 128000,
        "format": "mp3"
    }
}

response = requests.post(url, headers=headers, json=data)
print(response.json())
```

### 异步长文本语音合成

单次最大支持 100 万字符，适合整本书籍等长文本。

```python
# 创建任务
url = "https://api.minimaxi.com/v1/t2a_async"
data = {
    "model": "speech-02-hd",
    "text": "长文本内容...",
    "voice_id": "male-qn-qingse"
}

response = requests.post(url, headers=headers, json=data)
task_id = response.json()["task_id"]

# 查询任务状态
status_url = f"https://api.minimaxi.com/v1/tasks/{task_id}"
status_response = requests.get(status_url, headers=headers)
print(status_response.json())
```

---

## 视频生成 API

### 支持模型

| 模型 | 功能 | 分辨率 | 时长 |
|------|------|------|------|
| MiniMax-Hailuo-2.3 | 全新视频生成模型，肢体动作、物理表现与指令遵循全面升级 | 768P / 1080P | 6s / 10s |
| MiniMax-Hailuo-2.3-Fast | 图生视频模型，生成速度大幅提升，更快更优惠 | 768P / 1080P | 6s / 10s |
| MiniMax-Hailuo-02 | 新一代视频生成模型，1080P 原生，SOTA 指令遵循，极致物理表现 | 512P / 768P / 1080P | 6s / 10s |
| T2V-01 | 文本生成视频基础模型 | 720P | 6s |
| T2V-01-Director | 文本生成视频导演版，支持运镜控制 | 720P | 6s |
| I2V-01 | 图像生成视频基础模型 | 720P | 6s |
| I2V-01-live | 图像生成视频，支持多种画风 | 720P | 6s |
| I2V-01-Director | 图像生成视频导演版，支持运镜控制 | 720P | 6s |
| S2V-01 | 主体参考视频生成，保持人物特征一致性 | 1080P | 6s |

### 文生视频

```python
url = "https://api.minimaxi.com/v1/video_generation"
data = {
    "model": "MiniMax-Hailuo-2.3",
    "prompt": "一只猫在草地上奔跑",
    "aspect_ratio": "16:9"
}

response = requests.post(url, headers=headers, json=data)
task_id = response.json()["task_id"]
```

### 图生视频

```python
url = "https://api.minimaxi.com/v1/video_generation"
data = {
    "model": "MiniMax-Hailuo-2.3-Fast",
    "prompt": "图片中的主体开始跳舞",
    "input_files": [{"file_id": "${FILE_ID}"}]
}

response = requests.post(url, headers=headers, json=data)
```

### 查询视频任务

```python
status_url = f"https://api.minimaxi.com/v1/tasks/{task_id}"
response = requests.get(status_url, headers=headers)

# 任务成功时返回
# {
#     "status": "success",
#     "file_id": "xxx",
#     "download_url": "https://..."
# }
```

---

## 图像生成 API

### 支持模型

| 模型名称 | 简介 |
|----------|------|
| image-01 | 图像生成模型，画面表现细腻，支持文生图、图生图 |
| image-01-live | 在 image-01 基础上额外支持手绘、卡通等多种画风设置 |

### 文生图

```python
url = "https://api.minimaxi.com/v1/image_generation"
data = {
    "model": "image-01",
    "prompt": "一只可爱的橘猫在阳光下打盹",
    "aspect_ratio": "1:1",
    "resolution": 1024
}

response = requests.post(url, headers=headers, json=data)
print(response.json())
```

### 图生图

```python
url = "https://api.minimaxi.com/v1/image_generation"
data = {
    "model": "image-01",
    "prompt": "将图片中的猫变成蓝色",
    "input_files": [{"file_id": "${FILE_ID}"}]
}

response = requests.post(url, headers=headers, json=data)
```

---

## 音乐生成 API

### 支持模型

| 模型名称 | 使用方法 | 说明 |
|----------|----------|------|
| music-2.6 | 文本生成音乐 | 以声传情：翻唱入心，器乐入魂 |
| music-2.6-free | 文本生成音乐（限免版） | music-2.6 的限免版本，RPM 较低 |
| music-cover | 基于参考音频生成翻唱版本 | 支持一步翻唱和两步翻唱（可修改歌词），支持风格迁移和自动歌词提取 |
| music-cover-free | 基于参考音频生成翻唱（限免版） | music-cover 的限免版本，RPM 较低 |

### 音乐生成

```python
url = "https://api.minimaxi.com/v1/music_generation"
data = {
    "model": "music-2.6",
    "prompt": "一首轻快的流行歌曲，讲述追逐梦想的故事",
    "lyrics": "歌词内容..."
}

response = requests.post(url, headers=headers, json=data)
task_id = response.json()["task_id"]
```

### 翻唱生成（music-cover）

```python
# 方式一：一步翻唱
data = {
    "model": "music-cover",
    "prompt": "改编为抒情的钢琴伴奏版本",
    "audio_url": "https://example.com/source.mp3",  # 参考音频 URL
    "lyrics": "可选，自定义歌词，如不传则自动从参考音频中提取"
}

# 方式二：两步翻唱（可修改歌词）
# 第一步：预处理获取 cover_feature_id
preprocess_url = "https://api.minimaxi.com/v1/music_cover_preprocess"
preprocess_data = {
    "audio_url": "https://example.com/source.mp3"
}
preprocess_response = requests.post(preprocess_url, headers=headers, json=preprocess_data)
cover_feature_id = preprocess_response.json()["cover_feature_id"]

# 第二步：使用 cover_feature_id 生成翻唱
data = {
    "model": "music-cover",
    "cover_feature_id": cover_feature_id,
    "lyrics": "修改后的歌词..."
}
```

### 参考音频要求

- 时长：6 秒至 6 分钟
- 大小：最大 50MB
- 格式：支持常见音频格式（mp3、wav、flac 等）

---

## 歌词生成 API

### 功能说明

使用本接口生成歌词，支持完整歌曲创作和歌词编辑/续写。

### 支持模式

| 模式 | 说明 |
|------|------|
| write_full_song | 写完整歌曲 |
| edit | 编辑/续写歌词 |

### 调用示例

```python
url = "https://api.minimaxi.com/v1/lyrics_generation"

# 生成完整歌曲歌词
data = {
    "mode": "write_full_song",
    "prompt": "一首关于夏日海边的轻快情歌"
}

response = requests.post(url, headers=headers, json=data)
result = response.json()
print(f"歌名: {result['song_title']}")
print(f"风格标签: {result['style_tags']}")
print(f"歌词: {result['lyrics']}")
```

### 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| song_title | string | 生成的歌名 |
| style_tags | string | 风格标签，逗号分隔 |
| lyrics | string | 生成的歌词，包含结构标签，可直接用于 music_generation 的 lyrics 参数 |

### 支持的结构标签（14种）

`[Intro]`, `[Verse]`, `[Pre-Chorus]`, `[Chorus]`, `[Hook]`, `[Drop]`, `[Bridge]`, `[Solo]`, `[Build-up]`, `[Instrumental]`, `[Breakdown]`, `[Break]`, `[Interlude]`, `[Outro]`

### 歌词编辑/续写

```python
# 编辑现有歌词
data = {
    "mode": "edit",
    "prompt": "将歌词改为更抒情的风格",
    "lyrics": "现有的歌词内容..."
}

response = requests.post(url, headers=headers, json=data)
```

---

## 文件管理 API

### 文件支持格式

| 类型 | 格式 |
|------|------|
| 文档 | pdf, docx, txt, jsonl |
| 音频 | mp3, m4a, wav |

### 容量限制

- 总容量：100GB
- 单个文档容量：512MB

### 上传文件

```python
url = "https://api.minimaxi.com/v1/files/upload"
headers["Content-Type"] = "multipart/form-data"

with open("file.mp3", "rb") as f:
    files = {"file": ("audio.mp3", f, "audio/mpeg")}
    response = requests.post(url, headers=headers, files=files)

file_id = response.json()["file_id"]
```

### 列出文件

```python
url = "https://api.minimaxi.com/v1/files"
response = requests.get(url, headers=headers)
print(response.json())
```

### 下载文件

```python
download_url = f"https://api.minimaxi.com/v1/files/{file_id}/download"
response = requests.get(download_url, headers=headers)
```

### 删除文件

```python
delete_url = f"https://api.minimaxi.com/v1/files/{file_id}/delete"
response = requests.post(delete_url, headers=headers)
```

---

## Prompt 缓存

Prompt 缓存可以有效降低延迟和成本。

### 功能特性

- **自动缓存**：自动识别重复的上下文内容，无需更改接口调用方式
- **降低成本**：命中缓存的输入 Token 以更低价格计费
- **提升速度**：减少重复内容的处理时间

### 适用场景

- 系统提示词复用
- 固定的工具清单
- 多轮对话历史

### 缓存对比

| 特性 | Prompt 缓存（被动缓存） | Anthropic 主动缓存 |
|------|------------------------|-------------------|
| 使用方式 | 自动识别重复内容并缓存 | 显式设置 cache_control |
| 写入费用 | 无额外计费 | 首次写入需额外计费 |
| 缓存过期 | 根据系统负载自动调整 | 5分钟，自动续期 |
| 支持模型 | M2.7, M2.5, M2.1 系列 | M2.7, M2.5, M2.1, M2 系列 |

### 计费示例

假设标准输入价格为 10 元/1M Token，缓存命中价格为 1 元/1M Token：

```
总输入 Token: 50000
缓存命中 Token: 45000
新增输入 Token: 5000
输出 Token: 1000

费用计算：
- 新增输入费用：5000 × 10/1000000 = 0.05 元
- 缓存费用：45000 × 1/1000000 = 0.045 元
- 输出费用：1000 × 40/1000000 = 0.04 元
- 总费用：0.135 元

相比无缓存（0.54 元），节省 75%
```

---

## Anthropic API 兼容

通过 Anthropic SDK 调用 MiniMax 模型。

### 安装

```bash
pip install anthropic
```

### 环境变量

```bash
export ANTHROPIC_BASE_URL=https://api.minimaxi.com/anthropic
export ANTHROPIC_API_KEY=${YOUR_API_KEY}
```

### 支持功能

- 流式输出
- Interleaved Thinking（交错思维链）
- 工具使用（Function Calling）
- 主动缓存（cache_control）

---

## OpenAI API 兼容

通过 OpenAI SDK 调用 MiniMax 模型。

### 安装

```bash
pip install openai
```

### 环境变量

```bash
export OPENAI_BASE_URL=https://api.minimaxi.com/v1
export OPENAI_API_KEY=${YOUR_API_KEY}
```

---

## 速率限制

详细的速率限制信息请参考 [速率限制页面](https://platform.minimaxi.com/docs/pricing/rate-limit)。

如需更高的资源保障，可通过 `api@minimaxi.com` 与商务联系。

---

## 错误码

如遇 API 调用问题，可通过以下方式联系技术支持：

- 邮箱：Model@minimaxi.com
- GitHub：提交 Issue

---

## 常见问题

### 如何获取 API Key？

前往 **账户管理 > 接口密钥** 创建按量计费 API Key，或前往 **订阅管理 > Token Plan** 创建 Token Plan API Key。

### 如何提高速率限制？

可前往速率限制页面查看具体内容。如需更高限制，通过 `api@minimaxi.com` 与商务联系。

### TPS 是如何计算的？

```
TPS = 输出 token 数量 / (最后一个 token 的生成时间 - 第一个 token 的生成时间)
```

### 如何使用声音复刻服务？

需要完成**个人实名认证**或**企业认证**后方可使用。请前往 **账户管理 > 账户信息** 完成认证。

---

## 相关链接

- [MiniMax 开放平台文档中心](https://platform.minimaxi.com/docs)
- [Anthropic SDK 文档](https://docs.anthropic.com/)
- [OpenAI SDK 文档](https://platform.openai.com/docs)
- [MiniMax GitHub](https://github.com/MiniMaxAI)
- 技术支持邮箱：Model@minimaxi.com
