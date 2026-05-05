use thiserror::Error;

#[derive(Error, Debug)]
pub enum MiniMaxError {
    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("API error {code}: {message}")]
    Api {
        code: i32,
        message: String,
        trace_id: Option<String>,
    },

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("missing environment variable: {0}")]
    MissingEnv(String),

    #[error("task timeout after {max_retries} retries (task_id: {task_id})")]
    TaskTimeout {
        task_id: String,
        max_retries: i32,
    },

    #[error("task failed (task_id: {task_id})")]
    TaskFailed { task_id: String },

    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
