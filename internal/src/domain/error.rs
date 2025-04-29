use std::error::Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageServiceError {
    #[error("Unable to find: {0}")]
    NotFound(String),
    #[error("There must be at least a one fermentation step")]
    NoFermentationStep,
    #[error("")]
    InvalidStepConfiguration(String),
    #[error(
        "Rate for step {0} is misconfigured, the final temperature after its execution would not match the whished targeted temperature"
    )]
    InvalidRateConfiguration(String),
    #[error("Invalid step position: {0} does not exist")]
    InvalidPosition(usize),
    #[error("Something wrong happened {0}")]
    TechnicalError(String),
}
