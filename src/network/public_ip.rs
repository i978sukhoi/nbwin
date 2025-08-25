// Public IP 주소를 가져오는 모듈
// 외부 HTTP API를 사용하여 공인 IP 주소를 조회

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// 캐시 구조체 - Public IP를 캐싱하여 과도한 API 호출 방지
struct PublicIpCache {
    ip: Option<String>,
    last_updated: Option<Instant>,
    cache_duration: Duration,
}

impl PublicIpCache {
    fn new() -> Self {
        Self {
            ip: None,
            last_updated: None,
            cache_duration: Duration::from_secs(300), // 5분 캐시
        }
    }

    fn is_valid(&self) -> bool {
        if let Some(last_updated) = self.last_updated {
            last_updated.elapsed() < self.cache_duration
        } else {
            false
        }
    }

    fn get(&self) -> Option<String> {
        if self.is_valid() {
            self.ip.clone()
        } else {
            None
        }
    }

    fn set(&mut self, ip: String) {
        self.ip = Some(ip);
        self.last_updated = Some(Instant::now());
    }
}

// 전역 캐시 인스턴스
static CACHE: Lazy<Mutex<PublicIpCache>> = Lazy::new(|| Mutex::new(PublicIpCache::new()));

// Public IP 서비스 목록 (폴백 지원)
const IP_SERVICES: &[&str] = &[
    "https://api.ipify.org",         // 가장 간단하고 빠름
    "https://checkip.amazonaws.com", // AWS 제공, 신뢰성 높음
    "https://ipinfo.io/ip",          // 추가 정보 제공 가능
    "https://ifconfig.me/ip",        // 전통적인 서비스
];

/// Public IP 주소를 가져오는 함수
/// 캐시된 값이 있으면 반환, 없으면 API 호출
pub fn get_public_ip() -> Option<String> {
    // 먼저 캐시 확인
    {
        let cache = CACHE.lock().unwrap();
        if let Some(cached_ip) = cache.get() {
            return Some(cached_ip);
        }
    }

    // 캐시가 없거나 만료되었으면 새로 가져오기
    if let Some(ip) = fetch_public_ip() {
        let mut cache = CACHE.lock().unwrap();
        cache.set(ip.clone());
        Some(ip)
    } else {
        None
    }
}

/// 실제로 외부 API를 호출하여 Public IP를 가져오는 함수
fn fetch_public_ip() -> Option<String> {
    // HTTP 클라이언트 생성 (타임아웃 설정)
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .ok()?;

    // 각 서비스를 순차적으로 시도
    for service_url in IP_SERVICES {
        match try_fetch_from_service(&client, service_url) {
            Ok(ip) => {
                // 유효한 IP 주소인지 간단히 검증
                if is_valid_ip(&ip) {
                    return Some(ip);
                }
            }
            Err(_) => continue, // 다음 서비스 시도
        }
    }

    None
}

/// 특정 서비스에서 IP 주소를 가져오는 함수
fn try_fetch_from_service(client: &reqwest::blocking::Client, url: &str) -> Result<String> {
    let response = client.get(url).send().context("Failed to send request")?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP request failed with status: {}", response.status());
    }

    let ip = response
        .text()
        .context("Failed to read response text")?
        .trim()
        .to_string();

    Ok(ip)
}

/// IP 주소 문자열이 유효한지 간단히 검증하는 함수
fn is_valid_ip(ip: &str) -> bool {
    // IPv4 또는 IPv6 주소 형식인지 확인
    ip.parse::<std::net::IpAddr>().is_ok()
}

/// 비동기 버전 - 백그라운드에서 Public IP 업데이트
pub fn update_public_ip_async() {
    std::thread::spawn(|| {
        fetch_public_ip().inspect(|ip| {
            let mut cache = CACHE.lock().unwrap();
            cache.set(ip.clone());
        });
    });
}

/// 캐시를 강제로 무효화하는 함수
pub fn invalidate_cache() {
    let mut cache = CACHE.lock().unwrap();
    cache.ip = None;
    cache.last_updated = None;
}

/// Private IP인지 확인하는 유틸리티 함수
pub fn is_private_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ipv4) => {
            // RFC 1918 private ranges
            let octets = ipv4.octets();
            (octets[0] == 10)  // 10.0.0.0/8
                || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)  // 172.16.0.0/12
                || (octets[0] == 192 && octets[1] == 168)  // 192.168.0.0/16
                || (octets[0] == 127) // 127.0.0.0/8 (loopback)
        }
        std::net::IpAddr::V6(ipv6) => {
            // IPv6 private/local addresses
            ipv6.is_loopback()
                || ipv6.is_unspecified()
                || ipv6.segments()[0] & 0xfe00 == 0xfc00  // Unique local
                || ipv6.segments()[0] & 0xffc0 == 0xfe80 // Link local
        }
    }
}
