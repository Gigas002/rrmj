use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown rules profile: {0}")]
    UnknownRulesProfile(String),
}
