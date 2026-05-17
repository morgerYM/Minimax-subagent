use std::env;
use std::process::exit;

use minimax_api::utils;
use minimax_api::MiniMaxClient;
use minimax_api::types::*;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // 支持 -h/--help
    if args.len() >= 2 && (args[1] == "-h" || args[1] == "--help") {
        eprintln!("Usage: ./minimax <command> [args...]");
        eprintln!("Commands:");
        eprintln!("  list_voices               - 列出所有音色（配合 grep 使用）");
        eprintln!("  query_usage              - 查询账户用量");
        eprintln!("  text_to_audio <text> [--voice voice_id] [--speed 0.5-2.0] [--pitch -12~12] [--emotion happy|sad|angry|...] - 文字转语音（自动播放）");
        eprintln!("    voice_id 示例: female-shaonv, male-qn-qingse, female-yujie, Chinese (Mandarin)_News_Anchor");
        eprintln!("  web_search <query>       - 网络搜索");
        eprintln!("  understand_image <prompt> <image_path> - 图片理解");
        eprintln!("  chat <message>           - 文本对话");
        eprintln!("  generate_image <prompt> [--aspect 1:1|16:9|4:3] - 图像生成（自动打开）");
        eprintln!("  generate_video <prompt>  - 视频生成");
        eprintln!("  generate_music <prompt> <lyrics> - 音乐生成（自动播放）");
        eprintln!("  generate_lyrics <style>  - 歌词生成");
        eprintln!("  voice_clone <voice_id> <audio_file> [text] - 音色克隆");
        eprintln!("  voice_design <prompt> <preview_text> [voice_id] - 音色设计");
        exit(0);
    }

    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin minimax -- <command> [args...]");
        eprintln!("Commands:");
        eprintln!("  list_voices               - 列出所有音色");
        eprintln!("  query_usage              - 查询账户用量");
        eprintln!("  text_to_audio <text>     - 文字转语音");
        eprintln!("  web_search <query>       - 网络搜索");
        eprintln!("  understand_image <prompt> <image_path> - 图片理解");
        eprintln!("  chat <message>           - 文本对话");
        eprintln!("  generate_image <prompt>   - 图像生成");
        eprintln!("  generate_video <prompt>  - 视频生成");
        eprintln!("  generate_music <prompt> <lyrics> - 音乐生成");
        eprintln!("  generate_lyrics <style>  - 歌词生成");
        eprintln!("  voice_clone <voice_id> <audio_file> [text] - 音色克隆");
        eprintln!("  voice_design <prompt> <preview_text> [voice_id] - 音色设计");
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
            // 简单参数解析：text --voice xxx --speed 1.5 --pitch 0 --emotion happy
            // 默认音色: female-yujie
            let mut voice_id = "female-yujie".to_string();
            let mut speed: Option<f64> = None;
            let mut pitch: Option<i32> = None;
            let mut emotion: Option<String> = None;
            let mut text: String = String::new();
            let mut found_text = false;

            let mut i = 2;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--voice" && i + 1 < args.len() {
                    voice_id = args[i + 1].clone();
                    i += 2;
                } else if arg == "--speed" && i + 1 < args.len() {
                    speed = args[i + 1].parse().ok();
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
                eprintln!("Usage: text_to_audio <text> [--voice voice_id] [--speed 0.5-2.0] [--pitch -12~12] [--emotion happy|sad|angry|...]");
                exit(1);
            }

            let req = T2ARequest {
                model: "speech-2.8-hd".to_string(),
                text: text.clone(),
                stream: Some(false),
                voice_setting: VoiceSetting {
                    voice_id,
                    speed,
                    vol: None,
                    pitch,
                    emotion,
                },
                audio_setting: AudioSetting {
                    sample_rate: 32000,
                    bitrate: 128000,
                    format: "mp3".to_string(),
                    channel: 1,
                },
                language_boost: Some("auto".to_string()),
                output_format: None,
            };
            match client.text_to_audio(&req).await {
                Ok(resp) => {
                    if let Some(data) = &resp.data {
                        if let Some(audio) = &data.audio {
                            match utils::save_and_play_audio(audio, "text_to_audio") {
                                Ok(path) => println!("音频已保存并播放: {}", path.display()),
                                Err(e) => println!("音频生成成功，保存/播放失败: {}", e),
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
            if args.len() < 3 {
                eprintln!("Usage: web_search <query>");
                exit(1);
            }
            let query = &args[2];
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
            if args.len() < 3 {
                eprintln!("Usage: chat <message>");
                exit(1);
            }
            let message = &args[2];
            let req = ChatRequest {
                model: "MiniMax-M2.7".to_string(),
                messages: vec![ChatMessage {
                    role: "user".to_string(),
                    content: message.clone(),
                }],
                system: None,
                max_tokens: Some(4096),
                temperature: None,
                top_p: None,
                stream: false,
            };
            match client.chat(&req).await {
                Ok(resp) => {
                    let text: Vec<String> = resp.content
                        .iter()
                        .filter(|b| b.block_type == "text")
                        .filter_map(|b| b.text.as_deref())
                        .map(String::from)
                        .collect();
                    let result = text.join("\n");
                    if result.is_empty() {
                        println!("聊天完成，但无文本输出。");
                    } else {
                        println!("{}", result);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "generate_image" => {
            if args.len() < 3 {
                eprintln!("Usage: generate_image <prompt>");
                exit(1);
            }
            let prompt = &args[2];
            let req = ImageGenerationRequest {
                model: "image-01".to_string(),
                prompt: prompt.clone(),
                aspect_ratio: None,
                n: Some(1),
                prompt_optimizer: Some(true),
            };
            match client.generate_image(&req).await {
                Ok(resp) => {
                    if let Some(data) = &resp.data {
                        println!("图像生成成功 (共 {} 张):", data.image_urls.len());
                        for url in &data.image_urls {
                            println!("  {}", url);
                            if let Ok(path) = utils::download_and_open_image(url, "generate_image").await {
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
            if args.len() < 3 {
                eprintln!("Usage: generate_video <prompt>");
                exit(1);
            }
            let prompt = &args[2];
            let req = VideoGenerationRequest {
                model: "MiniMax-Hailuo-2.3".to_string(),
                prompt: prompt.clone(),
                first_frame_image: None,
                duration: None,
                resolution: None,
            };
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

        "query_video" => {
            if args.len() < 3 {
                eprintln!("Usage: query_video <task_id>");
                exit(1);
            }
            let task_id = &args[2];
            match client.query_video(task_id).await {
                Ok(resp) => {
                    println!("任务状态: {}", resp.status);
                    if let Some(file_id) = &resp.file_id {
                        println!("file_id: {}", file_id);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "generate_music" => {
            if args.len() < 4 {
                eprintln!("Usage: generate_music <prompt> <lyrics>");
                exit(1);
            }
            let prompt = &args[2];
            let lyrics = &args[3];
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
                cover_feature_id: None,
                timbre: None,
            };
            match client.generate_music(&req).await {
                Ok(resp) => {
                    if let Some(data) = &resp.data {
                        if let Some(audio) = &data.audio {
                            match utils::save_and_play_audio(audio, "generate_music") {
                                Ok(path) => println!("音乐已保存并播放: {}", path.display()),
                                Err(e) => println!("音乐生成成功，保存/播放失败: {}", e),
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
            if args.len() < 4 {
                eprintln!("Usage: voice_clone <voice_id> <audio_file> [text]");
                exit(1);
            }
            let voice_id = &args[2];
            let audio_file = &args[3];
            let text = args.get(4).map(|s| s.as_str());

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
                text: text.map(String::from),
                model: None,
            };

            match client.voice_clone(&req).await {
                Ok(resp) => {
                    println!("音色克隆成功!");
                    println!("voice_id: {}", req.voice_id);
                    if let Some(demo) = resp.demo_audio {
                        println!("demo_audio: {}", demo);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        "voice_design" => {
            if args.len() < 4 {
                eprintln!("Usage: voice_design <prompt> <preview_text> [voice_id]");
                exit(1);
            }
            let prompt = &args[2];
            let preview_text = &args[3];
            let voice_id = args.get(4).map(|s| s.as_str());

            let req = VoiceDesignRequest {
                prompt: prompt.clone(),
                preview_text: preview_text.clone(),
                voice_id: voice_id.map(String::from),
            };
            match client.voice_design(&req).await {
                Ok(resp) => {
                    println!("音色设计成功!");
                    if let Some(id) = resp.voice_id {
                        println!("voice_id: {}", id);
                    }
                    if let Some(audio) = resp.trial_audio {
                        println!("trial_audio 长度: {} 字符", audio.len());
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }

        _ => {
            eprintln!("Unknown command: {}", args[1]);
            exit(1);
        }
    }
}