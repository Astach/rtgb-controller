use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandSchedulerServiceError {
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
    #[error("Unable to convert {0} to {1}")]
    ConversionError(&'static str, &'static str),
}

#[derive(Error, Debug)]
pub enum CommandExecutorServiceError {
    #[error("Unable to find: {0}")]
    NotFound(String),
    #[error("Something wrong happened {0}")]
    TechnicalError(String),
    #[error("Only Planned command can be executed")]
    StatusError,
}
