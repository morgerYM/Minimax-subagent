> ## Documentation Index
> Fetch the complete documentation index at: https://platform.minimaxi.com/docs/llms.txt
> Use this file to discover all available pages before exploring further.

# 音乐生成 (Music Generation)

> 使用本接口，输入歌词和歌曲描述，进行歌曲生成。



## OpenAPI

````yaml /api-reference/music/api/openapi.json POST /v1/music_generation
openapi: 3.1.0
info:
  title: MiniMax Music Generation API
  description: >-
    MiniMax music generation API with support for creating music from text
    prompts and lyrics
  license:
    name: MIT
  version: 1.0.0
servers:
  - url: https://api.minimaxi.com
security:
  - bearerAuth: []
paths:
  /v1/music_generation:
    post:
      tags:
        - Music
      summary: Music Generation
      operationId: generateMusic
      parameters:
        - name: Content-Type
          in: header
          required: true
          description: 请求体的媒介类型，请设置为 `application/json`，确保请求数据的格式为 JSON
          schema:
            type: string
            enum:
              - application/json
            default: application/json
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/GenerateMusicReq'
        required: true
      responses:
        '200':
          description: ''
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/GenerateMusicResp'
components:
  schemas:
    GenerateMusicReq:
      type: object
      required:
        - model
      properties:
        model:
          type: string
          description: |-
            使用的模型名称。可选值：

            - `music-2.6`（推荐）：文本生成音乐，仅限 Token Plan 用户和付费用户使用，RPM 较高
            - `music-cover`：基于参考音频生成翻唱版本，仅限 Token Plan 用户和付费用户使用，RPM 较高
            - `music-2.6-free`：`music-2.6` 的限免版本，所有用户可通过 API Key 使用，RPM 较低
            - `music-cover-free`：`music-cover` 的限免版本，所有用户可通过 API Key 使用，RPM 较低
          enum:
            - music-2.6
            - music-cover
            - music-2.6-free
            - music-cover-free
        prompt:
          type: string
          description: >-
            音乐的描述，用于指定风格、情绪和场景。例如"流行音乐, 难过, 适合在下雨的晚上"。<br>注意：

            - `music-2.6` / `music-2.6-free` 纯音乐（`is_instrumental:
            true`）：必填，长度限制 [1, 2000] 个字符

            - `music-2.6` / `music-2.6-free`（非纯音乐）：可选，长度限制 [0, 2000] 个字符

            - `music-cover` / `music-cover-free`：必填，描述目标翻唱风格，长度限制 [10, 300] 个字符
          maxLength: 2000
        lyrics:
          type: string
          description: >-
            歌曲歌词，使用 `\n` 分隔每行。支持结构标签：`[Intro]`, `[Verse]`, `[Pre Chorus]`,
            `[Chorus]`, `[Interlude]`, `[Bridge]`, `[Outro]`, `[Post Chorus]`,
            `[Transition]`, `[Break]`, `[Hook]`, `[Build Up]`, `[Inst]`,
            `[Solo]`。

            <br>

            注意：

            - `music-2.6` / `music-2.6-free` 纯音乐（`is_instrumental: true`）：非必填

            - `music-2.6` / `music-2.6-free`（非纯音乐）：必填，长度限制 [1, 3500] 个字符

            - `music-cover` / `music-cover-free`：可选，如不传则通过 ASR 自动从参考音频中提取歌词，长度限制
            [10, 1000] 个字符

            - 当 `lyrics_optimizer: true` 且 `lyrics` 为空时，系统将根据 `prompt` 自动生成歌词
          minLength: 1
          maxLength: 3500
        stream:
          type: boolean
          description: 是否使用流式传输，默认为 `false`
          default: false
        output_format:
          type: string
          description: >-
            音频的返回格式，可选值为 `url` 或 `hex`，默认为 `hex`。当 `stream` 为 `true` 时，仅支持 `hex`
            格式。注意：url 的有效期为 24 小时，请及时下载
          enum:
            - url
            - hex
          default: hex
        audio_setting:
          $ref: '#/components/schemas/AudioSetting'
        aigc_watermark:
          type: boolean
          description: '是否在音频末尾添加水印，默认为 `false`。仅在非流式 (`stream: false`) 请求时生效'
        lyrics_optimizer:
          type: boolean
          description: |-
            是否根据 `prompt` 描述自动生成歌词。仅 `music-2.6` / `music-2.6-free` 支持。

            设为 `true` 且 `lyrics` 为空时，系统会根据 prompt 自动生成歌词。默认为 `false`
          default: false
        is_instrumental:
          type: boolean
          description: |-
            是否生成纯音乐（无人声）。仅 `music-2.6` / `music-2.6-free` 支持。

            设为 `true` 时，`lyrics` 字段非必填。默认为 `false`
          default: false
        audio_url:
          type: string
          description: >-
            参考音频的 URL 地址。仅用于 `music-cover` / `music-cover-free` 模型。`audio_url` 和
            `audio_base64` 必须且只能提供其中一个。与 `cover_feature_id` 互斥。


            参考音频要求：

            - 时长：6 秒至 6 分钟

            - 大小：最大 50 MB

            - 格式：支持常见音频格式（mp3、wav、flac 等）
        audio_base64:
          type: string
          description: >-
            Base64 编码的参考音频。仅用于 `music-cover` / `music-cover-free` 模型。`audio_url`
            和 `audio_base64` 必须且只能提供其中一个。与 `cover_feature_id` 互斥。


            参考音频要求：

            - 时长：6 秒至 6 分钟

            - 大小：最大 50 MB

            - 格式：支持常见音频格式（mp3、wav、flac 等）
        cover_feature_id:
          type: string
          description: >-
            [翻唱前处理](/api-reference/music-cover-preprocess) 接口返回的特征
            ID，用于**两步翻唱流程**，支持修改歌词后生成翻唱。


            仅用于 `music-cover` / `music-cover-free` 模型。与 `audio_url` 和
            `audio_base64` 互斥。


            - 传入时 `lyrics` 为必填（长度限制 [10, 1000] 个字符）

            - `cover_feature_id` 有效期为 24 小时

            - 相同音频内容返回相同的 `cover_feature_id`
      example:
        model: music-2.6
        prompt: 独立民谣,忧郁,内省,渴望,独自漫步,咖啡馆
        lyrics: |-
          [verse]
          街灯微亮晚风轻抚
          影子拉长独自漫步
          旧外套裹着深深忧郁
          不知去向渴望何处
          [chorus]
          推开木门香气弥漫
          熟悉的角落陌生人看
        audio_setting:
          sample_rate: 44100
          bitrate: 256000
          format: mp3
    GenerateMusicResp:
      type: object
      properties:
        data:
          $ref: '#/components/schemas/MusicData'
        base_resp:
          $ref: '#/components/schemas/BaseResp'
      example:
        data:
          audio: hex编码的音频数据
          status: 2
        trace_id: 04ede0ab069fb1ba8be5156a24b1e081
        extra_info:
          music_duration: 25364
          music_sample_rate: 44100
          music_channel: 2
          bitrate: 256000
          music_size: 813651
        analysis_info: null
        base_resp:
          status_code: 0
          status_msg: success
    AudioSetting:
      type: object
      description: 音频输出配置
      properties:
        sample_rate:
          type: integer
          description: 采样率。可选值：`16000`, `24000`, `32000`, `44100`
        bitrate:
          type: integer
          description: 比特率。可选值：`32000`, `64000`, `128000`, `256000`
        format:
          type: string
          description: 音频编码格式。
          enum:
            - mp3
            - wav
            - pcm
    MusicData:
      type: object
      properties:
        status:
          type: integer
          description: |-
            音乐合成状态：
            1: 合成中
            2: 已完成
        audio:
          type: string
          description: 当 `output_format` 为 `hex` 时返回，是音频文件的 16 进制编码字符串
    BaseResp:
      type: object
      description: 状态码及详情
      properties:
        status_code:
          type: integer
          description: |-
            状态码及其分别含义如下：

            `0`: 请求成功

            `1002`: 触发限流，请稍后再试

            `1004`: 账号鉴权失败，请检查 API-Key 是否填写正确

            `1008`: 账号余额不足

            `1026`: 图片描述涉及敏感内容

            `2013`: 传入参数异常，请检查入参是否按要求填写

            `2049`: 无效的api key

            更多内容可查看 [错误码查询列表](/api-reference/errorcode) 了解详情
        status_msg:
          type: string
          description: 具体错误详情
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: |-
        `HTTP: Bearer Auth`
         - Security Scheme Type: http
         - HTTP Authorization Scheme: Bearer API_key，用于验证账户信息，可在 [账户管理>接口密钥](https://platform.minimaxi.com/user-center/basic-information/interface-key) 中查看。

````


> ## Documentation Index
> Fetch the complete documentation index at: https://platform.minimaxi.com/docs/llms.txt
> Use this file to discover all available pages before exploring further.

# 翻唱前处理 (Music Cover Preprocess)

> 对参考音频进行预处理，提取音频特征和歌词，用于两步翻唱流程。



## OpenAPI

````yaml /api-reference/music/api/openapi.json POST /v1/music_cover_preprocess
openapi: 3.1.0
info:
  title: MiniMax Music Generation API
  description: >-
    MiniMax music generation API with support for creating music from text
    prompts and lyrics
  license:
    name: MIT
  version: 1.0.0
servers:
  - url: https://api.minimaxi.com
security:
  - bearerAuth: []
paths:
  /v1/music_cover_preprocess:
    post:
      tags:
        - Music
      summary: 翻唱前处理 (Music Cover Preprocess)
      operationId: coverPreprocess
      parameters:
        - name: Content-Type
          in: header
          required: true
          description: 请求体的媒介类型，请设置为 `application/json`，确保请求数据的格式为 JSON
          schema:
            type: string
            enum:
              - application/json
            default: application/json
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CoverPreprocessReq'
        required: true
      responses:
        '200':
          description: ''
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/CoverPreprocessResp'
components:
  schemas:
    CoverPreprocessReq:
      type: object
      required:
        - model
      properties:
        model:
          type: string
          description: 模型名称。必须为 `music-cover`。
          enum:
            - music-cover
        audio_url:
          type: string
          description: |-
            参考音频的 URL 地址。`audio_url` 和 `audio_base64` 必须且只能提供其中一个。

            参考音频要求：
            - 时长：6 秒至 6 分钟
            - 大小：最大 50 MB
            - 格式：支持常见音频格式（mp3、wav、flac 等）
        audio_base64:
          type: string
          description: |-
            Base64 编码的参考音频。`audio_url` 和 `audio_base64` 必须且只能提供其中一个。

            参考音频要求：
            - 时长：6 秒至 6 分钟
            - 大小：最大 50 MB
            - 格式：支持常见音频格式（mp3、wav、flac 等）
      example:
        model: music-cover
        audio_url: https://example.com/song.mp3
    CoverPreprocessResp:
      type: object
      properties:
        cover_feature_id:
          type: string
          description: >-
            预处理后的音频特征唯一标识，有效期 24 小时。将此 ID
            传入[音乐生成接口](/api-reference/music-generation)的 `cover_feature_id`
            参数以进行两步翻唱。


            相同音频内容会返回相同的 `cover_feature_id`（基于 MD5 去重）。
        formatted_lyrics:
          type: string
          description: >-
            通过 ASR 从参考音频中提取并格式化的歌词，包含 `[Verse]`、`[Chorus]`、`[Bridge]`
            等段落标签。你可以修改这些歌词后传入音乐生成接口。
        structure_result:
          type: string
          description: >-
            JSON
            字符串，包含歌曲结构分析结果，含段落类型（`intro`、`verse`、`chorus`、`bridge`、`outro`、`inst`、`silence`）及其起止时间戳（秒）。
        audio_duration:
          type: number
          format: double
          description: 参考音频的时长（秒）。
        trace_id:
          type: string
          description: 请求追踪 ID。
        base_resp:
          $ref: '#/components/schemas/BaseResp'
      example:
        cover_feature_id: a1b2c3d4e5f67890abcdef1234567890
        formatted_lyrics: |-
          [Verse 1]
          歌曲第一行
          歌曲第二行

          [Chorus]
          这是副歌部分
          高声歌唱
        structure_result: >-
          {"num_segments":4,"segments":[{"start":0,"end":15.5,"label":"intro"},{"start":15.5,"end":45.2,"label":"verse"},{"start":45.2,"end":75.0,"label":"chorus"},{"start":75.0,"end":90.0,"label":"outro"}]}
        audio_duration: 90
        trace_id: 061e5f144eb7f10b1fdde81126e24f91
        base_resp:
          status_code: 0
          status_msg: success
    BaseResp:
      type: object
      description: 状态码及详情
      properties:
        status_code:
          type: integer
          description: |-
            状态码及其分别含义如下：

            `0`: 请求成功

            `1002`: 触发限流，请稍后再试

            `1004`: 账号鉴权失败，请检查 API-Key 是否填写正确

            `1008`: 账号余额不足

            `1026`: 图片描述涉及敏感内容

            `2013`: 传入参数异常，请检查入参是否按要求填写

            `2049`: 无效的api key

            更多内容可查看 [错误码查询列表](/api-reference/errorcode) 了解详情
        status_msg:
          type: string
          description: 具体错误详情
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: |-
        `HTTP: Bearer Auth`
         - Security Scheme Type: http
         - HTTP Authorization Scheme: Bearer API_key，用于验证账户信息，可在 [账户管理>接口密钥](https://platform.minimaxi.com/user-center/basic-information/interface-key) 中查看。

````

> ## Documentation Index
> Fetch the complete documentation index at: https://platform.minimaxi.com/docs/llms.txt
> Use this file to discover all available pages before exploring further.

# 歌词生成 (Lyrics Generation)

> 使用本接口生成歌词，支持完整歌曲创作和歌词编辑/续写。



## OpenAPI

````yaml /api-reference/music/lyrics/api/openapi.json POST /v1/lyrics_generation
openapi: 3.1.0
info:
  title: MiniMax Lyrics Generation API
  description: MiniMax 歌词生成 API，支持完整歌曲创作和歌词编辑/续写
  license:
    name: MIT
  version: 1.0.0
servers:
  - url: https://api.minimaxi.com
security:
  - bearerAuth: []
paths:
  /v1/lyrics_generation:
    post:
      tags:
        - Music
      summary: 歌词生成
      operationId: generateLyrics
      parameters:
        - name: Content-Type
          in: header
          required: true
          description: 请求体的媒介类型，请设置为 `application/json`，确保请求数据的格式为 JSON
          schema:
            type: string
            enum:
              - application/json
            default: application/json
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/GenerateLyricsReq'
        required: true
      responses:
        '200':
          description: 成功响应
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/GenerateLyricsResp'
components:
  schemas:
    GenerateLyricsReq:
      type: object
      required:
        - mode
      properties:
        mode:
          type: string
          description: 生成模式。<br>`write_full_song`：写完整歌曲<br>`edit`：编辑/续写歌词
          enum:
            - write_full_song
            - edit
        prompt:
          type: string
          description: 提示词/指令，用于描述歌曲主题、风格或编辑方向。为空时随机生成。
          maxLength: 2000
        lyrics:
          type: string
          description: 现有歌词内容，仅在 `edit` 模式下有效。可用于续写或修改已有歌词。
          maxLength: 3500
        title:
          type: string
          description: 歌曲标题。传入后输出将保持该标题不变。
      example:
        mode: write_full_song
        prompt: 一首关于夏日海边的轻快情歌
    GenerateLyricsResp:
      type: object
      properties:
        song_title:
          type: string
          description: 生成的歌名。若请求传入 `title` 则保持一致。
        style_tags:
          type: string
          description: 风格标签，逗号分隔。例如：`Pop, Upbeat, Female Vocals`
        lyrics:
          type: string
          description: >-
            生成的歌词，包含结构标签。可直接用于[音乐生成接口](/api-reference/music-generation)的
            `lyrics` 参数生成歌曲。<br>支持的结构标签（14种）：`[Intro]`, `[Verse]`,
            `[Pre-Chorus]`, `[Chorus]`, `[Hook]`, `[Drop]`, `[Bridge]`,
            `[Solo]`, `[Build-up]`, `[Instrumental]`, `[Breakdown]`, `[Break]`,
            `[Interlude]`, `[Outro]`
        base_resp:
          $ref: '#/components/schemas/BaseResp'
      example:
        song_title: 夏日海风的约定
        style_tags: Mandopop, Summer Vibe, Romance, Lighthearted, Beach Pop
        lyrics: |-
          [Intro]
          (Ooh-ooh-ooh)
          (Yeah)
          阳光洒满了海面

          [Verse 1]
          海风轻轻吹拂你发梢
          Smiling face, like a summer dream
          浪花拍打着脚边
          Leaving footprints, you and me
          沙滩上留下我们的笑
          Every moment, a sweet melody
          看着你眼中的闪耀
          Like the stars in the deep blue sea

          [Pre-Chorus]
          你说这感觉多么奇妙
          (So wonderful)
          想要永远停留在这一秒
          (Right here, right now)
          心跳加速，像海浪在奔跑

          [Chorus]
          Oh, 夏日的海边，我们的约定
          阳光下，你的身影，如此动听
          微风吹散了烦恼，只留下甜蜜
          这瞬间，只想和你，永远在一起
          (永远在一起)

          [Verse 2]
          ...
        base_resp:
          status_code: 0
          status_msg: success
    BaseResp:
      type: object
      description: 状态码及详情
      properties:
        status_code:
          type: integer
          description: |-
            状态码及其分别含义如下：

            `0`: 请求成功

            `1002`: 触发限流，请稍后再试

            `1004`: 账号鉴权失败，请检查 API-Key 是否填写正确

            `1008`: 账号余额不足

            `1026`: 输入包含敏感内容

            `2013`: 传入参数异常，请检查入参是否按要求填写

            `2049`: 无效的api key

            更多内容可查看 [错误码查询列表](/api-reference/errorcode) 了解详情
        status_msg:
          type: string
          description: 具体错误详情
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: |-
        `HTTP: Bearer Auth`
         - Security Scheme Type: http
         - HTTP Authorization Scheme: Bearer API_key，用于验证账户信息，可在 [账户管理>接口密钥](https://platform.minimaxi.com/user-center/basic-information/interface-key) 中查看。

````