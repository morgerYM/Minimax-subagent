//! Binary-side tool factory registrations.
//!
//! Instead of a central `McpToolDispatcher` with a 22-arm `match` block,
//! each capability area (TTS, video, image, …) registers its tools as
//! self-contained [`AgentTool`]s via a factory function.
//!
//! Each [`AgentTool`] bundles its JSON schema (what the LLM sees) with its
//! execution logic (closed over the provider `Arc`). The agent loop never
//! knows which provider is behind a tool — it just calls `execute()`.
//!
//! ## Design
//!
//! - Capability tools (TTS, video, image, …) are registered via factories.
//! - The `run_subagent` tool is special: it needs the *resolved* tool list
//!   to filter tools for recursive invocations. It is therefore built via
//!   [`build_run_subagent_tool`] AFTER resolving all capability tools.

use std::sync::Arc;

use minimax_api::mcp_params::*;
use minimax_api::subagent::{
    tools_for_subagent, AgentTool, DispatchResult, NoParams, SubagentDef,
    SubagentRegistry, ToolRegistry, call_tool_result_to_dispatch, parse_input, run_agent_loop,
    schema_of, to_tool_err, LoopResult, RUN_SUBAGENT_NAME,
};
use minimax_api::tools::{chat, files, image, music, search, tts, usage, video};
use minimax_api::MiniMaxClient;

// ============================================================
// Top-level: register all capability tools (NOT run_subagent)
// ============================================================

/// Register all MiniMax-provided **capability** tools (TTS, video, image,
/// music, chat, search, files, usage) into the registry.
///
/// The `run_subagent` tool is NOT registered here — it depends on the
/// resolved tool list and must be built via [`build_run_subagent_tool`]
/// after resolving the capability tools.
pub fn register_minimax_capability_tools(
    registry: &mut ToolRegistry,
    tts: Arc<dyn tts::TtsProvider>,
    voice: Arc<dyn tts::VoiceProvider>,
    video: Arc<dyn video::VideoProvider>,
    image: Arc<dyn image::ImageProvider>,
    music: Arc<dyn music::MusicProvider>,
    chat: Arc<dyn chat::ChatProvider>,
    search: Arc<dyn search::SearchProvider>,
    files: Arc<dyn files::FileProvider>,
    usage: Arc<dyn usage::UsageProvider>,
) {
    register_tts_tools(registry, tts);
    register_voice_tools(registry, voice);
    register_video_tools(registry, video);
    register_image_tools(registry, image);
    register_understand_image_tool(registry, search.clone());
    register_music_tools(registry, music);
    register_chat_tool(registry, chat);
    register_web_search_tool(registry, search);
    register_files_tools(registry, files);
    register_usage_tool(registry, usage);
}

/// Build the `run_subagent` tool, which depends on the **resolved** tool
/// list so it can filter tools for recursive invocations.
pub fn build_run_subagent_tool(
    client: MiniMaxClient,
    subagent_registry: Arc<SubagentRegistry>,
    all_tools: Vec<AgentTool>,
) -> AgentTool {
    AgentTool::new(
        RUN_SUBAGENT_NAME,
        "Delegate a sub-task to a named subagent (defined in subagents/<name>.json). \
         The subagent runs its own agent loop with its own system prompt and can itself \
         call any tool. Returns the subagent's final text output. Use this to break a \
         complex task into specialized sub-tasks.",
        schema_of::<RunSubagentParams>(),
        {
            Arc::new(move |input, current_depth| {
                let client = client.clone();
                let registry = subagent_registry.clone();
                let tools = all_tools.clone();
                Box::pin(async move {
                    let params: RunSubagentParams = parse_input(input)?;

                    let sub: &SubagentDef = registry.get(&params.name).ok_or_else(|| {
                        minimax_api::MiniMaxError::Config(format!(
                            "subagent '{}' not found. Use list_subagents to see available subagents.",
                            params.name
                        ))
                    })?;

                    // Determine effective allowed_tools: param overrides config
                    let sub_tools = if let Some(tool_override) = &params.allowed_tools {
                        let mut effective = sub.clone();
                        effective.allowed_tools = Some(tool_override.clone());
                        tools_for_subagent(&tools, &effective)
                    } else {
                        tools_for_subagent(&tools, sub)
                    };

                    let result: LoopResult = run_agent_loop(
                        &client,
                        sub,
                        &params.task,
                        current_depth + 1,
                        &sub_tools,
                    )
                    .await?;

                    Ok(DispatchResult {
                        output: result.final_output,
                        is_error: false,
                    })
                })
            })
        },
    )
}

// ============================================================
// TTS tools
// ============================================================

