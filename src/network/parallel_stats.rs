// 병렬 통계 수집 모듈
// rayon을 사용하여 여러 네트워크 인터페이스의 통계를 동시에 수집

use anyhow::{Result, Context};
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use crate::network::interface::NetworkInterface;
use crate::network::stats::InterfaceStats;

/// 병렬로 모든 인터페이스의 통계를 수집
pub fn collect_all_stats_parallel(interfaces: &[NetworkInterface]) -> Result<Vec<InterfaceStats>> {
    // rayon의 par_iter를 사용하여 병렬 처리
    let stats: Result<Vec<_>> = interfaces
        .par_iter() // 병렬 이터레이터로 변환
        .map(|interface| {
            // 각 인터페이스에 대해 통계 수집 (병렬로 실행됨)
            collect_interface_stats(interface.index)
                .with_context(|| format!("Failed to collect stats for interface {}", interface.name))
        })
        .collect(); // 결과를 수집
    
    stats
}

/// 활성 인터페이스들의 통계만 병렬로 수집
pub fn collect_active_stats_parallel(interfaces: &[NetworkInterface]) -> Result<Vec<(usize, InterfaceStats)>> {
    // 활성 인터페이스의 인덱스와 함께 수집
    let active_indices: Vec<_> = interfaces
        .iter()
        .enumerate()
        .filter(|(_, iface)| iface.is_up && !iface.is_loopback)
        .map(|(idx, iface)| (idx, iface.index))
        .collect();
    
    // 병렬로 통계 수집
    let stats: Result<Vec<_>> = active_indices
        .par_iter()
        .map(|(idx, if_index)| {
            collect_interface_stats(*if_index)
                .map(|stats| (*idx, stats))
                .with_context(|| format!("Failed to collect stats for interface index {}", if_index))
        })
        .collect();
    
    stats
}

/// 배치 통계 수집 - 인터페이스를 그룹으로 나누어 처리
pub fn collect_stats_in_batches(
    interfaces: &[NetworkInterface], 
    batch_size: usize
) -> Result<Vec<InterfaceStats>> {
    let mut all_stats = Vec::with_capacity(interfaces.len());
    
    // 인터페이스를 배치로 나누어 처리
    for batch in interfaces.chunks(batch_size) {
        let batch_stats: Result<Vec<_>> = batch
            .par_iter()
            .map(|interface| collect_interface_stats(interface.index))
            .collect();
        
        all_stats.extend(batch_stats?);
    }
    
    Ok(all_stats)
}

/// 단일 인터페이스의 통계 수집 (플랫폼별 구현 호출)
fn collect_interface_stats(interface_index: u32) -> Result<InterfaceStats> {
    #[cfg(target_os = "windows")]
    {
        super::windows_api::get_interface_statistics(interface_index)
    }
    
    #[cfg(target_os = "linux")]
    {
        super::linux_api::get_interface_statistics(interface_index)
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(anyhow::anyhow!("Unsupported platform"))
    }
}

/// 통계 수집 성능 측정을 위한 래퍼
pub struct StatsCollector {
    interfaces: Arc<Vec<NetworkInterface>>,
    last_collection_time: Option<Instant>,
    use_parallel: bool,
}

impl StatsCollector {
    /// 새로운 StatsCollector 생성
    pub fn new(interfaces: Vec<NetworkInterface>) -> Self {
        Self {
            interfaces: Arc::new(interfaces),
            last_collection_time: None,
            use_parallel: true, // 기본적으로 병렬 처리 사용
        }
    }
    
    /// 병렬 처리 활성화/비활성화
    pub fn set_parallel(&mut self, use_parallel: bool) {
        self.use_parallel = use_parallel;
    }
    
    /// 통계 수집 및 수집 시간 측정
    pub fn collect(&mut self) -> Result<(Vec<InterfaceStats>, f64)> {
        let start = Instant::now();
        
        let stats = if self.use_parallel {
            // 병렬 수집
            collect_all_stats_parallel(&self.interfaces)?
        } else {
            // 순차 수집 (비교용)
            self.collect_sequential()?
        };
        
        let elapsed = start.elapsed().as_secs_f64();
        self.last_collection_time = Some(start);
        
        Ok((stats, elapsed))
    }
    
    /// 순차적으로 통계 수집 (성능 비교용)
    fn collect_sequential(&self) -> Result<Vec<InterfaceStats>> {
        self.interfaces
            .iter()
            .map(|interface| collect_interface_stats(interface.index))
            .collect()
    }
    
    /// 마지막 수집 이후 경과 시간
    pub fn time_since_last_collection(&self) -> Option<f64> {
        self.last_collection_time
            .map(|time| time.elapsed().as_secs_f64())
    }
}

/// 병렬 처리 최적화를 위한 스레드 풀 크기 계산
pub fn optimal_thread_count(interface_count: usize) -> usize {
    use std::cmp::{min, max};
    
    // CPU 코어 수 가져오기
    let cpu_count = rayon::current_num_threads();
    
    // 인터페이스가 적으면 스레드도 적게 사용
    // 인터페이스가 많으면 CPU 코어 수만큼 사용
    min(
        max(2, cpu_count), // 최소 2개, 최대 CPU 코어 수
        max(1, interface_count / 2) // 인터페이스 수의 절반
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimal_thread_count() {
        // 인터페이스가 적을 때
        assert!(optimal_thread_count(2) >= 1);
        assert!(optimal_thread_count(4) >= 1);
        
        // 인터페이스가 많을 때
        let many = optimal_thread_count(20);
        assert!(many >= 2);
        assert!(many <= rayon::current_num_threads());
    }
}