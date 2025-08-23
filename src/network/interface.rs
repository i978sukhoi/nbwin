// 네트워크 인터페이스 정보를 다루는 모듈
use std::net::IpAddr;  // 표준 라이브러리의 IP 주소 타입
use anyhow::Result;    // 에러 처리용 Result 타입

// #[derive(...)]: 자동으로 trait 구현 생성
// Debug: println!("{:?}", obj) 형태로 디버그 출력 가능
// Clone: .clone() 메서드로 복사본 생성 가능
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    // pub: 구조체 필드를 외부에서 접근 가능하도록 공개
    pub index: u32,                    // 네트워크 인터페이스 인덱스 번호
    pub name: String,                  // 인터페이스 이름 (예: "eth0")
    pub description: String,           // 사용자 친화적 설명 (예: "Realtek PCIe GbE")
    pub mac_address: String,           // MAC 주소 문자열 (예: "00:11:22:33:44:55")
    pub ip_addresses: Vec<IpAddr>,     // IP 주소 목록 (여러개 가능)
    pub is_up: bool,                   // 인터페이스 활성 상태 (UP/DOWN)
    pub is_loopback: bool,             // 루프백 인터페이스 여부
    pub speed: u64,                    // 인터페이스 속도 (bits per second)
}

// impl 블록: 구조체에 메서드 구현
impl NetworkInterface {
    // 새로운 NetworkInterface 인스턴스를 생성하는 생성자 함수
    // Self: 현재 구조체 타입(NetworkInterface)을 의미
    pub fn new(index: u32, name: String, description: String) -> Self {
        Self {
            // 필드명과 변수명이 같으면 축약 문법 사용 가능 (index: index → index)
            index,
            name,
            description,
            // String::new(): 빈 문자열 생성
            mac_address: String::new(),
            // Vec::new(): 빈 벡터 생성
            ip_addresses: Vec::new(),
            // 기본값들 설정
            is_up: false,
            is_loopback: false,
            speed: 0,
        }
    }

    // MAC 주소 바이트 배열을 문자열로 변환하는 유틸리티 함수
    // &[u8]: u8 바이트들의 슬라이스에 대한 참조
    pub fn format_mac_address(bytes: &[u8]) -> String {
        bytes
            .iter()                          // 바이트 배열의 iterator 생성
            .map(|b| format!("{:02X}", b))   // 각 바이트를 2자리 16진수 대문자로 포맷
            .collect::<Vec<_>>()             // 결과를 Vec<String>으로 수집
            .join(":")                       // ":"로 연결하여 하나의 문자열로 만듦
    }

    // 가상 네트워크 인터페이스인지 판별하는 메서드
    // &self: 구조체 인스턴스에 대한 immutable 참조 (self를 수정하지 않음)
    pub fn is_virtual(&self) -> bool {
        // description을 소문자로 변환하여 대소문자 구분 없이 비교
        let lower_desc = self.description.to_lowercase();
        
        // || 연산자: 논리 OR, 하나라도 true면 전체가 true
        // .contains(): 문자열에 특정 부분 문자열이 포함되어 있는지 확인
        lower_desc.contains("virtual") 
            || lower_desc.contains("vmware")
            || lower_desc.contains("virtualbox")
            || lower_desc.contains("hyper-v")
            || lower_desc.contains("vpn")
            || lower_desc.contains("tap")
            || lower_desc.contains("tun")
    }

    // 사용자에게 표시할 인터페이스 이름을 반환하는 메서드
    pub fn display_name(&self) -> String {
        // description이 있으면 description 사용, 없으면 name 사용
        if !self.description.is_empty() {
            // .clone(): 문자열의 소유권을 가진 복사본 생성
            self.description.clone()
        } else {
            self.name.clone()
        }
    }
}

// 네트워크 인터페이스 목록을 가져오는 공개 함수 (플랫폼별 구현)
pub fn list_interfaces() -> Result<Vec<NetworkInterface>> {
    #[cfg(windows)]
    {
        // Windows API를 통해 네트워크 인터페이스 정보를 가져옴
        crate::network::windows_api::get_network_interfaces()
    }
    
    #[cfg(unix)]
    {
        // Linux API를 통해 네트워크 인터페이스 정보를 가져옴
        crate::network::linux_api::get_network_interfaces()
    }
}