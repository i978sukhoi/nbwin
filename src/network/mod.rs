// 네트워크 관련 기능들을 담은 모듈 (mod.rs)
// 이 파일은 network 모듈의 진입점 역할

// pub mod: 공개 서브모듈 선언
pub mod interface;    // 네트워크 인터페이스 정보 처리 (interface.rs)
pub mod stats;        // 네트워크 통계 및 대역폭 계산 (stats.rs)  
pub mod windows_api;  // Windows API 호출 관련 기능 (windows_api.rs)