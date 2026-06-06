//! Static catalog of all tools the LLM can invoke during an agent loop.
//!
//! Each entry is a [`ToolSpec`] sent to the LLM in the `tools` field of
//! the chat request. The 23 existing MiniMax tools come from `mcp_params`,
//! plus the new `run_subagent` tool that enables subagent composition.
//!
//! NOTE: descriptions here are what the **LLM** sees (English, focused on
//! "when to call this tool"). They may differ from the `#[tool(description)]`
//! attributes in `src/main.rs`, which surface in the MCP `tools/list` for
//! human users. Keep both in sync if a tool's purpose changes.

use schemars::schema_for;
use schemars::JsonSchema;

use crate::mcp_params::*;
use crate::types::ToolSpec;

use super::types::SubagentDef;

/// Name of the recursive subagent tool. Always implicitly available to
/// every subagent, regardless of `allowed_tools` whitelist.
pub const RUN_SUBAGENT_NAME: &str = "run_subagent";

/// Sentinel Param type for tools that take no input.
#[derive(Debug, Clone, Default, serde::Deserialize, JsonSchema)]
pub struct NoParams {}

/// Build a `ToolSpec` for any type that implements `JsonSchema`.
fn spec<P: JsonSchema>(name: &str, description: &str) -> ToolSpec {
    ToolSpec {
        name: name.to_string(),
        description: description.to_string(),
        input_schema: serde_json::to_value(schema_for!(P))
            .expect("schema_for! must produce a serializable schema"),
    }
}

/// All 24 tools the LLM can see.
pub fn all_tool_specs() -> Vec<ToolSpec> {
    vec![
        // ---------------- TTS / Voice ----------------
        spec::<TextToAudioParams>(
            "text_to_audio",
            "Convert text to speech. Returns audio hex data inline, or saves to a file when `output_directory` is set.",
        ),
        spec::<TextToAudioStreamParams>(
            "text_to_audio_stream",
            "WebSocket streaming text-to-speech with low first-byte latency. Best for real-time voice output.",
        ),
        spec::<GenerateAudioAsyncParams>(
            "generate_audio_async",
            "Submit an async TTS task (up to 50,000 characters). Returns a task_id; poll with `query_audio_task`.",
        ),
        spec::<QueryAudioTaskParams>(
            "query_audio_task",
            "Poll an async TTS task. When complete, automatically downloads and extracts the mp3.",
        ),
        spec::<ListVoicesParams>(
            "list_voices",
            "List all available voices (system + cloned + designed).",
        ),
        spec::<VoiceCloneParams>(
            "voice_clone",
            "Clone a voice from a reference audio file. The new voice is reusable via its `voice_id`.",
        ),
        spec::<VoiceDesignParams>(
            "voice_design",
            "Design a brand-new voice from a text description. Returns a `voice_id` and a trial audio.",
        ),
        spec::<DeleteVoiceParams>(
            "delete_voice",
            "Delete a previously cloned or designed voice.",
        ),
        // ---------------- Image ----------------
        spec::<GenerateImageParams>(
            "generate_image",
            "Generate one or more images from a text prompt. Returns image URLs (or saves to `output_directory`).",
        ),
        spec::<UnderstandImageParams>(
            "understand_image",
            "Analyze an image with the VLM model. Pass an HTTP(S) URL or local file path.",
        ),
        // ---------------- Video ----------------
        spec::<GenerateVideoParams>(
            "generate_video",
            "Generate a video from a prompt (or first-frame image). Default async — returns task_id; set `async_mode=false` to wait.",
        ),
        spec::<QueryVideoParams>(
            "query_video",
            "Poll a video generation task. When complete, returns a download URL or saves to `output_directory`.",
        ),
        spec::<GenerateVideoAgentParams>(
            "generate_video_agent",
            "Run a pre-built video template (a 'video agent') that takes text + media inputs.",
        ),
        spec::<QueryVideoAgentParams>(
            "query_video_agent",
            "Poll a video-agent (template) task.",
        ),
        // ---------------- Music ----------------
        spec::<GenerateMusicParams>(
            "generate_music",
            "Generate a song from a style prompt and lyrics (with optional [Verse]/[Chorus] tags).",
        ),
        spec::<GenerateLyricsParams>(
            "generate_lyrics",
            "Write full song lyrics, or edit/continue existing ones.",
        ),
        spec::<GenerateMusicCoverParams>(
            "generate_music_cover",
            "Generate a cover of a reference audio. Optional custom lyrics and style prompt.",
        ),
        // ---------------- Chat / Search ----------------
        spec::<ChatParams>(
            "chat",
            "Single-turn text chat with a MiniMax LLM. Useful for reasoning, summarization, or text generation without tool use.",
        ),
        spec::<WebSearchParams>(
            "web_search",
            "Web search via MiniMax Coding Plan. Returns organic results + related searches.",
        ),
        // ---------------- Files ----------------
        spec::<ListFilesParams>(
            "list_files",
            "List files uploaded to MiniMax under a given purpose (e.g. voice_clone, t2a_async_input).",
        ),
        spec::<RetrieveFileParams>(
            "retrieve_file",
            "Get metadata + download URL for a previously uploaded MiniMax file.",
        ),
        spec::<DeleteFileParams>(
            "delete_file",
            "Delete a MiniMax file by id and purpose.",
        ),
        // ---------------- Account ----------------
        spec::<NoParams>(
            "query_usage",
            "Query account token balance and usage summary.",
        ),
        // ---------------- Subagent (recursive) ----------------
        spec::<RunSubagentParams>(
            RUN_SUBAGENT_NAME,
            "Delegate a sub-task to a named subagent (defined in subagents/<name>.json). The subagent runs its own agent loop with its own system prompt and can itself call any tool. Returns the subagent's final text output. Use this to break a complex task into specialized sub-tasks.",
        ),
    ]
}

