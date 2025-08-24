// Example of how to use nbmon library
// This is a simplified version that shows the architecture

use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct NetworkInterface {
    name: String,
    description: String,
    is_up: bool,
    is_virtual: bool,
}

#[derive(Debug, Clone)]
struct NetworkStats {
    bytes_sent: u64,
    bytes_received: u64,
    timestamp: Instant,
}

impl NetworkStats {
    fn calculate_bandwidth(&self, previous: &NetworkStats) -> (f64, f64) {
        let duration = self.timestamp.duration_since(previous.timestamp).as_secs_f64();
        if duration == 0.0 {
            return (0.0, 0.0);
        }
        
        let download_rate = (self.bytes_received - previous.bytes_received) as f64 / duration;
        let upload_rate = (self.bytes_sent - previous.bytes_sent) as f64 / duration;
        
        (download_rate, upload_rate)
    }
}

fn format_bytes_per_sec(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1024.0 {
        format!("{:.1} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else if bytes_per_sec < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
    }
}

// Simulated data for testing
fn get_mock_interfaces() -> Vec<NetworkInterface> {
    vec![
        NetworkInterface {
            name: "Ethernet".to_string(),
            description: "Realtek PCIe GbE Family Controller".to_string(),
            is_up: true,
            is_virtual: false,
        },
        NetworkInterface {
            name: "Wi-Fi".to_string(),
            description: "Intel(R) Wi-Fi 6 AX200 160MHz".to_string(),
            is_up: true,
            is_virtual: false,
        },
        NetworkInterface {
            name: "VirtualBox Host-Only".to_string(),
            description: "VirtualBox Host-Only Ethernet Adapter".to_string(),
            is_up: false,
            is_virtual: true,
        },
    ]
}

fn get_mock_stats(base_sent: u64, base_received: u64) -> NetworkStats {
    // Simulate some network activity
    let random_sent = (Instant::now().elapsed().as_millis() % 1000) as u64 * 1024;
    let random_received = (Instant::now().elapsed().as_millis() % 2000) as u64 * 1024;
    
    NetworkStats {
        bytes_sent: base_sent + random_sent,
        bytes_received: base_received + random_received,
        timestamp: Instant::now(),
    }
}

fn main() {
    println!("NBMon - Cross-platform Network Bandwidth Monitor");
    println!("{}", "=".repeat(50));
    
    let interfaces = get_mock_interfaces();
    
    println!("\nDetected Network Interfaces:");
    println!("{}", "-".repeat(50));
    
    for (idx, iface) in interfaces.iter().enumerate() {
        println!("\n[{}] {}", idx + 1, iface.description);
        println!("    Name: {}", iface.name);
        println!("    Status: {}", if iface.is_up { "UP ✓" } else { "DOWN" });
        println!("    Type: {}", if iface.is_virtual { "Virtual" } else { "Physical" });
    }
    
    println!("\n{}", "=".repeat(50));
    println!("Monitoring active interfaces (5 seconds)...");
    println!("{}", "-".repeat(50));
    
    let active_interfaces: Vec<_> = interfaces
        .iter()
        .filter(|i| i.is_up && !i.is_virtual)
        .collect();
    
    if active_interfaces.is_empty() {
        println!("No active physical interfaces found!");
        return;
    }
    
    // Initialize stats tracking
    let mut stats_history: HashMap<String, NetworkStats> = HashMap::new();
    let mut base_values: HashMap<String, (u64, u64)> = HashMap::new();
    
    for iface in &active_interfaces {
        let initial_stats = get_mock_stats(0, 0);
        stats_history.insert(iface.name.clone(), initial_stats);
        base_values.insert(iface.name.clone(), (0, 0));
    }
    
    // Monitor for 5 seconds
    for i in 1..=5 {
        thread::sleep(Duration::from_secs(1));
        println!("\n[Update {}]", i);
        
        for iface in &active_interfaces {
            let (base_sent, base_received) = base_values.get(&iface.name)
                .expect("Base values should contain all active interfaces");
            let current_stats = get_mock_stats(
                *base_sent + i * 1_000_000,
                *base_received + i * 2_000_000
            );
            
            if let Some(prev_stats) = stats_history.get(&iface.name) {
                let (download_rate, upload_rate) = current_stats.calculate_bandwidth(prev_stats);
                
                println!("  {}:", iface.description);
                println!("    ↓ Download: {}", format_bytes_per_sec(download_rate));
                println!("    ↑ Upload:   {}", format_bytes_per_sec(upload_rate));
                println!("    Total: {} down / {} up",
                    format_bytes_per_sec(current_stats.bytes_received as f64),
                    format_bytes_per_sec(current_stats.bytes_sent as f64)
                );
            }
            
            stats_history.insert(iface.name.clone(), current_stats);
        }
    }
    
    println!("\n{}", "=".repeat(50));
    println!("Monitoring complete!");
}