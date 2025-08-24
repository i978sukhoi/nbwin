// 커스텀 에러 타입과 에러 처리 유틸리티
//
// 이 모듈은 애플리케이션 전체에서 사용되는 에러 타입과 헬퍼 함수들을 정의합니다.

use anyhow::{Context, Result};
use std::fmt;

/// 애플리케이션 에러 타입
#[derive(Debug)]
pub enum AppError {
    /// 네트워크 인터페이스 관련 에러
    NetworkInterface(String),
    /// 터미널 UI 관련 에러
    Terminal(String),
    /// 설정 관련 에러
    Configuration(String),
    /// 시스템 API 관련 에러
    SystemApi(String),
    /// 권한 관련 에러
    Permission(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NetworkInterface(msg) => write!(f, "Network interface error: {}", msg),
            AppError::Terminal(msg) => write!(f, "Terminal error: {}", msg),
            AppError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            AppError::SystemApi(msg) => write!(f, "System API error: {}", msg),
            AppError::Permission(msg) => write!(f, "Permission error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// 에러에 컨텍스트를 추가하는 헬퍼 trait
pub trait ErrorContext<T> {
    /// 에러에 사용자 친화적인 컨텍스트를 추가
    fn user_context(self, msg: &str) -> Result<T>;

    /// 에러에 기술적 컨텍스트를 추가
    fn tech_context(self, msg: &str) -> Result<T>;
}

impl<T> ErrorContext<T> for Result<T> {
    fn user_context(self, msg: &str) -> Result<T> {
        self.with_context(|| format!("❌ {}", msg))
    }

    fn tech_context(self, msg: &str) -> Result<T> {
        self.with_context(|| format!("[Technical] {}", msg))
    }
}

/// 디버그 정보를 로깅하는 매크로
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

/// 에러를 로깅하고 컨텍스트를 추가하는 매크로
#[macro_export]
macro_rules! log_error {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(val) => Ok(val),
            Err(e) => {
                eprintln!("Error occurred: {} - Context: {}", e, $context);
                Err(e).context($context)
            }
        }
    };
}

/// 복구 가능한 에러를 처리하는 헬퍼 함수
pub fn handle_recoverable_error<T>(result: Result<T>, default: T, context: &str) -> T {
    match result {
        Ok(val) => val,
        Err(e) => {
            eprintln!(
                "⚠️  Recoverable error: {} - Using default value. Context: {}",
                e, context
            );
            default
        }
    }
}

/// 시스템 API 에러를 처리하는 헬퍼 함수
#[cfg(target_os = "windows")]
pub fn handle_windows_error(error_code: u32, operation: &str) -> AppError {
    use windows::core::Error;

    let win_error = Error::from_win32();
    let message = format!(
        "Windows API error during {}: Code {} - {}",
        operation,
        error_code,
        win_error.message()
    );

    AppError::SystemApi(message)
}

#[cfg(target_os = "linux")]
pub fn handle_linux_error(errno: i32, operation: &str) -> AppError {
    let message = format!(
        "Linux system error during {}: {} (errno: {})",
        operation,
        std::io::Error::from_raw_os_error(errno),
        errno
    );

    AppError::SystemApi(message)
}