/// Tools a specific subagent is allowed to call.
///
/// `run_subagent` is always included, even if the subagent's
/// `allowed_tools` whitelist doesn't list it. If `allowed_tools` is
/// `None`, every tool is included.
pub fn specs_for(subagent: &SubagentDef) -> Vec<ToolSpec> {
    let allowed = subagent.allowed_tools.as_ref();
    all_tool_specs()
        .into_iter()
        .filter(|s| {
            if s.name == RUN_SUBAGENT_NAME {
                return true;
            }
            match allowed {
                None => true,
                Some(whitelist) => whitelist.iter().any(|name| name == &s.name),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subagent::types::SubagentDef;

    fn make_subagent(name: &str, allowed: Option<Vec<&str>>) -> SubagentDef {
        SubagentDef {
            name: name.to_string(),
            description: "test".to_string(),
            system: "test".to_string(),
            model: None,
            max_tokens: None,
            temperature: None,
            max_iterations: None,
            allowed_tools: allowed.map(|v| v.into_iter().map(String::from).collect()),
            max_depth: None,
        }
    }

    #[test]
    fn all_tool_specs_count_and_run_subagent() {
        let all = all_tool_specs();
        // 22 existing MCP tools + 1 new (run_subagent)
        assert_eq!(all.len(), 24, "expected 24 tools, got {}", all.len());
        assert!(all.iter().any(|s| s.name == RUN_SUBAGENT_NAME));
    }

    #[test]
    fn no_whitelist_returns_all() {
        let sub = make_subagent("any", None);
        let specs = specs_for(&sub);
        assert_eq!(specs.len(), 24);
    }

    #[test]
    fn whitelist_filters_correctly() {
        let sub = make_subagent("limited", Some(vec!["text_to_audio", "generate_image"]));
        let specs = specs_for(&sub);
        let names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
        assert!(names.contains(&"text_to_audio".to_string()));
        assert!(names.contains(&"generate_image".to_string()));
        assert!(!names.contains(&"generate_video".to_string()));
        // run_subagent is always present
        assert!(names.contains(&RUN_SUBAGENT_NAME.to_string()));
    }

    #[test]
    fn empty_whitelist_still_allows_run_subagent() {
        let sub = make_subagent("none", Some(vec![]));
        let specs = specs_for(&sub);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, RUN_SUBAGENT_NAME);
    }

    #[test]
    fn unknown_tool_name_in_whitelist_is_silently_ignored() {
        let sub = make_subagent(
            "typo",
            Some(vec!["text_to_audio", "fake_tool_doesnt_exist"]),
        );
        let specs = specs_for(&sub);
        let names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
        // text_to_audio kept, fake dropped, run_subagent auto-included
        assert!(names.contains(&"text_to_audio".to_string()));
        assert!(!names.contains(&"fake_tool_doesnt_exist".to_string()));
        assert!(names.contains(&RUN_SUBAGENT_NAME.to_string()));
        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn tool_spec_schemas_are_objects() {
        let all = all_tool_specs();
        for spec in &all {
            // Each input_schema should be a JSON object (not array/null/etc.)
            assert!(
                spec.input_schema.is_object(),
                "tool '{}' input_schema is not a JSON object: {:?}",
                spec.name,
                spec.input_schema
            );
        }
    }
}
