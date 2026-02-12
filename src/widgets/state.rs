use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded(chrono::DateTime<chrono::Local>),
    Error(String),
}

impl Display for LoadingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadingState::Loaded(time) => {
                write!(f, "Last refreshed at {}", time.format("%Y-%m-%d %H:%M:%S"))
            }
            _ => write!(f, "{:?}", self),
        }
    }
}
