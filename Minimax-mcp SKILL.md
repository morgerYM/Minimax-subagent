---
name: Minimax-mcp
description: MiniMax MCP tools for text/speech/video/image/music generation. Triggers: "MiniMax", "MCP tools", "generate speech/video/image/music", "text to audio", "voice clone", "TTS", "voice design"
metadata:
  type: skill
---

# MiniMax MCP Tools

Text-to-speech, video, image, music generation via MiniMax API.

## Text-to-Speech (语音合成)

```
text_to_audio text="你好世界" voice_id="Portuguese_LovelyLady"
  → 短文本同步 TTS，返回音频数据或下载链接
  → 支持 emotion: happy/sad/angry/fearful/disgusted/surprised/calm/fluent/whisper

text_to_audio_stream text="实时语音" voice_id="Portuguese_LovelyLady"
  → WebSocket 流式 TTS，低延迟首包
  → 支持 continuous_sound 模式（韵律更自然，speech-2.8-hd/turbo）

generate_audio_async text="很长很长的文本..." voice_id="Portuguese_LovelyLady"
  → 异步 TTS（最长5万字符 或 100万字符 via text_file_id），立即返回 task_id
  → 支持完整 9 种 emotion 和更多音频格式（wav/pcmu_raw/pcmu_wav/opus）

query_audio_task task_id="xxx" output_directory="/path/to/save"
  → 查询异步任务，完成后自动下载并解压 mp3
```

## Voice Management (音色管理)

```
list_voices voice_type="all"
  → 列出所有音色（system/voice_cloning）

voice_clone voice_id="my_voice" file="/path/to/audio.wav" text="试听文本"
  → 克隆音色（上传参考音频）

voice_design prompt="温柔的女性声音" preview_text="你好"
  → 通过文字描述创建全新音色 ⚠️ 需要较大余额

delete_voice voice_id="xxx" voice_type="voice_cloning"
  → 删除指定音色
```

## Video Generation (视频生成)

```
generate_video prompt="一个女人在跳舞" model="MiniMax-Hailuo-2.3"
  → 异步生成视频，返回 task_id（默认）

query_video task_id="xxx" output_directory="/path/to/save"
  → 查询视频任务状态，完成后下载

generate_video_agent template_id="xxx" text_inputs=[{"value":"狮子"}]
  → 基于模板生成视频

query_video_agent task_id="xxx"
  → 查询视频 Agent 任务状态
```

## Image Generation (图像生成)

```
generate_image prompt="一只可爱的猫" aspect_ratio="1:1"
  → 生成图片

generate_image prompt="山水风景" style_type="watercolor" style_weight=0.8
  → 水彩风格图像

generate_image prompt="产品图" width=1024 height=1024 n=3
  → 指定尺寸和数量
```

## Music Generation (音乐生成)

```
generate_music prompt="欢快的流行音乐" lyrics="[Verse]你好世界[Verse]..." model="music-2.6"
  → 生成音乐，支持 [Intro][Verse][Chorus][Bridge][Outro] 标签

generate_lyrics prompt="关于爱情的歌词" title="爱情之歌"
  → 生成完整歌词

generate_music_cover audio_url="https://example.com/audio.mp3"
  → 生成翻唱版本（参考音频 URL）
```

## Chat/AI (聊天)

```
chat prompt="你好" model="MiniMax-M3"
  → 标准对话（默认模型，1M 上下文窗口）

chat prompt="分析这张图片" model="coding-plan-vlm" image_source="/path/to/image.png"
  → VLM 模型，分析图片

chat prompt="搜索最新新闻" model="coding-plan-search"
  → 搜索模型，网络检索

# 也支持其他模型
chat prompt="..." model="MiniMax-M2.5"
  → 兼容旧模型
```

## Search/Vision (搜索/视觉)

```
web_search query="Python 教程 2024"
  → 网络搜索，返回结果和建议

understand_image image_source="/path/to/image.png" prompt="描述图片内容"
understand_image image_source="https://example.com/image.jpg" prompt="图片里有什么"
  → 图片理解（支持本地路径或 HTTP URL）
```

## File Management (文件管理)

```
list_files purpose="voice_clone"
  → 列出文件（purpose: voice_clone/prompt_audio/t2a_async_input）

retrieve_file file_id=123
  → 获取文件信息和下载链接

delete_file file_id=123 purpose="voice_clone"
  → 删除文件
```

## Account (账户)

```
query_usage
  → 查询 Token 余额和 API 用量
```

## Subagent (子智能体)

Subagent 是一个**带预设 system prompt 的 LLM agent**,可以调用任意 MCP 工具,并能**递归**调用其他 subagent。
Subagent 定义在项目根目录的 `subagents/<name>.json` 中(用户编辑),启动时加载。

```
list_subagents
  → 列出所有已加载的 subagent(name + description)

get_subagent name="video-creator"
  → 查看 subagent 完整配置(system / 允许的工具 / 限额)

run_subagent name="video-creator" task="用 happy 情感读 'Hello'"
  → 运行 subagent,返回最终文本输出 + 完整 tool 调用历史(含递归 subagent 调用链)
```

**示例 subagent 配置** `subagents/echo.json`:

```json
{
  "name": "echo",
  "description": "回声测试 agent",
  "system": "你回显用户的输入,不要调用任何工具。",
  "model": "MiniMax-M3",
  "max_tokens": 4096,
  "max_iterations": 5,
  "allowed_tools": []
}
```

**递归示例** `subagents/orchestrator.json` 委派给 `subagents/worker.json`:

```json
{
  "name": "orchestrator",
  "description": "把任务拆给 worker",
  "system": "调用 run_subagent(name=\"worker\", task=<用户原始任务>)后,直接回复 worker 的输出。"
}
```

⚠️ `run_subagent` 本身**总是隐式可用**给所有 subagent,即使 `allowed_tools` 没有列出 — 这是递归组合的关键。

---

## Tips

- **播放音频**: `afplay /path/to/file.mp3`
- **推荐音色**: Portuguese_LovelyLady（女声）、female-yujie（御姐）、female-shaonv（默认）
- **⚠️ 注意**: voice_design/voice_clone 需要较大账户余额，余额不足报 API error 1008
- **图片理解**: 本地文件会自动转换为 base64，无需手动处理
- **TTS 模型版本**: 默认 `speech-2.8-hd`；`fluent`/`whisper` emotion 仅 `speech-2.6-*` 模型支持；`continuous_sound` 仅 `speech-2.8-hd/turbo` 支持
- **chat 模型**: 默认 `MiniMax-M3`（1M 上下文）；如需 VLM/搜索使用 `coding-plan-vlm` / `coding-plan-search`

## Latest API Updates (2026)

- **chat**: 默认模型升级为 `MiniMax-M3`（支持 1M 上下文窗口，最大输出 16,384 tokens）
- **TTS**: 同步接口支持完整 9 种 emotion（happy/sad/angry/fearful/disgusted/surprised/calm/fluent/whisper）
- **TTS**: 音频格式扩展为 `mp3/pcm/flac/wav/pcmu_raw/pcmu_wav/opus`
- **TTS**: 异步接口单次最长 5 万字符，通过 `text_file_id` 可支持 100 万字符
- **video**: 支持 `MiniMax-Hailuo-2.3`（默认）/`MiniMax-Hailuo-02`，02 模型支持 6/10 秒时长和 768P/1080P 分辨率