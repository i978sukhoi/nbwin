// 라이브러리의 메인 파일 (lib.rs)
// 다른 모듈들을 선언하고 외부에서 사용할 수 있도록 re-export

// pub mod: 공개 모듈 선언 - 다른 파일에서 접근 가능
pub mod network;  // src/network/ 디렉토리의 모듈
pub mod utils;    // src/utils/ 디렉토리의 모듈  
pub mod ui;       // src/ui/ 디렉토리의 모듈
pub mod error;    // src/error.rs 에러 처리 모듈

// pub use: re-export - 이 라이브러리를 사용하는 코드에서 쉽게 접근할 수 있도록 함
// 예: nbwin::NetworkInterface 대신 use nbwin::NetworkInterface로 바로 사용 가능
pub use network::interface::NetworkInterface;  // 네트워크 인터페이스 구조체
pub use network::stats::InterfaceStats;        // 인터페이스 통계 구조체
pub use ui::{App, ImprovedApp};                // TUI 애플리케이션 구조체들