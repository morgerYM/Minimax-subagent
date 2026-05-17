use std::env;
use std::process::exit;

use minimax_api::MiniMaxClient;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin test_mcp -- <command> [args...]");
        eprintln!("Commands:");
        eprintln!("  list_voices               - 列出所有音色");
        eprintln!("  query_usage              - 查询账户用量");
        eprintln!("  text_to_audio <text>     - 文字转语音");
        eprintln!("  web_search <query>       - 网络搜索");
        eprintln!("  understand_image <prompt> <image_path> - 图片理解");
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
            if args.len() < 3 {
                eprintln!("Usage: text_to_audio <text>");
                exit(1);
            }
            let text = &args[2];
            let req = minimax_api::types::T2ARequest {
                model: "speech-2.8-hd".to_string(),
                text: text.clone(),
                stream: Some(false),
                voice_setting: minimax_api::types::VoiceSetting {
                    voice_id: "female-shaonv".to_string(),
                    speed: None,
                    vol: None,
                    pitch: None,
                    emotion: None,
                },
                audio_setting: minimax_api::types::AudioSetting {
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
                            println!("音频生成成功，数据长度: {} 字符", audio.len());
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
            let req = minimax_api::types::SearchRequest { q: query.clone() };
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

            let req = minimax_api::types::VlmRequest {
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

        _ => {
            eprintln!("Unknown command: {}", args[1]);
            exit(1);
        }
    }
}