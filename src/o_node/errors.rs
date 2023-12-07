use thiserror::Error;

#[derive(Debug, Error)]
pub enum VideoQueryError {
    #[error("Error deserialing query")]
    ErrorDeserializingQuery,
    #[error("Error deserialing answer")]
    ErrorDeserializingAnswer,
}
