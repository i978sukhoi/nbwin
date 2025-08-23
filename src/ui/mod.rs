// 터미널 사용자 인터페이스(TUI) 관련 모듈들
// ratatui 라이브러리를 사용한 실시간 네트워크 모니터링 화면 구현

pub mod app;          // 기본 TUI 애플리케이션 구현 (app.rs)
pub mod app_improved; // 향상된 TUI 애플리케이션 구현 (app_improved.rs)
pub mod layout;       // 화면 레이아웃 관련 유틸리티 (layout.rs)
pub mod widgets;      // 커스텀 위젯들 (widgets/ 디렉토리)

// TUI 애플리케이션 구조체들을 외부에서 쉽게 사용할 수 있도록 re-export
pub use app::App;              // 기본 TUI 애플리케이션
pub use app_improved::ImprovedApp;  // 향상된 TUI 애플리케이션 (기본값)