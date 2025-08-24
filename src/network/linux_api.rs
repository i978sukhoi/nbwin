// Linux 네트워크 인터페이스 API 구현
// /proc/net/dev와 /sys/class/net을 사용한 간단한 구현

use anyhow::{Result, Context};
use std::net::IpAddr;
use std::fs;
use std::path::Path;
use crate::network::interface::NetworkInterface;
use crate::network::stats::InterfaceStats;

/// Linux 시스템에서 네트워크 인터페이스 목록을 가져오는 함수
/// /sys/class/net 디렉터리를 읽어서 인터페이스 정보 수집
pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    let mut interfaces = Vec::new();
    let mut index = 1u32;
    
    // /sys/class/net 디렉터리의 모든 인터페이스 읽기
    let net_dir = Path::new("/sys/class/net");
    if !net_dir.exists() {
        return Err(anyhow::anyhow!("/sys/class/net directory not found"))
            .context("This system may not be Linux or the /sys filesystem is not mounted");
    }
    
    for entry in fs::read_dir(net_dir)
        .context("Failed to read /sys/class/net directory")
        .context("Check if you have permission to access network information")? {
        let entry = entry.context("Failed to read directory entry")?;
        let iface_name = entry.file_name().to_string_lossy().to_string();
        
        // 인터페이스 정보 생성
        let mut interface = NetworkInterface::new(index, iface_name.clone(), iface_name.clone());
        
        // 인터페이스 상태 읽기
        let operstate_path = entry.path().join("operstate");
        if let Ok(state) = fs::read_to_string(&operstate_path) {
            interface.is_up = state.trim() == "up";
        }
        
        // 루프백 확인
        let type_path = entry.path().join("type");
        if let Ok(iface_type) = fs::read_to_string(&type_path) {
            // type 772 = loopback
            interface.is_loopback = iface_type.trim() == "772";
        }
        
        // MAC 주소 읽기
        let address_path = entry.path().join("address");
        if let Ok(mac) = fs::read_to_string(&address_path) {
            let mac = mac.trim();
            if mac != "00:00:00:00:00:00" && !mac.is_empty() {
                interface.mac_address = mac.to_uppercase();
            }
        }
        
        // 속도 읽기 (가능한 경우)
        let speed_path = entry.path().join("speed");
        if let Ok(speed_str) = fs::read_to_string(&speed_path) {
            if let Ok(speed_mbps) = speed_str.trim().parse::<u64>() {
                interface.speed = speed_mbps * 1_000_000; // Mbps to bps
            } else {
                interface.speed = 1_000_000_000; // 기본값 1 Gbps
            }
        } else {
            interface.speed = 1_000_000_000; // 기본값 1 Gbps
        }
        
        // IP 주소는 간단히 빈 배열로 (필요시 ip 명령 사용 가능)
        interface.ip_addresses = Vec::new();
        
        interfaces.push(interface);
        index += 1;
    }
    
    Ok(interfaces)
}

/// Linux 시스템에서 특정 인터페이스의 네트워크 통계를 가져오는 함수
/// /proc/net/dev 파일을 파싱하여 통계 수집
pub fn get_interface_statistics(interface_index: u32) -> Result<InterfaceStats> {
    // /proc/net/dev 파일에서 통계 읽기
    let proc_content = fs::read_to_string("/proc/net/dev")
        .context("Failed to read /proc/net/dev")
        .context("Network statistics file not accessible")?
    
    // 인터페이스 목록 가져오기
    let interfaces = get_network_interfaces()?;
    let target_interface = interfaces
        .get((interface_index - 1) as usize)
        .context("Interface index out of range")?;
    
    parse_proc_net_dev(&target_interface.name, &proc_content, interface_index)
}

/// /proc/net/dev 내용을 파싱하여 인터페이스 통계를 추출하는 함수
fn parse_proc_net_dev(target_name: &str, content: &str, interface_index: u32) -> Result<InterfaceStats> {
    // /proc/net/dev 파일 파싱
    for line in content.lines().skip(2) { // 헤더 2줄 건너뛰기
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // 인터페이스 이름과 통계 분리
        if let Some(colon_pos) = line.find(':') {
            let iface_name = line[..colon_pos].trim();
            
            if iface_name == target_name {
                let stats_str = &line[colon_pos + 1..];
                let parts: Vec<&str> = stats_str.split_whitespace().collect();
                
                if parts.len() >= 16 {
                    let mut stats = InterfaceStats::new(interface_index);
                    
                    // /proc/net/dev 형식:
                    // RX: bytes, packets, errs, drop, fifo, frame, compressed, multicast
                    // TX: bytes, packets, errs, drop, fifo, colls, carrier, compressed
                    stats.bytes_received = parts[0].parse().unwrap_or(0);
                    stats.packets_received = parts[1].parse().unwrap_or(0);
                    stats.errors_in = parts[2].parse().unwrap_or(0);
                    
                    stats.bytes_sent = parts[8].parse().unwrap_or(0);
                    stats.packets_sent = parts[9].parse().unwrap_or(0);
                    stats.errors_out = parts[10].parse().unwrap_or(0);
                    
                    return Ok(stats);
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Interface '{}' not found in /proc/net/dev", target_name))
        .context("The interface may have been removed or renamed")
}