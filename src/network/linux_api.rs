// Linux 네트워크 인터페이스 API 구현
// rtnetlink와 /proc/net/dev를 사용한 네트워크 통계 수집

use anyhow::{Result, Context};
use std::collections::HashMap;
use std::net::IpAddr;
use rtnetlink::{new_connection, Handle};
use tokio::runtime::Runtime;
use crate::network::interface::NetworkInterface;
use crate::network::stats::InterfaceStats;

/// Linux 시스템에서 네트워크 인터페이스 목록을 가져오는 함수
/// rtnetlink를 사용하여 시스템의 모든 네트워크 인터페이스 정보를 수집
pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    // Tokio 런타임 생성 (비동기 작업을 동기적으로 실행하기 위해)
    let rt = Runtime::new().context("Failed to create Tokio runtime")?;
    
    rt.block_on(async {
        // rtnetlink 연결 생성
        let (connection, handle, _) = new_connection().context("Failed to create netlink connection")?;
        
        // 백그라운드에서 연결 실행
        tokio::spawn(connection);
        
        collect_interfaces(handle).await
    })
}

/// 실제 인터페이스 정보를 수집하는 비동기 함수
async fn collect_interfaces(handle: Handle) -> Result<Vec<NetworkInterface>> {
    let mut interfaces = Vec::new();
    
    // 모든 네트워크 링크 정보 가져오기
    let mut links = handle.link().get().execute();
    
    while let Some(link) = links.try_next().await.context("Failed to get network links")? {
        // 링크 인덱스와 이름 추출
        let index = link.header.index;
        let name = link.attributes.name.unwrap_or_else(|| format!("interface_{}", index));
        
        // NetworkInterface 인스턴스 생성
        let mut interface = NetworkInterface::new(index, name.clone(), name.clone());
        
        // 인터페이스 상태 설정
        interface.is_up = (link.header.flags & libc::IFF_UP as u32) != 0;
        interface.is_loopback = (link.header.flags & libc::IFF_LOOPBACK as u32) != 0;
        
        // MAC 주소 설정
        if let Some(address) = link.attributes.address {
            if !address.is_empty() {
                interface.mac_address = NetworkInterface::format_mac_address(&address);
            }
        }
        
        // 인터페이스 속도 설정 (가능한 경우)
        // Linux에서는 ethtool을 통해 얻어야 하므로 기본값 사용
        interface.speed = 1_000_000_000; // 1 Gbps 기본값
        
        // IP 주소 수집
        interface.ip_addresses = get_interface_addresses(&handle, index).await?;
        
        interfaces.push(interface);
    }
    
    Ok(interfaces)
}

/// 특정 인터페이스의 IP 주소들을 가져오는 함수
async fn get_interface_addresses(handle: &Handle, interface_index: u32) -> Result<Vec<IpAddr>> {
    let mut addresses = Vec::new();
    
    // IPv4 주소 수집
    let mut addr_v4 = handle.address().get().set_link_index_filter(interface_index).execute();
    while let Some(addr) = addr_v4.try_next().await.context("Failed to get IPv4 addresses")? {
        if let Some(ip_addr) = addr.attributes.address {
            if ip_addr.len() == 4 {
                let octets = [ip_addr[0], ip_addr[1], ip_addr[2], ip_addr[3]];
                addresses.push(IpAddr::V4(std::net::Ipv4Addr::from(octets)));
            } else if ip_addr.len() == 16 {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&ip_addr);
                addresses.push(IpAddr::V6(std::net::Ipv6Addr::from(bytes)));
            }
        }
    }
    
    Ok(addresses)
}

/// Linux 시스템에서 특정 인터페이스의 네트워크 통계를 가져오는 함수
/// /proc/net/dev 파일을 파싱하거나 rtnetlink를 사용
pub fn get_interface_statistics(interface_index: u32) -> Result<InterfaceStats> {
    // /proc/net/dev 파일에서 통계 읽기 (더 간단하고 안정적)
    let proc_content = std::fs::read_to_string("/proc/net/dev")
        .context("Failed to read /proc/net/dev")?;
    
    parse_proc_net_dev(interface_index, &proc_content)
}

/// /proc/net/dev 내용을 파싱하여 인터페이스 통계를 추출하는 함수
fn parse_proc_net_dev(target_index: u32, content: &str) -> Result<InterfaceStats> {
    // 인터페이스 이름을 인덱스로 매핑하기 위해 먼저 모든 인터페이스 정보 수집
    let interfaces = get_network_interfaces()?;
    let target_name = interfaces
        .iter()
        .find(|iface| iface.index == target_index)
        .map(|iface| &iface.name)
        .context("Interface not found")?;
    
    // /proc/net/dev 파일 파싱
    for line in content.lines().skip(2) { // 헤더 2줄 건너뛰기
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 17 { // 최소 17개 필드 필요
            continue;
        }
        
        // 인터페이스 이름 추출 (콜론 제거)
        let iface_name = parts[0].trim_end_matches(':');
        
        if iface_name == target_name {
            let mut stats = InterfaceStats::new(target_index);
            
            // /proc/net/dev 형식: 인터페이스명 + 16개 통계 필드
            // RX: bytes, packets, errs, drop, fifo, frame, compressed, multicast
            // TX: bytes, packets, errs, drop, fifo, colls, carrier, compressed
            stats.bytes_received = parts[1].parse().unwrap_or(0);
            stats.packets_received = parts[2].parse().unwrap_or(0);
            stats.errors_in = parts[3].parse().unwrap_or(0);
            
            stats.bytes_sent = parts[9].parse().unwrap_or(0);
            stats.packets_sent = parts[10].parse().unwrap_or(0);
            stats.errors_out = parts[11].parse().unwrap_or(0);
            
            return Ok(stats);
        }
    }
    
    Err(anyhow::anyhow!("Interface {} not found in /proc/net/dev", target_name))
}

// rtnetlink의 futures 트레이트 사용을 위한 import
use futures::stream::TryStreamExt;

// libc constants를 사용하기 위한 상수 정의 (rtnetlink가 제공하지 않는 경우)
#[allow(dead_code)]
mod constants {
    pub const IFF_UP: u32 = 0x1;
    pub const IFF_LOOPBACK: u32 = 0x8;
}

// libc 크레이트가 필요한 경우를 대비한 대체 구현
#[cfg(not(target_os = "linux"))]
mod libc {
    pub const IFF_UP: u32 = 0x1;
    pub const IFF_LOOPBACK: u32 = 0x8;
}