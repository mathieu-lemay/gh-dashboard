use std::fmt::Debug;

use gh_dashboard::Error;

#[derive(Debug, Error)]
pub struct AppError(String);

impl AppError {
    pub fn from_color_eyre(value: color_eyre::Report) -> Self {
        AppError(format!("{}", value))
    }
}

#[derive(Debug, Error)]
pub struct ServiceError(String);