fn register_tts_tools(registry: &mut ToolRegistry, tts: Arc<dyn tts::TtsProvider>) {
    registry.register_fn("tts", move |_ctx| {
        let p = tts.clone();
        vec![
            AgentTool::new(
                "text_to_audio",
                "Convert text to speech. Returns audio hex data inline, or saves to a file when `output_directory` is set.",
                schema_of::<TextToAudioParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: TextToAudioParams = parse_input(input)?;
                            let r = tts::handle_text_to_audio(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "text_to_audio_stream",
                "WebSocket streaming text-to-speech with low first-byte latency. Best for real-time voice output.",
                schema_of::<TextToAudioStreamParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: TextToAudioStreamParams = parse_input(input)?;
                            let r = tts::handle_text_to_audio_stream(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "generate_audio_async",
                "Submit an async TTS task (up to 50,000 characters). Returns a task_id; poll with `query_audio_task`.",
                schema_of::<GenerateAudioAsyncParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: GenerateAudioAsyncParams = parse_input(input)?;
                            let r = tts::handle_generate_audio_async(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "query_audio_task",
                "Poll an async TTS task. When complete, automatically downloads and extracts the mp3.",
                schema_of::<QueryAudioTaskParams>(),
                {
                    let p = p;
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: QueryAudioTaskParams = parse_input(input)?;
                            let r = tts::handle_query_audio_task(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
        ]
    });
}

// ============================================================
// Voice tools
// ============================================================

fn register_voice_tools(registry: &mut ToolRegistry, voice: Arc<dyn tts::VoiceProvider>) {
    registry.register_fn("voice", move |_ctx| {
        let p = voice.clone();
        vec![
            AgentTool::new(
                "list_voices",
                "List all available voices (system + cloned + designed).",
                schema_of::<ListVoicesParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: ListVoicesParams = parse_input(input)?;
                            let r = tts::handle_list_voices(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "voice_clone",
                "Clone a voice from a reference audio file. The new voice is reusable via its `voice_id`.",
                schema_of::<VoiceCloneParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: VoiceCloneParams = parse_input(input)?;
                            let r = tts::handle_voice_clone(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "voice_design",
                "Design a brand-new voice from a text description. Returns a `voice_id` and a trial audio.",
                schema_of::<VoiceDesignParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: VoiceDesignParams = parse_input(input)?;
                            let r = tts::handle_voice_design(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "delete_voice",
                "Delete a previously cloned or designed voice.",
                schema_of::<DeleteVoiceParams>(),
                {
                    let p = p;
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: DeleteVoiceParams = parse_input(input)?;
                            let r = tts::handle_delete_voice(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
        ]
    });
}

// ============================================================
// Video tools
// ============================================================

fn register_video_tools(registry: &mut ToolRegistry, video: Arc<dyn video::VideoProvider>) {
    registry.register_fn("video", move |_ctx| {
        let p = video.clone();
        vec![
            AgentTool::new(
                "generate_video",
                "Generate a video from a prompt (or first-frame image). Default async — returns task_id; set `async_mode=false` to wait.",
                schema_of::<GenerateVideoParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: GenerateVideoParams = parse_input(input)?;
                            let r = video::handle_generate_video(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "query_video",
                "Poll a video generation task. When complete, returns a download URL or saves to `output_directory`.",
                schema_of::<QueryVideoParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: QueryVideoParams = parse_input(input)?;
                            let r = video::handle_query_video(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "generate_video_agent",
                "Run a pre-built video template (a 'video agent') that takes text + media inputs.",
                schema_of::<GenerateVideoAgentParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: GenerateVideoAgentParams = parse_input(input)?;
                            let r = video::handle_generate_video_agent(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "query_video_agent",
                "Poll a video-agent (template) task.",
                schema_of::<QueryVideoAgentParams>(),
                {
                    let p = p;
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: QueryVideoAgentParams = parse_input(input)?;
                            let r = video::handle_query_video_agent(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
        ]
    });
}

// ============================================================
// Image tools
// ============================================================

fn register_image_tools(registry: &mut ToolRegistry, image: Arc<dyn image::ImageProvider>) {
    registry.register_fn("image", move |_ctx| {
        let p = image.clone();
        vec![AgentTool::new(
            "generate_image",
            "Generate one or more images from a text prompt. Returns image URLs (or saves to `output_directory`).",
            schema_of::<GenerateImageParams>(),
            {
                Arc::new(move |input, _depth| {
                    let p = p.clone();
                    Box::pin(async move {
                        let params: GenerateImageParams = parse_input(input)?;
                        let r = image::handle_generate_image(&*p, params).await.map_err(to_tool_err)?;
                        Ok(call_tool_result_to_dispatch(r))
                    })
                })
            },
        )]
    });
}

fn register_understand_image_tool(
    registry: &mut ToolRegistry,
    search: Arc<dyn search::SearchProvider>,
) {
    registry.register_fn("understand-image", move |_ctx| {
        let p = search.clone();
        vec![AgentTool::new(
            "understand_image",
            "Analyze an image with the VLM model. Pass an HTTP(S) URL or local file path.",
            schema_of::<UnderstandImageParams>(),
            {
                Arc::new(move |input, _depth| {
                    let p = p.clone();
                    Box::pin(async move {
                        let params: UnderstandImageParams = parse_input(input)?;
                        let r = search::handle_understand_image(&*p, params).await.map_err(to_tool_err)?;
                        Ok(call_tool_result_to_dispatch(r))
                    })
                })
            },
        )]
    });
}

// ============================================================
// Music tools
// ============================================================

fn register_music_tools(registry: &mut ToolRegistry, music: Arc<dyn music::MusicProvider>) {
    registry.register_fn("music", move |_ctx| {
        let p = music.clone();
        vec![
            AgentTool::new(
                "generate_music",
                "Generate a song from a style prompt and lyrics (with optional [Verse]/[Chorus] tags).",
                schema_of::<GenerateMusicParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: GenerateMusicParams = parse_input(input)?;
                            let r = music::handle_generate_music(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "generate_lyrics",
                "Write full song lyrics, or edit/continue existing ones.",
                schema_of::<GenerateLyricsParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: GenerateLyricsParams = parse_input(input)?;
                            let r = music::handle_generate_lyrics(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "generate_music_cover",
                "Generate a cover of a reference audio. Optional custom lyrics and style prompt.",
                schema_of::<GenerateMusicCoverParams>(),
                {
                    let p = p;
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: GenerateMusicCoverParams = parse_input(input)?;
                            let r = music::handle_generate_music_cover(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
        ]
    });
}

// ============================================================
// Chat tool
// ============================================================

fn register_chat_tool(registry: &mut ToolRegistry, chat: Arc<dyn chat::ChatProvider>) {
    registry.register_fn("chat", move |_ctx| {
        let p = chat.clone();
        vec![AgentTool::new(
            "chat",
            "Single-turn text chat with a MiniMax LLM. Useful for reasoning, summarization, or text generation without tool use.",
            schema_of::<ChatParams>(),
            {
                Arc::new(move |input, _depth| {
                    let p = p.clone();
                    Box::pin(async move {
                        let params: ChatParams = parse_input(input)?;
                        let r = chat::handle_chat(&*p, params).await.map_err(to_tool_err)?;
                        Ok(call_tool_result_to_dispatch(r))
                    })
                })
            },
        )]
    });
}

// ============================================================
// Web search tool
// ============================================================

fn register_web_search_tool(
    registry: &mut ToolRegistry,
    search: Arc<dyn search::SearchProvider>,
) {
    registry.register_fn("web-search", move |_ctx| {
        let p = search.clone();
        vec![AgentTool::new(
            "web_search",
            "Web search via MiniMax Coding Plan. Returns organic results + related searches.",
            schema_of::<WebSearchParams>(),
            {
                Arc::new(move |input, _depth| {
                    let p = p.clone();
                    Box::pin(async move {
                        let params: WebSearchParams = parse_input(input)?;
                        let r = search::handle_web_search(&*p, params).await.map_err(to_tool_err)?;
                        Ok(call_tool_result_to_dispatch(r))
                    })
                })
            },
        )]
    });
}

// ============================================================
// Files tools
// ============================================================

fn register_files_tools(registry: &mut ToolRegistry, files: Arc<dyn files::FileProvider>) {
    registry.register_fn("files", move |_ctx| {
        let p = files.clone();
        vec![
            AgentTool::new(
                "list_files",
                "List files uploaded to MiniMax under a given purpose (e.g. voice_clone, t2a_async_input).",
                schema_of::<ListFilesParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: ListFilesParams = parse_input(input)?;
                            let r = files::handle_list_files(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "retrieve_file",
                "Get metadata + download URL for a previously uploaded MiniMax file.",
                schema_of::<RetrieveFileParams>(),
                {
                    let p = p.clone();
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: RetrieveFileParams = parse_input(input)?;
                            let r = files::handle_retrieve_file(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
            AgentTool::new(
                "delete_file",
                "Delete a MiniMax file by id and purpose.",
                schema_of::<DeleteFileParams>(),
                {
                    let p = p;
                    Arc::new(move |input, _depth| {
                        let p = p.clone();
                        Box::pin(async move {
                            let params: DeleteFileParams = parse_input(input)?;
                            let r = files::handle_delete_file(&*p, params).await.map_err(to_tool_err)?;
                            Ok(call_tool_result_to_dispatch(r))
                        })
                    })
                },
            ),
        ]
    });
}

// ============================================================
// Usage tool
// ============================================================

fn register_usage_tool(registry: &mut ToolRegistry, usage: Arc<dyn usage::UsageProvider>) {
    registry.register_fn("usage", move |_ctx| {
        let p = usage.clone();
        vec![AgentTool::new(
            "query_usage",
            "Query account token balance and usage summary.",
            schema_of::<NoParams>(),
            {
                Arc::new(move |_input, _depth| {
                    let p = p.clone();
                    Box::pin(async move {
                        let r = usage::handle_query_usage(&*p).await.map_err(to_tool_err)?;
                        Ok(call_tool_result_to_dispatch(r))
                    })
                })
            },
        )]
    });
}
