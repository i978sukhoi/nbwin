// 병렬 통계 수집 성능 벤치마크
// 순차 처리와 병렬 처리의 성능 차이를 측정

use anyhow::Result;
use nbmon::network::interface;
use nbmon::network::parallel_stats::{
    collect_active_stats_parallel, collect_all_stats_parallel, StatsCollector,
};
use std::time::Instant;

fn main() -> Result<()> {
    println!("=== NBMon Parallel Stats Collection Benchmark ===\n");

    // 네트워크 인터페이스 목록 가져오기
    let interfaces = interface::list_interfaces()?;

    if interfaces.is_empty() {
        println!("No network interfaces found!");
        return Ok(());
    }

    println!("Found {} network interfaces", interfaces.len());

    // 활성 인터페이스 수 계산
    let active_count = interfaces
        .iter()
        .filter(|iface| iface.is_up && !iface.is_loopback)
        .count();
    println!("Active interfaces: {}\n", active_count);

    // CPU 코어 수 확인
    println!("CPU cores available: {}", rayon::current_num_threads());
    println!("{}", "-".repeat(50));

    // 1. 순차 처리 벤치마크
    println!("\n1. Sequential Collection:");
    let sequential_times = benchmark_sequential(&interfaces, 10)?;
    print_stats(&sequential_times);

    // 2. 병렬 처리 벤치마크 (전체 인터페이스)
    println!("\n2. Parallel Collection (All Interfaces):");
    let parallel_all_times = benchmark_parallel_all(&interfaces, 10)?;
    print_stats(&parallel_all_times);

    // 3. 병렬 처리 벤치마크 (활성 인터페이스만)
    println!("\n3. Parallel Collection (Active Only):");
    let parallel_active_times = benchmark_parallel_active(&interfaces, 10)?;
    print_stats(&parallel_active_times);

    // 4. StatsCollector 사용 벤치마크
    println!("\n4. StatsCollector with Parallel:");
    let collector_times = benchmark_collector(&interfaces, 10)?;
    print_stats(&collector_times);

    // 성능 비교
    println!("\n{}", "=".repeat(50));
    println!("Performance Comparison:");
    println!("{}", "-".repeat(50));

    let seq_avg = average(&sequential_times);
    let par_all_avg = average(&parallel_all_times);
    let par_active_avg = average(&parallel_active_times);
    let collector_avg = average(&collector_times);

    println!("Sequential:        {:.3} ms", seq_avg);
    println!(
        "Parallel (All):    {:.3} ms ({:.1}x speedup)",
        par_all_avg,
        seq_avg / par_all_avg
    );
    println!(
        "Parallel (Active): {:.3} ms ({:.1}x speedup)",
        par_active_avg,
        seq_avg / par_active_avg
    );
    println!(
        "StatsCollector:    {:.3} ms ({:.1}x speedup)",
        collector_avg,
        seq_avg / collector_avg
    );

    // 권장사항
    println!("\n{}", "=".repeat(50));
    println!("Recommendations:");
    if interfaces.len() > 4 && par_all_avg < seq_avg * 0.8 {
        println!("✅ Parallel collection is recommended for your system");
        println!(
            "   Expected performance gain: {:.0}%",
            (1.0 - par_all_avg / seq_avg) * 100.0
        );
    } else if interfaces.len() <= 4 {
        println!("⚠️  Your system has few interfaces");
        println!("   Parallel collection may not provide significant benefits");
    } else {
        println!("🔍 Performance gains are marginal");
        println!("   Consider using parallel collection for consistency");
    }

    Ok(())
}

// 순차 처리 벤치마크
fn benchmark_sequential(
    interfaces: &[nbmon::NetworkInterface],
    iterations: usize,
) -> Result<Vec<f64>> {
    use nbmon::network::stats::get_interface_stats;

    let mut times = Vec::new();

    for _ in 0..iterations {
        let start = Instant::now();

        for interface in interfaces {
            let _ = get_interface_stats(interface.index);
        }

        times.push(start.elapsed().as_secs_f64() * 1000.0); // ms로 변환
    }

    Ok(times)
}

// 병렬 처리 벤치마크 (전체)
fn benchmark_parallel_all(
    interfaces: &[nbmon::NetworkInterface],
    iterations: usize,
) -> Result<Vec<f64>> {
    let mut times = Vec::new();

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = collect_all_stats_parallel(interfaces)?;
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    Ok(times)
}

// 병렬 처리 벤치마크 (활성 인터페이스만)
fn benchmark_parallel_active(
    interfaces: &[nbmon::NetworkInterface],
    iterations: usize,
) -> Result<Vec<f64>> {
    let mut times = Vec::new();

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = collect_active_stats_parallel(interfaces)?;
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    Ok(times)
}

// StatsCollector 벤치마크
fn benchmark_collector(
    interfaces: &[nbmon::NetworkInterface],
    iterations: usize,
) -> Result<Vec<f64>> {
    let mut collector = StatsCollector::new(interfaces.to_vec());
    let mut times = Vec::new();

    for _ in 0..iterations {
        let (_, elapsed) = collector.collect()?;
        times.push(elapsed * 1000.0); // ms로 변환
    }

    Ok(times)
}

// 통계 출력
fn print_stats(times: &[f64]) {
    let avg = average(times);
    let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let std_dev = standard_deviation(times);

    println!("  Average: {:.3} ms", avg);
    println!("  Min:     {:.3} ms", min);
    println!("  Max:     {:.3} ms", max);
    println!("  Std Dev: {:.3} ms", std_dev);
}

// 평균 계산
fn average(times: &[f64]) -> f64 {
    times.iter().sum::<f64>() / times.len() as f64
}

// 표준편차 계산
fn standard_deviation(times: &[f64]) -> f64 {
    let avg = average(times);
    let variance = times.iter().map(|x| (x - avg).powi(2)).sum::<f64>() / times.len() as f64;
    variance.sqrt()
}
