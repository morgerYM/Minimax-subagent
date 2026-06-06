use std::env;
use std::io::Read;
use std::process::exit;

use minimax_api::utils;
use minimax_api::ws_client::WsTtsClient;
use minimax_api::MiniMaxClient;
use minimax_api::types::*;

const DEFAULT_SYSTEM_PROMPT: &str = "\
你是一个编程助手。遵循 Karpathy Guidelines：\
1) 先思考再编码，不确定时主动询问；\
2) 简洁优先，只写最小必要代码；\
3) 精准修改，不乱动无关代码；\
4) 目标驱动，定义可验证的成功标准。";

fn collect_args(args: &[String], start: usize) -> String {
    if start >= args.len() {
        return String::new();
    }
    args[start..].join(" ")
}

/// Parse `--file <path>` and `--output-directory <dir>` from `args[start..]`.
/// Returns `(file, output_directory, next_index)`. These flags must appear
/// before any positional arguments (they are consumed in order from the start).
fn parse_output_flags(
    args: &[String],
    start: usize,
) -> (Option<String>, Option<String>, usize) {
    let mut file: Option<String> = None;
    let mut dir: Option<String> = None;
    let mut i = start;
    while i < args.len() {
        let a = &args[i];
        if a == "--file" && i + 1 < args.len() {
            file = Some(args[i + 1].clone());
            i += 2;
        } else if a == "--output-directory" && i + 1 < args.len() {
            dir = Some(args[i + 1].clone());
            i += 2;
        } else {
            break;
        }
    }
    (file, dir, i)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // 支持 -h/--help
    if args.len() >= 2 && (args[1] == "-h" || args[1] == "--help") {
        eprintln!("Usage: ./minimax <command> [args...]");
        eprintln!("Commands:");
        eprintln!("  list_voices [voice_type] - 列出音色（voice_type: all/system/voice_cloning/voice_generation，默认 all）");
        eprintln!("  query_usage              - 查询账户用量");
        eprintln!("  text_to_audio <text> [--voice v] [--speed 0.5-2.0] [--vol 0-10] [--pitch -12~12] [--emotion e]");
        eprintln!("    [--file <path>] [--output-directory <dir>] - 文字转语音（自动播放；指定输出路径时仅保存不播放）");
        eprintln!("  text_to_audio_stream <text> [--voice v] [--emotion e] [--continuous-sound] [--file <path>] [--output-directory <dir>] - 流式 TTS（WebSocket，单次最大 10000 字符；continuous_sound 仅 speech-2.8-hd/turbo）");
        eprintln!("  generate_audio_async <text> [--voice v] [--emotion e] [--text-file-id id] [--file <path>] [--output-directory <dir>] - 异步 TTS（≤5万字符；--file/--output-directory 在 query_audio_task 时生效）");
        eprintln!("  query_audio_task <task_id> [--file <path>] [--output-directory <dir>] - 查询异步 TTS 任务并下载播放（指定输出路径时仅保存）");
        eprintln!("  web_search <query>       - 网络搜索");
        eprintln!("  understand_image <prompt> <image_path> - 图片理解");
        eprintln!("  chat [--model m] [--system s] [--max-tokens n] [--temperature t] [--messages json] <message> - 文本对话");
        eprintln!("  generate_image [--aspect-ratio r] [--n n] [--style-type s] [--seed s] ... [--file <path>] [--output-directory <dir>] <prompt> - 图像生成（自动打开；指定输出路径时仅保存）");
        eprintln!("  generate_video [--wait] [--model m] [--resolution r] [--duration d] ... [--file <path>] [--output-directory <dir>] <prompt> - 视频生成");
        eprintln!("  query_video <task_id> [--file <path>] [--output-directory <dir>] - 查询视频任务状态（指定输出路径时下载保存）");
        eprintln!("  generate_music <prompt> <lyrics> [--file <path>] [--output-directory <dir>] - 音乐生成（自动播放；is_instrumental=true 时 lyrics 传空串）");
        eprintln!("  generate_music_cover <audio_url> [--prompt p] [--lyrics l] [--file <path>] [--output-directory <dir>] - 翻唱（自动播放；内部自动预处理音频）");
        eprintln!("  generate_lyrics <style>  - 歌词生成");
        eprintln!("  voice_clone <voice_id> <audio_file> [text] [--file <path>] [--output-directory <dir>] - 音色克隆（自动上传参考音频）");
        eprintln!("  voice_design <prompt> <preview_text> [voice_id] [--file <path>] [--output-directory <dir>] - 音色设计 ⚠️ 需要较大账户余额（API error 1008）");
        eprintln!("  delete_voice <voice_type> <voice_id> - 删除音色");
        eprintln!("  list_files <purpose>     - 列出平台文件");
        eprintln!("  retrieve_file <file_id>  - 查看文件详情");
        eprintln!("  delete_file <file_id> <purpose> - 删除文件");
        eprintln!("  generate_video_agent <template_id> [--text-inputs v1,v2] [--media-inputs u1,u2] [--file <path>] [--output-directory <dir>] - 模板视频");
        eprintln!("  query_video_agent <task_id> [--file <path>] [--output-directory <dir>] - 查询模板视频状态（指定输出路径时下载保存）");
        exit(0);
    }

    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin minimax -- <command> [args...]");
        eprintln!("Commands:");
        eprintln!("  list_voices               - 列出所有音色");
        eprintln!("  query_usage              - 查询账户用量");
        eprintln!("  text_to_audio <text>     - 文字转语音");
        eprintln!("  text_to_audio_stream <text> - 流式 TTS");
        eprintln!("  generate_audio_async <text> - 异步 TTS");
        eprintln!("  query_audio_task <id>    - 查询异步 TTS");
        eprintln!("  web_search <query>       - 网络搜索");
        eprintln!("  understand_image <p> <f> - 图片理解");
        eprintln!("  chat <message>           - 文本对话");
        eprintln!("  generate_image <prompt>   - 图像生成");
        eprintln!("  generate_video <prompt>  - 视频生成");
        eprintln!("  query_video <task_id>    - 查询视频");
        eprintln!("  generate_music <p> <l>   - 音乐生成");
        eprintln!("  generate_music_cover <u> - 翻唱");
        eprintln!("  generate_lyrics <style>  - 歌词生成");
        eprintln!("  voice_clone <id> <f> [t] - 音色克隆");
        eprintln!("  voice_design <p> <t> [id] - 音色设计");
        eprintln!("  delete_voice <type> <id> - 删除音色");
        eprintln!("  list_files <purpose>     - 列出文件");
        eprintln!("  retrieve_file <id>       - 文件详情");
        eprintln!("  delete_file <id> <purp>  - 删除文件");
        eprintln!("  generate_video_agent <id> - 模板视频");
        eprintln!("  query_video_agent <id>   - 查询模板视频");
        exit(1);
    }

    let client = match MiniMaxClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    match args[1].as_str() {
        "list_voices" => {
            match client.list_voices(None).await {
                Ok(resp) => {
                    println!("=== 系统音色 ===");
                    for v in &resp.system_voice {
                        println!("  {} — {}", v.voice_id, v.voice_name);
                    }
                    println!("\n=== 克隆音色 ===");
                    for v in &resp.voice_cloning {
                        println!("  {} — {}", v.voice_id, v.voice_name);
                    }
                    println!("\n共 {} 个系统音色 + {} 个克隆音色",
                        resp.system_voice.len(), resp.voice_cloning.len());
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "query_usage" => {
            match client.get_token_plan_remains().await {
                Ok(resp) => {
                    let mut lines = Vec::new();
                    let mut keys: Vec<&String> = resp.extra.keys().collect();
                    keys.sort();
                    for key in keys {
                        if let Some(val) = resp.extra.get(key) {
                            lines.push(format!("{}: {}", key, val));
                        }
                    }
                    if lines.is_empty() {
                        println!("status: {}", resp.base_resp.status_msg);
                    } else {
                        for line in lines {
                            println!("{}", line);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "text_to_audio" => {
            // 简单参数解析：text --voice xxx --speed 1.5 --vol 1.0 --pitch 0 --emotion happy
            // 默认音色: female-yujie
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            let mut voice_id = "female-yujie".to_string();
            let mut speed: Option<f64> = None;
            let mut vol: Option<f64> = None;
            let mut pitch: Option<i32> = None;
            let mut emotion: Option<String> = None;
            let mut text: String = String::new();
            let mut found_text = false;

            let mut i = next_i;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--voice" && i + 1 < args.len() {
                    voice_id = args[i + 1].clone();
                    i += 2;
                } else if arg == "--speed" && i + 1 < args.len() {
                    speed = args[i + 1].parse().ok();
                    i += 2;
                } else if arg == "--vol" && i + 1 < args.len() {
                    vol = args[i + 1].parse().ok();
                    i += 2;
                } else if arg == "--pitch" && i + 1 < args.len() {
                    pitch = args[i + 1].parse().ok();
                    i += 2;
                } else if arg == "--emotion" && i + 1 < args.len() {
                    emotion = Some(args[i + 1].clone());
                    i += 2;
                } else if arg.starts_with("--") {
                    eprintln!("Unknown option: {}", arg);
                    exit(1);
                } else {
                    // 第一个非选项参数是文本，之后的非选项参数也合并到文本
                    if !found_text {
                        text = arg.clone();
                        found_text = true;
                    } else {
                        text.push(' ');
                        text.push_str(arg);
                    }
                    i += 1;
                }
            }

            if text.is_empty() {
                eprintln!("Usage: text_to_audio <text> [--voice voice_id] [--speed 0.5-2.0] [--vol 0-10] [--pitch -12~12] [--emotion happy|sad|angry|calm|fluent|whisper|...] [--file <path>] [--output-directory <dir>]");
                exit(1);
            }

            let req = T2ARequest {
                model: "speech-2.8-hd".to_string(),
                text: text.clone(),
                stream: Some(false),
                stream_options: None,
                voice_setting: VoiceSetting {
                    voice_id,
                    speed,
                    vol,
                    pitch,
                    emotion,
                    text_normalization: None,
                    latex_read: None,
                    english_normalization: None,
                },
                audio_setting: AudioSetting {
                    sample_rate: 32000,
                    bitrate: 128000,
                    format: "mp3".to_string(),
                    channel: 1,
                    force_cbr: None,
                },
                pronunciation_dict: None,
                timbre_weights: None,
                language_boost: Some("auto".to_string()),
                voice_modify: None,
                subtitle_enable: None,
                subtitle_type: None,
                output_format: None,
                aigc_watermark: None,
            };
            match client.text_to_audio(&req).await {
                Ok(resp) => {
                    if let Some(data) = &resp.data {
                        if let Some(audio) = &data.audio {
                            let custom = out_file.is_some() || out_dir.is_some();
                            if custom {
                                match utils::decode_hex_audio(audio) {
                                    Ok(bytes) => {
                                        match utils::write_output_file(
                                            out_file.as_deref(),
                                            out_dir.as_deref(),
                                            "text_to_audio",
                                            &text,
                                            "mp3",
                                            &bytes,
                                        )
                                        .await
                                        {
                                            Ok(Some(p)) => {
                                                println!("音频已保存: {}", p.display())
                                            }
                                            Ok(None) => println!("音频生成成功"),
                                            Err(e) => println!("保存失败: {}", e),
                                        }
                                    }
                                    Err(e) => println!("hex 解码失败: {}", e),
                                }
                            } else {
                                match utils::save_and_play_audio(audio, "text_to_audio") {
                                    Ok(path) => println!("音频已保存并播放: {}", path.display()),
                                    Err(e) => println!("音频生成成功，保存/播放失败: {}", e),
                                }
                            }
                        } else {
                            println!("音频生成成功，但无数据");
                        }
                    } else {
                        println!("音频生成成功");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "web_search" => {
            let query = collect_args(&args, 2);
            if query.is_empty() {
                eprintln!("Usage: web_search <query>");
                exit(1);
            }
            let req = SearchRequest { q: query.clone() };
            match client.search(&req).await {
                Ok(resp) => {
                    println!("搜索结果 (共 {} 条):\n", resp.organic.len());
                    for (i, result) in resp.organic.iter().enumerate() {
                        println!("{}. {}", i + 1, result.title);
                        println!("   URL: {}", result.link);
                        println!("   {}\n", result.snippet);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "understand_image" => {
            if args.len() < 4 {
                eprintln!("Usage: understand_image <prompt> <image_path>");
                exit(1);
            }
            let prompt = &args[2];
            let image_path = &args[3];

            let expanded = if image_path.starts_with('~') {
                let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                image_path.replacen('~', &home, 1)
            } else {
                image_path.to_string()
            };

            let data_url = minimax_api::utils::process_image_url(&expanded).await;

            let req = VlmRequest {
                prompt: prompt.clone(),
                image_url: data_url,
            };
            match client.vlm(&req).await {
                Ok(resp) => {
                    if let Some(content) = resp.content {
                        println!("{}", content);
                    } else {
                        println!("未返回内容");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "chat" => {
            let mut model = "MiniMax-M3".to_string();
            let mut system: Option<String> = None;
            let mut max_tokens = 2048;
            let mut temperature = 0.7;
            let mut messages_json: Option<String> = None;
            let mut text = String::new();

            let mut i = 2;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--model" && i + 1 < args.len() {
                    model = args[i + 1].clone();
                    i += 2;
                } else if arg == "--system" && i + 1 < args.len() {
                    system = Some(args[i + 1].clone());
                    i += 2;
                } else if arg == "--max-tokens" && i + 1 < args.len() {
                    max_tokens = args[i + 1].parse().unwrap_or(2048);
                    i += 2;
                } else if arg == "--temperature" && i + 1 < args.len() {
                    temperature = args[i + 1].parse().unwrap_or(0.7);
                    i += 2;
                } else if arg == "--messages" && i + 1 < args.len() {
                    messages_json = Some(args[i + 1].clone());
                    i += 2;
                } else if arg.starts_with("--") {
                    eprintln!("Unknown option: {}", arg);
                    exit(1);
                } else {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(arg);
                    i += 1;
                }
            }

            let messages = if let Some(json) = &messages_json {
                if !text.is_empty() {
                    eprintln!("Error: --messages 和位置文本互斥，只能使用一种");
                    exit(1);
                }
                serde_json::from_str::<Vec<ChatMessage>>(json).unwrap_or_else(|e| {
                    eprintln!("Error: --messages JSON 解析失败: {}", e);
                    exit(1);
                })
            } else {
                if text.is_empty() {
                    eprintln!("Usage: chat [--model m] [--system s] [--max-tokens n] [--temperature t] [--messages json] <message>");
                    exit(1);
                }
                vec![ChatMessage {
                    role: "user".to_string(),
                    content: text,
                }]
            };

            let system_prompt = system.unwrap_or_else(|| DEFAULT_SYSTEM_PROMPT.to_string());

            let req = ChatRequest {
                model,
                messages,
                system: Some(system_prompt),
                max_tokens: Some(max_tokens),
                temperature: Some(temperature),
                top_p: None,
                stream: false,
            };
            match client.chat(&req).await {
                Ok(resp) => {
                    let result: Vec<String> = resp.content
                        .iter()
                        .filter(|b| b.block_type == "text")
                        .filter_map(|b| b.text.as_deref())
                        .map(String::from)
                        .collect();
                    let text = result.join("\n");
                    if text.is_empty() {
                        println!("聊天完成，但无文本输出。");
                    } else {
                        println!("{}", text);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "generate_image" => {
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            let mut model = "image-01".to_string();
            let mut aspect_ratio: Option<String> = None;
            let mut n: Option<i32> = Some(1);
            let mut style_type: Option<String> = None;
            let mut style_weight: Option<f64> = None;
            let mut width: Option<i32> = None;
            let mut height: Option<i32> = None;
            let mut seed: Option<i64> = None;
            let mut prompt_optimizer = Some(true);
            let mut response_format: Option<String> = None;
            let mut prompt = String::new();

            let mut i = next_i;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--model" && i + 1 < args.len() {
                    model = args[i + 1].clone();
                    i += 2;
                } else if arg == "--aspect-ratio" && i + 1 < args.len() {
                    aspect_ratio = Some(args[i + 1].clone());
                    i += 2;
                } else if arg == "--n" && i + 1 < args.len() {
                    n = args[i + 1].parse().ok();
                    i += 2;
                } else if arg == "--style-type" && i + 1 < args.len() {
                    style_type = Some(args[i + 1].clone());
                    i += 2;
                } else if arg == "--style-weight" && i + 1 < args.len() {
                    style_weight = args[i + 1].parse().ok();
                    i += 2;
                } else if arg == "--width" && i + 1 < args.len() {
                    width = args[i + 1].parse().ok();
                    i += 2;
                } else if arg == "--height" && i + 1 < args.len() {
                    height = args[i + 1].parse().ok();
                    i += 2;
                } else if arg == "--seed" && i + 1 < args.len() {
                    seed = args[i + 1].parse().ok();
                    i += 2;
                } else if arg == "--no-optimizer" {
                    prompt_optimizer = Some(false);
                    i += 1;
                } else if arg == "--response-format" && i + 1 < args.len() {
                    response_format = Some(args[i + 1].clone());
                    i += 2;
                } else if arg.starts_with("--") {
                    eprintln!("Unknown option: {}", arg);
                    exit(1);
                } else {
                    if !prompt.is_empty() { prompt.push(' '); }
                    prompt.push_str(arg);
                    i += 1;
                }
            }

            if prompt.is_empty() {
                eprintln!("Usage: generate_image [--model m] [--aspect-ratio 1:1|16:9|...] [--n 1-9] [--style-type cartoon|...] [--style-weight 0-1] [--width w] [--height h] [--seed s] [--no-optimizer] [--response-format url|base64] [--file <path>] [--output-directory <dir>] <prompt>");
                exit(1);
            }

            let style = style_type.map(|st| ImageStyle {
                style_type: st,
                style_weight,
            });

            let req = ImageGenerationRequest {
                model,
                prompt: prompt.clone(),
                aspect_ratio,
                n,
                prompt_optimizer,
                width,
                height,
                response_format,
                seed,
                aigc_watermark: None,
                subject_reference: None,
                style,
            };
            match client.generate_image(&req).await {
                Ok(resp) => {
                    if let Some(data) = &resp.data {
                        let custom = out_file.is_some() || out_dir.is_some();
                        println!("图像生成成功 (共 {} 张):", data.image_urls.len());
                        let total = data.image_urls.len();
                        for (idx, url) in data.image_urls.iter().enumerate() {
                            println!("  {}", url);
                            if custom {
                                // Mirror MCP image handler: per-index stem when n>1.
                                let per_file = if let Some(f) = out_file.as_deref() {
                                    let p = std::path::Path::new(f);
                                    let stem = p
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("image");
                                    let ext = p
                                        .extension()
                                        .and_then(|e| e.to_str())
                                        .unwrap_or("jpg");
                                    if total > 1 {
                                        format!("{stem}_{idx}.{ext}")
                                    } else {
                                        format!("{stem}.{ext}")
                                    }
                                } else {
                                    String::new()
                                };
                                match utils::resolve_output_file(
                                    if per_file.is_empty() { None } else { Some(per_file.as_str()) },
                                    out_dir.as_deref(),
                                    "image",
                                    &format!("{}_{}", idx, prompt),
                                    "jpg",
                                ) {
                                    Ok(Some(path)) => {
                                        match client.download_to_path(url, &path).await {
                                            Ok(()) => println!("  [已保存: {}]", path.display()),
                                            Err(e) => println!("  [下载失败: {}]", e),
                                        }
                                    }
                                    Ok(None) => {
                                        // Should not happen since custom==true
                                    }
                                    Err(e) => println!("  [路径解析失败: {}]", e),
                                }
                            } else if let Ok(path) = utils::download_and_open_image(url, "generate_image").await {
                                println!("  [已打开: {}]", path.display());
                            }
                        }
                    } else {
                        println!("图像生成成功，但无数据");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "generate_video" => {
            let mut model = "MiniMax-Hailuo-2.3".to_string();
            let mut wait = false;
            let mut resolution: Option<String> = None;
            let mut duration: Option<i32> = None;
            let mut first_frame_image: Option<String> = None;
            let mut last_frame_image: Option<String> = None;
            let mut subject_ref_json: Option<String> = None;
            let mut prompt = String::new();

            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            let mut i = next_i;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--model" && i + 1 < args.len() {
                    model = args[i + 1].clone();
                    i += 2;
                } else if arg == "--wait" {
                    wait = true;
                    i += 1;
                } else if arg == "--resolution" && i + 1 < args.len() {
                    resolution = Some(args[i + 1].clone());
                    i += 2;
                } else if arg == "--duration" && i + 1 < args.len() {
                    duration = args[i + 1].parse().ok();
                    i += 2;
                } else if arg == "--first-frame" && i + 1 < args.len() {
                    first_frame_image = Some(args[i + 1].clone());
                    i += 2;
                } else if arg == "--last-frame" && i + 1 < args.len() {
                    last_frame_image = Some(args[i + 1].clone());
                    i += 2;
                } else if arg == "--subject-reference" && i + 1 < args.len() {
                    subject_ref_json = Some(args[i + 1].clone());
                    i += 2;
                } else if arg.starts_with("--") {
                    eprintln!("Unknown option: {}", arg);
                    exit(1);
                } else {
                    if !prompt.is_empty() { prompt.push(' '); }
                    prompt.push_str(arg);
                    i += 1;
                }
            }

            if prompt.is_empty() {
                eprintln!("Usage: generate_video [--model m] [--wait] [--resolution 768P|1080P] [--duration 6|10] [--first-frame url] [--last-frame url] [--subject-reference 'json'] [--file <path>] [--output-directory <dir>] <prompt>");
                exit(1);
            }

            // 解析 --subject-reference JSON
            let subject_reference = subject_ref_json.and_then(|s| {
                let v: serde_json::Value = serde_json::from_str(&s).ok()?;
                let arr = v.as_array()?;
                Some(
                    arr.iter()
                        .filter_map(|item| {
                            let reference_type = item.get("type")?.as_str()?.to_string();
                            let image = item.get("image")?
                                .as_array()?
                                .iter()
                                .filter_map(|s| s.as_str().map(String::from))
                                .collect();
                            Some(SubjectReference { reference_type, image })
                        })
                        .collect::<Vec<_>>()
                )
            });

            let req = VideoGenerationRequest {
                model,
                prompt: prompt.clone(),
                first_frame_image,
                last_frame_image,
                subject_reference,
                duration,
                resolution,
                prompt_optimizer: None,
                fast_pretreatment: None,
                callback_url: None,
                aigc_watermark: None,
            };

            if wait {
                println!("提交视频任务并等待完成...");
                match client.generate_video_and_download(&req).await {
                    Ok(bytes) => {
                        let custom = out_file.is_some() || out_dir.is_some();
                        if custom {
                            // Use mp4 ext (avoids the legacy .mp3 bug for video bytes)
                            match utils::write_output_file(
                                out_file.as_deref(),
                                out_dir.as_deref(),
                                "video",
                                &prompt,
                                "mp4",
                                &bytes,
                            )
                            .await
                            {
                                Ok(Some(p)) => println!("视频已保存: {}", p.display()),
                                Ok(None) => println!("视频生成成功"),
                                Err(e) => eprintln!("视频下载成功但保存失败: {}", e),
                            }
                        } else {
                            match utils::save_and_play_audio_bytes(&bytes, "video") {
                                Ok(path) => println!("视频已保存: {}", path.display()),
                                Err(e) => eprintln!("视频下载成功但保存失败: {}", e),
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        exit(1);
                    }
                }
            } else {
                match client.create_video(&req).await {
                    Ok(resp) => {
                        println!("视频任务已提交!");
                        println!("task_id: {}", resp.task_id);
                        println!("使用 query_video {} 查询进度", resp.task_id);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        exit(1);
                    }
                }
            }
        }

        "query_video" => {
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            if next_i >= args.len() {
                eprintln!("Usage: query_video <task_id> [--file <path>] [--output-directory <dir>]");
                exit(1);
            }
            let task_id = &args[next_i];
            let custom = out_file.is_some() || out_dir.is_some();
            match client.query_video(task_id).await {
                Ok(resp) => {
                    println!("任务状态: {}", resp.status);
                    if let Some(file_id) = &resp.file_id {
                        println!("file_id: {}", file_id);
                    }
                    if resp.status == "Success" {
                        if let Some(file_id) = &resp.file_id {
                            match client.get_file_download_url(file_id).await {
                                Ok(download_url) => {
                                    if custom {
                                        match utils::resolve_output_file(
                                            out_file.as_deref(),
                                            out_dir.as_deref(),
                                            "video",
                                            task_id,
                                            "mp4",
                                        ) {
                                            Ok(Some(path)) => {
                                                match client.download_to_path(&download_url, &path).await {
                                                    Ok(()) => println!("视频已保存: {}", path.display()),
                                                    Err(e) => eprintln!("下载失败: {}", e),
                                                }
                                            }
                                            Ok(None) => println!("视频可下载: {}", download_url),
                                            Err(e) => eprintln!("路径解析失败: {}", e),
                                        }
                                    } else {
                                        println!("download_url: {}", download_url);
                                    }
                                }
                                Err(e) => eprintln!("获取下载链接失败: {}", e),
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "generate_music" => {
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            if next_i + 1 >= args.len() {
                eprintln!("Usage: generate_music [--file <path>] [--output-directory <dir>] <prompt> <lyrics>");
                exit(1);
            }
            let prompt = &args[next_i];
            let lyrics = &args[next_i + 1];
            let req = MusicGenerationRequest {
                model: "music-2.6".to_string(),
                prompt: prompt.clone(),
                lyrics: lyrics.clone(),
                audio_setting: MusicAudioSetting {
                    sample_rate: 32000,
                    bitrate: 128000,
                    format: "mp3".to_string(),
                },
                output_format: None,
                audio_url: None,
                audio_base64: None,
                cover_feature_id: None,
                timbre: None,
                stream: None,
                aigc_watermark: None,
                lyrics_optimizer: None,
                is_instrumental: None,
            };
            let custom = out_file.is_some() || out_dir.is_some();
            match client.generate_music(&req).await {
                Ok(resp) => {
                    if let Some(data) = &resp.data {
                        if let Some(audio) = &data.audio {
                            if custom {
                                match utils::decode_hex_audio(audio) {
                                    Ok(bytes) => {
                                        match utils::write_output_file(
                                            out_file.as_deref(),
                                            out_dir.as_deref(),
                                            "generate_music",
                                            prompt,
                                            "mp3",
                                            &bytes,
                                        )
                                        .await
                                        {
                                            Ok(Some(p)) => println!("音乐已保存: {}", p.display()),
                                            Ok(None) => println!("音乐生成成功"),
                                            Err(e) => println!("保存失败: {}", e),
                                        }
                                    }
                                    Err(e) => println!("hex 解码失败: {}", e),
                                }
                            } else {
                                match utils::save_and_play_audio(audio, "generate_music") {
                                    Ok(path) => println!("音乐已保存并播放: {}", path.display()),
                                    Err(e) => println!("音乐生成成功，保存/播放失败: {}", e),
                                }
                            }
                        } else {
                            println!("音乐生成成功，但无音频数据");
                        }
                    } else {
                        println!("音乐生成成功");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "generate_lyrics" => {
            if args.len() < 3 {
                eprintln!("Usage: generate_lyrics <style>");
                exit(1);
            }
            let prompt = &args[2];
            let req = LyricsGenerationRequest {
                mode: "write_full_song".to_string(),
                prompt: prompt.clone(),
                lyrics: None,
                title: None,
            };
            match client.generate_lyrics(&req).await {
                Ok(resp) => {
                    if let Some(title) = &resp.song_title {
                        println!("歌名: {}", title);
                    }
                    if let Some(tags) = &resp.style_tags {
                        println!("风格标签: {}", tags);
                    }
                    if let Some(lyrics) = &resp.lyrics {
                        println!("\n歌词:\n{}", lyrics);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "voice_clone" => {
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            if next_i + 1 >= args.len() {
                eprintln!("Usage: voice_clone [--file <path>] [--output-directory <dir>] <voice_id> <audio_file> [text]");
                exit(1);
            }
            let voice_id = &args[next_i];
            let audio_file = &args[next_i + 1];
            let text = args.get(next_i + 2).map(|s| s.as_str());

            // Upload the audio file first
            let upload_resp = match client.upload_file(std::path::Path::new(audio_file), "voice_clone").await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error uploading file: {}", e);
                    exit(1);
                }
            };

            let file_id = match upload_resp.file {
                Some(f) => f.file_id,
                None => {
                    eprintln!("Error: upload failed, no file_id returned");
                    exit(1);
                }
            };

            let req = VoiceCloneRequest {
                file_id,
                voice_id: voice_id.clone(),
                clone_prompt: None,
                text: text.map(String::from),
                // 当传 text 时必须传 model (官方要求)
                model: if text.is_some() {
                    Some("speech-2.8-hd".to_string())
                } else {
                    None
                },
                language_boost: None,
                need_noise_reduction: None,
                need_volume_normalization: None,
                aigc_watermark: None,
            };

            let custom = out_file.is_some() || out_dir.is_some();
            match client.voice_clone(&req).await {
                Ok(resp) => {
                    println!("音色克隆成功!");
                    println!("voice_id: {}", req.voice_id);
                    if let Some(demo) = &resp.demo_audio {
                        if custom {
                            match utils::resolve_output_file(
                                out_file.as_deref(),
                                out_dir.as_deref(),
                                "voice_clone",
                                text.unwrap_or("voice"),
                                "wav",
                            ) {
                                Ok(Some(path)) => {
                                    match client.download_to_path(demo, &path).await {
                                        Ok(()) => println!("试听音频已保存: {}", path.display()),
                                        Err(e) => eprintln!("下载失败: {}", e),
                                    }
                                }
                                Ok(None) => println!("demo_audio: {}", demo),
                                Err(e) => eprintln!("路径解析失败: {}", e),
                            }
                        } else {
                            println!("demo_audio: {}", demo);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "voice_design" => {
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            if next_i + 1 >= args.len() {
                eprintln!("Usage: voice_design [--file <path>] [--output-directory <dir>] <prompt> <preview_text> [voice_id]");
                exit(1);
            }
            let prompt = &args[next_i];
            let preview_text = &args[next_i + 1];
            let voice_id = args.get(next_i + 2).map(|s| s.as_str());

            let req = VoiceDesignRequest {
                prompt: prompt.clone(),
                preview_text: preview_text.clone(),
                voice_id: voice_id.map(String::from),
            };
            let custom = out_file.is_some() || out_dir.is_some();
            match client.voice_design(&req).await {
                Ok(resp) => {
                    println!("音色设计成功!");
                    if let Some(id) = resp.voice_id.clone() {
                        println!("voice_id: {}", id);
                    }
                    if let Some(audio_hex) = &resp.trial_audio {
                        if custom {
                            match utils::decode_hex_audio(audio_hex) {
                                Ok(bytes) => {
                                    match utils::write_output_file(
                                        out_file.as_deref(),
                                        out_dir.as_deref(),
                                        "voice_design",
                                        preview_text,
                                        "mp3",
                                        &bytes,
                                    )
                                    .await
                                    {
                                        Ok(Some(p)) => println!("试听音频已保存: {}", p.display()),
                                        Ok(None) => println!("试听音频长度: {} 字符", audio_hex.len()),
                                        Err(e) => eprintln!("保存失败: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("hex 解码失败: {}", e),
                            }
                        } else {
                            println!("trial_audio 长度: {} 字符", audio_hex.len());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "text_to_audio_stream" => {
            let mut voice_id = "female-yujie".to_string();
            let mut model = "speech-2.8-hd".to_string();
            let mut speed: Option<f64> = None;
            let mut vol: Option<f64> = None;
            let mut pitch: Option<i32> = None;
            let mut emotion: Option<String> = None;
            let mut sample_rate = 32000;
            let mut bitrate = 128000;
            let mut format = "mp3".to_string();
            let mut channel = 1;
            let mut language_boost = Some("auto".to_string());
            let mut continuous_sound = false;
            let mut text = String::new();

            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            let mut i = next_i;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--voice" && i + 1 < args.len() { voice_id = args[i+1].clone(); i += 2; }
                else if arg == "--model" && i + 1 < args.len() { model = args[i+1].clone(); i += 2; }
                else if arg == "--speed" && i + 1 < args.len() { speed = args[i+1].parse().ok(); i += 2; }
                else if arg == "--vol" && i + 1 < args.len() { vol = args[i+1].parse().ok(); i += 2; }
                else if arg == "--pitch" && i + 1 < args.len() { pitch = args[i+1].parse().ok(); i += 2; }
                else if arg == "--emotion" && i + 1 < args.len() { emotion = Some(args[i+1].clone()); i += 2; }
                else if arg == "--sample-rate" && i + 1 < args.len() { sample_rate = args[i+1].parse().unwrap_or(32000); i += 2; }
                else if arg == "--bitrate" && i + 1 < args.len() { bitrate = args[i+1].parse().unwrap_or(128000); i += 2; }
                else if arg == "--format" && i + 1 < args.len() { format = args[i+1].clone(); i += 2; }
                else if arg == "--channel" && i + 1 < args.len() { channel = args[i+1].parse().unwrap_or(1); i += 2; }
                else if arg == "--language-boost" && i + 1 < args.len() { language_boost = Some(args[i+1].clone()); i += 2; }
                else if arg == "--continuous-sound" { continuous_sound = true; i += 1; }
                else if arg.starts_with("--") { eprintln!("Unknown option: {}", arg); exit(1); }
                else { if !text.is_empty() { text.push(' '); } text.push_str(arg); i += 1; }
            }

            if text.is_empty() {
                eprintln!("Usage: text_to_audio_stream [--voice v] [--speed 0.5-2.0] [--vol 0-10] [--pitch -12~12] [--emotion e] [--model m] [--format f] [--sample-rate r] [--bitrate b] [--channel c] [--language-boost l] [--continuous-sound] [--file <path>] [--output-directory <dir>] <text>");
                exit(1);
            }

            let ws_req = WsTaskStart {
                event: "task_start".to_string(),
                model,
                voice_setting: WsVoiceSetting {
                    voice_id,
                    speed,
                    vol,
                    pitch,
                    emotion,
                    english_normalization: None,
                    latex_read: None,
                },
                audio_setting: Some(WsAudioSetting { sample_rate, bitrate, format: format.clone(), channel }),
                language_boost,
                pronunciation_dict: None,
                timbre_weights: None,
                voice_modify: None,
                subtitle_enable: None,
                subtitle_type: None,
                continuous_sound: Some(continuous_sound),
            };

            let mut ws = match WsTtsClient::connect(&client.base_url, &client.api_key).await {
                Ok(ws) => ws,
                Err(e) => { eprintln!("WebSocket 连接失败: {}", e); exit(1); }
            };
            if let Err(e) = ws.task_start(&ws_req).await {
                eprintln!("task_start 失败: {}", e);
                exit(1);
            }
            match ws.task_continue(&text).await {
                Ok((audio_bytes, _finished)) => {
                    ws.task_finish().await.ok();
                    if audio_bytes.is_empty() {
                        println!("流式 TTS 完成，但无音频数据");
                    } else {
                        let custom = out_file.is_some() || out_dir.is_some();
                        if custom {
                            match utils::write_output_file(
                                out_file.as_deref(),
                                out_dir.as_deref(),
                                "stream_tts",
                                &text,
                                &format,
                                &audio_bytes,
                            )
                            .await
                            {
                                Ok(Some(p)) => println!("音频已保存: {}", p.display()),
                                Ok(None) => println!("流式 TTS 完成"),
                                Err(e) => eprintln!("保存失败: {}", e),
                            }
                        } else {
                            match utils::save_and_play_audio_bytes(&audio_bytes, "stream_tts") {
                                Ok(path) => println!("音频已保存并播放: {}", path.display()),
                                Err(e) => eprintln!("保存播放失败: {}", e),
                            }
                        }
                    }
                }
                Err(e) => {
                    ws.task_finish().await.ok();
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "generate_audio_async" => {
            let mut voice_id = "female-yujie".to_string();
            let mut model = "speech-2.8-hd".to_string();
            let mut speed: Option<f64> = None;
            let mut vol: Option<f64> = None;
            let mut pitch: Option<i32> = None;
            let mut emotion: Option<String> = None;
            let mut sample_rate = 32000;
            let mut bitrate = 128000;
            let mut format = "mp3".to_string();
            let mut channel = 2;
            let mut language_boost = Some("auto".to_string());
            let mut text_file_id: Option<i64> = None;
            let mut text = String::new();

            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            let mut i = next_i;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--voice" && i + 1 < args.len() { voice_id = args[i+1].clone(); i += 2; }
                else if arg == "--model" && i + 1 < args.len() { model = args[i+1].clone(); i += 2; }
                else if arg == "--speed" && i + 1 < args.len() { speed = args[i+1].parse().ok(); i += 2; }
                else if arg == "--vol" && i + 1 < args.len() { vol = args[i+1].parse().ok(); i += 2; }
                else if arg == "--pitch" && i + 1 < args.len() { pitch = args[i+1].parse().ok(); i += 2; }
                else if arg == "--emotion" && i + 1 < args.len() { emotion = Some(args[i+1].clone()); i += 2; }
                else if arg == "--sample-rate" && i + 1 < args.len() { sample_rate = args[i+1].parse().unwrap_or(32000); i += 2; }
                else if arg == "--bitrate" && i + 1 < args.len() { bitrate = args[i+1].parse().unwrap_or(128000); i += 2; }
                else if arg == "--format" && i + 1 < args.len() { format = args[i+1].clone(); i += 2; }
                else if arg == "--channel" && i + 1 < args.len() { channel = args[i+1].parse().unwrap_or(2); i += 2; }
                else if arg == "--language-boost" && i + 1 < args.len() { language_boost = Some(args[i+1].clone()); i += 2; }
                else if arg == "--text-file-id" && i + 1 < args.len() { text_file_id = args[i+1].parse().ok(); i += 2; }
                else if arg.starts_with("--") { eprintln!("Unknown option: {}", arg); exit(1); }
                else { if !text.is_empty() { text.push(' '); } text.push_str(arg); i += 1; }
            }

            if text.is_empty() && text_file_id.is_none() {
                eprintln!("Usage: generate_audio_async [--voice v] [--speed 0.5-2.0] [--vol 0-10] [--pitch -12~12] [--emotion e] [--model m] [--format f] [--sample-rate r] [--bitrate b] [--channel c] [--language-boost l] [--text-file-id id] [--file <path>] [--output-directory <dir>] <text>");
                exit(1);
            }

            let req = T2AAsyncRequest {
                model,
                text,
                text_file_id,
                voice_setting: VoiceSetting {
                    voice_id,
                    speed,
                    vol,
                    pitch,
                    emotion,
                    text_normalization: None,
                    latex_read: None,
                    english_normalization: None,
                },
                audio_setting: Some(AsyncAudioSetting { audio_sample_rate: sample_rate, bitrate, format, channel }),
                pronunciation_dict: None,
                language_boost,
                voice_modify: None,
                aigc_watermark: None,
            };
            match client.create_async_tts(&req).await {
                Ok(resp) => {
                    println!("异步 TTS 任务已创建!");
                    println!("task_id: {}", resp.task_id);
                    println!("file_id: {}", resp.file_id);
                    println!("usage_characters: {}", resp.usage_characters);
                    println!("使用 query_audio_task {} 查询并下载", resp.task_id);
                    if out_file.is_some() || out_dir.is_some() {
                        println!("提示: --file/--output-directory 在 query_audio_task 步骤生效");
                        if let Some(f) = &out_file { println!("  output_file = {}", f); }
                        if let Some(d) = &out_dir { println!("  output_directory = {}", d); }
                    }
                }
                Err(e) => { eprintln!("Error: {}", e); exit(1); }
            }
        }

        "query_audio_task" => {
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            if next_i >= args.len() {
                eprintln!("Usage: query_audio_task [--file <path>] [--output-directory <dir>] <task_id>");
                exit(1);
            }
            let task_id: i64 = match args[next_i].parse() {
                Ok(id) => id,
                Err(_) => { eprintln!("Invalid task_id: {}", args[next_i]); exit(1); }
            };

            let query = match client.query_async_tts(task_id).await {
                Ok(q) => q,
                Err(e) => { eprintln!("Error: {}", e); exit(1); }
            };

            if query.status == "Failed" || query.status == "Expired" {
                eprintln!("任务失败或已过期: {}", query.status);
                exit(1);
            }

            let file_id = query.file_id.unwrap_or(task_id);

            use std::io::Cursor;
            let download_url = match client.poll_file_download_url(file_id, 30, 5).await {
                Ok(url) => url,
                Err(e) => { eprintln!("轮询下载链接失败: {}", e); exit(1); }
            };
            let tar_bytes = match client.download_bytes(&download_url).await {
                Ok(b) => b,
                Err(e) => { eprintln!("下载失败: {}", e); exit(1); }
            };
            let mut archive = tar::Archive::new(Cursor::new(&tar_bytes));
            let mut mp3_bytes = Vec::new();
            for entry in archive.entries().unwrap_or_else(|e| {
                eprintln!("解压 tar 失败: {}", e);
                exit(1);
            }) {
                let mut entry = entry.unwrap_or_else(|e| {
                    eprintln!("读取 tar entry 失败: {}", e);
                    exit(1);
                });
                let path = entry.path().unwrap_or_else(|e| {
                    eprintln!("读取 entry path 失败: {}", e);
                    exit(1);
                });
                if path.extension().map(|e| e == "mp3").unwrap_or(false) {
                    entry.read_to_end(&mut mp3_bytes).unwrap_or_else(|e| {
                        eprintln!("读取 mp3 失败: {}", e);
                        exit(1);
                    });
                    break;
                }
            }
            if mp3_bytes.is_empty() {
                eprintln!("tar 中未找到 mp3 文件");
                exit(1);
            }
            let custom = out_file.is_some() || out_dir.is_some();
            if custom {
                match utils::write_output_file(
                    out_file.as_deref(),
                    out_dir.as_deref(),
                    "async_tts",
                    &task_id.to_string(),
                    "mp3",
                    &mp3_bytes,
                )
                .await
                {
                    Ok(Some(p)) => println!("音频已保存: {}", p.display()),
                    Ok(None) => println!("音频下载成功"),
                    Err(e) => eprintln!("保存失败: {}", e),
                }
            } else {
                match utils::save_and_play_audio_bytes(&mp3_bytes, "async_tts") {
                    Ok(path) => println!("音频已保存并播放: {}", path.display()),
                    Err(e) => eprintln!("保存播放失败: {}", e),
                }
            }
        }

        "generate_music_cover" => {
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            if next_i >= args.len() {
                eprintln!("Usage: generate_music_cover [--file <path>] [--output-directory <dir>] <audio_url> [--prompt p] [--lyrics l]");
                exit(1);
            }
            let audio_url = args[next_i].clone();
            let mut prompt: Option<String> = None;
            let mut lyrics: Option<String> = None;

            let mut i = next_i + 1;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--prompt" && i + 1 < args.len() { prompt = Some(args[i+1].clone()); i += 2; }
                else if arg == "--lyrics" && i + 1 < args.len() { lyrics = Some(args[i+1].clone()); i += 2; }
                else if arg.starts_with("--") { eprintln!("Unknown option: {}", arg); exit(1); }
                else { i += 1; }
            }

            let cover_feature_id = match client.preprocess_music_cover(&audio_url).await {
                Ok(resp) => resp.cover_feature_id,
                Err(e) => { eprintln!("预处理失败: {}", e); exit(1); }
            };

            let prompt_for_req = prompt.clone();
            let req = MusicGenerationRequest {
                model: "music-cover".to_string(),
                prompt: prompt_for_req.unwrap_or_default(),
                lyrics: lyrics.unwrap_or_default(),
                audio_setting: MusicAudioSetting { sample_rate: 32000, bitrate: 128000, format: "mp3".to_string() },
                output_format: None,
                audio_url: None,
                audio_base64: None,
                cover_feature_id,
                timbre: None,
                stream: None,
                aigc_watermark: None,
                lyrics_optimizer: None,
                is_instrumental: None,
            };

            match client.generate_music(&req).await {
                Ok(resp) => {
                    if let Some(data) = &resp.data {
                        if let Some(audio) = &data.audio {
                            let custom = out_file.is_some() || out_dir.is_some();
                            if custom {
                                match utils::decode_hex_audio(audio) {
                                    Ok(bytes) => {
                                        match utils::write_output_file(
                                            out_file.as_deref(),
                                            out_dir.as_deref(),
                                            "music_cover",
                                            prompt.as_deref().unwrap_or("cover"),
                                            "mp3",
                                            &bytes,
                                        )
                                        .await
                                        {
                                            Ok(Some(p)) => println!("翻唱已保存: {}", p.display()),
                                            Ok(None) => println!("翻唱生成成功"),
                                            Err(e) => println!("保存失败: {}", e),
                                        }
                                    }
                                    Err(e) => println!("hex 解码失败: {}", e),
                                }
                            } else {
                                match utils::save_and_play_audio(audio, "music_cover") {
                                    Ok(path) => println!("翻唱已保存并播放: {}", path.display()),
                                    Err(e) => println!("翻唱生成成功，保存播放失败: {}", e),
                                }
                            }
                        } else { println!("翻唱生成成功，但无音频数据"); }
                    } else { println!("翻唱生成成功"); }
                }
                Err(e) => { eprintln!("Error: {}", e); exit(1); }
            }
        }

        "delete_voice" => {
            if args.len() < 4 {
                eprintln!("Usage: delete_voice <voice_type> <voice_id>");
                eprintln!("  voice_type: system, voice_cloning, voice_generation");
                exit(1);
            }
            let voice_type = &args[2];
            let voice_id = &args[3];

            let req = DeleteVoiceRequest {
                voice_type: voice_type.clone(),
                voice_id: voice_id.clone(),
            };
            match client.delete_voice(&req).await {
                Ok(_) => println!("音色已删除: {} ({})", voice_id, voice_type),
                Err(e) => { eprintln!("Error: {}", e); exit(1); }
            }
        }

        "list_files" => {
            if args.len() < 3 {
                eprintln!("Usage: list_files <purpose>");
                eprintln!("  purpose: voice_clone, prompt_audio, t2a_async_input, t2a_async, video_generation");
                exit(1);
            }
            let purpose = &args[2];
            match client.list_files(purpose).await {
                Ok(resp) => {
                    println!("{} 分类下共 {} 个文件:\n", purpose, resp.files.len());
                    for f in &resp.files {
                        println!("  file_id: {:?} — {:?} ({} bytes)",
                            f.file_id, f.filename,
                            f.bytes.map(|b| b.to_string()).unwrap_or_else(|| "?".to_string()));
                    }
                }
                Err(e) => { eprintln!("Error: {}", e); exit(1); }
            }
        }

        "retrieve_file" => {
            if args.len() < 3 {
                eprintln!("Usage: retrieve_file <file_id>");
                exit(1);
            }
            let file_id: i64 = match args[2].parse() {
                Ok(id) => id,
                Err(_) => { eprintln!("Invalid file_id: {}", args[2]); exit(1); }
            };
            match client.retrieve_file_info(file_id).await {
                Ok(resp) => {
                    if let Some(f) = &resp.file {
                        println!("file_id: {:?}", f.file_id);
                        println!("filename: {:?}", f.filename);
                        println!("size: {} bytes", f.bytes.map(|b| b.to_string()).unwrap_or_else(|| "?".to_string()));
                        println!("download_url: {:?}", f.download_url);
                    } else {
                        println!("未找到文件信息");
                    }
                }
                Err(e) => { eprintln!("Error: {}", e); exit(1); }
            }
        }

        "delete_file" => {
            if args.len() < 4 {
                eprintln!("Usage: delete_file <file_id> <purpose>");
                exit(1);
            }
            let file_id: i64 = match args[2].parse() {
                Ok(id) => id,
                Err(_) => { eprintln!("Invalid file_id: {}", args[2]); exit(1); }
            };
            let purpose = &args[3];
            match client.delete_file(file_id, purpose).await {
                Ok(_) => println!("文件已删除: file_id={}, purpose={}", file_id, purpose),
                Err(e) => { eprintln!("Error: {}", e); exit(1); }
            }
        }

        "generate_video_agent" => {
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            if next_i >= args.len() {
                eprintln!("Usage: generate_video_agent [--file <path>] [--output-directory <dir>] <template_id> [--text-inputs v1,v2,...] [--media-inputs u1,u2,...]");
                exit(1);
            }
            let template_id = args[next_i].clone();
            let mut text_inputs: Option<Vec<TextInput>> = None;
            let mut media_inputs: Option<Vec<MediaInput>> = None;

            let mut i = next_i + 1;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--text-inputs" && i + 1 < args.len() {
                    text_inputs = Some(
                        args[i+1].split(',').map(|v| TextInput { value: v.trim().to_string() }).collect()
                    );
                    i += 2;
                } else if arg == "--media-inputs" && i + 1 < args.len() {
                    media_inputs = Some(
                        args[i+1].split(',').map(|v| MediaInput { value: v.trim().to_string() }).collect()
                    );
                    i += 2;
                } else if arg.starts_with("--") { eprintln!("Unknown option: {}", arg); exit(1); }
                else { i += 1; }
            }

            let req = VideoTemplateGenerationRequest { template_id, text_inputs, media_inputs, callback_url: None };
            match client.create_video_template(&req).await {
                Ok(resp) => {
                    let tid = resp.task_id.as_deref().unwrap_or("N/A");
                    println!("视频 Agent 任务已提交!");
                    println!("task_id: {}", tid);
                    println!("使用 query_video_agent {} 查询进度", tid);
                    if out_file.is_some() || out_dir.is_some() {
                        println!("提示: --file/--output-directory 在 query_video_agent 步骤生效");
                        if let Some(f) = &out_file { println!("  output_file = {}", f); }
                        if let Some(d) = &out_dir { println!("  output_directory = {}", d); }
                    }
                }
                Err(e) => { eprintln!("Error: {}", e); exit(1); }
            }
        }

        "query_video_agent" => {
            let (out_file, out_dir, next_i) = parse_output_flags(&args, 2);
            if next_i >= args.len() {
                eprintln!("Usage: query_video_agent [--file <path>] [--output-directory <dir>] <task_id>");
                exit(1);
            }
            let task_id = &args[next_i];
            let custom = out_file.is_some() || out_dir.is_some();
            match client.query_video_template(task_id).await {
                Ok(resp) => {
                    println!("状态: {}", resp.status);
                    if resp.status == "Success" {
                        if let Some(ref url) = resp.video_url {
                            if custom {
                                match utils::resolve_output_file(
                                    out_file.as_deref(),
                                    out_dir.as_deref(),
                                    "video_agent",
                                    task_id,
                                    "mp4",
                                ) {
                                    Ok(Some(path)) => {
                                        match client.download_to_path(url, &path).await {
                                            Ok(()) => println!("视频已保存: {}", path.display()),
                                            Err(e) => eprintln!("下载失败: {}", e),
                                        }
                                    }
                                    Ok(None) => println!("video_url: {}", url),
                                    Err(e) => eprintln!("路径解析失败: {}", e),
                                }
                            } else {
                                println!("video_url: {}", url);
                            }
                        }
                    } else if resp.status == "Fail" {
                        println!("任务失败");
                    }
                }
                Err(e) => { eprintln!("Error: {}", e); exit(1); }
            }
        }

        _ => {
            eprintln!("Unknown command: {}", args[1]);
            exit(1);
        }
    }
}