use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed during launch of default editor")]
    FailedEditorLaunch,

    #[error("failed to write content to temp file: {0}")]
    FailedWritingTempFileContent(std::io::Error),

    #[error("failed to create temp file: {0}")]
    FailedCreatingTempFile(std::io::Error),

    #[error("no command was provided")]
    NoCommandProvided,
}
