use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxObserverError {
    UnsupportedPlatform,
    CreateFailed,
    RegisterFailed,
    RunLoopUnavailable,
    ExistingWindowQueryFailed,
    NotificationNotObserved,
}

impl fmt::Display for AxObserverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedPlatform => "AX window observer is unsupported on this platform",
            Self::CreateFailed => "AX window observer could not be created",
            Self::RegisterFailed => "AX window-created notification could not be registered",
            Self::RunLoopUnavailable => "AX observer run loop is unavailable",
            Self::ExistingWindowQueryFailed => {
                "AX observer could not query existing application windows"
            }
            Self::NotificationNotObserved => "AX window-created notification was not observed",
        })
    }
}

impl std::error::Error for AxObserverError {}
