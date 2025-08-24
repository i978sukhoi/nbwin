// 데이터 크기와 속도를 사용자 친화적인 문자열로 포맷팅하는 유틸리티 함수들
use bytesize::ByteSize; // 바이트 크기를 사람이 읽기 쉬운 형태로 변환해주는 외부 라이브러리

// 바이트 수를 사람이 읽기 쉬운 형태로 포맷팅 (예: 1024 → "1.0 KB")
pub fn format_bytes(bytes: u64) -> String {
    // ByteSize::b(): u64 바이트 값으로 ByteSize 인스턴스 생성
    // to_string_as(true): true는 이진 단위(1024) 사용을 의미
    ByteSize::b(bytes).to_string_as(true)
}

// 초당 바이트 수를 포맷팅 (예: 1536.0 → "1.5 KB/s")
pub fn format_bytes_per_sec(bytes_per_sec: f64) -> String {
    // if-else if 체인: 값의 크기에 따라 적절한 단위 선택
    if bytes_per_sec < 1024.0 {
        // {:.1}: 소수점 첫째자리까지 표시
        format!("{:.1} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        // 1 MB 미만
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else if bytes_per_sec < 1024.0 * 1024.0 * 1024.0 {
        // 1 GB 미만
        format!("{:.1} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else {
        // 1 GB 이상
        format!("{:.1} GB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
    }
}

// 초당 비트 수를 포맷팅 - 네트워크 속도 표시용 (예: 1000000 → "1.0 Mbps")
// 비트 단위는 십진법(1000) 사용이 일반적
pub fn format_bits_per_sec(bits_per_sec: u64) -> String {
    if bits_per_sec < 1000 {
        format!("{} bps", bits_per_sec) // bits per second
    } else if bits_per_sec < 1_000_000 {
        // 1M 미만
        // as f64: u64를 f64로 타입 변환 (나눗셈을 위해)
        format!("{:.1} Kbps", bits_per_sec as f64 / 1000.0)
    } else if bits_per_sec < 1_000_000_000 {
        // 1G 미만
        // 1_000_000: 가독성을 위한 숫자 구분자 (1000000와 동일)
        format!("{:.1} Mbps", bits_per_sec as f64 / 1_000_000.0)
    } else {
        // 1G 이상
        format!("{:.1} Gbps", bits_per_sec as f64 / 1_000_000_000.0)
    }
}
