// 네트워크 관련 기능들을 담은 모듈 (mod.rs)
// 이 파일은 network 모듈의 진입점 역할

// 공통 모듈들 (크로스플랫폼)
pub mod interface; // 네트워크 인터페이스 정보 처리 (interface.rs)
pub mod parallel_stats;
pub mod public_ip; // Public IP 주소 조회 (public_ip.rs)
pub mod stats; // 네트워크 통계 및 대역폭 계산 (stats.rs) // 병렬 통계 수집 (parallel_stats.rs)

// 플랫폼별 API 모듈들 (조건부 컴파일)
#[cfg(windows)]
pub mod windows_api; // Windows API 호출 관련 기능 (windows_api.rs)

#[cfg(unix)]
pub mod linux_api; // Linux API 호출 관련 기능 (linux_api.rs)
