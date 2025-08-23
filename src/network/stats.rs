use std::time::Instant;
use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct InterfaceStats {
    pub interface_index: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors_in: u64,
    pub errors_out: u64,
    pub timestamp: Option<Instant>,
}

#[derive(Debug, Clone, Default)]
pub struct BandwidthStats {
    pub download_rate: f64, // bytes per second
    pub upload_rate: f64,   // bytes per second
    pub total_downloaded: u64,
    pub total_uploaded: u64,
    pub peak_download_rate: f64,
    pub peak_upload_rate: f64,
}

impl InterfaceStats {
    pub fn new(interface_index: u32) -> Self {
        Self {
            interface_index,
            timestamp: Some(Instant::now()),
            ..Default::default()
        }
    }

    pub fn calculate_bandwidth(&self, previous: &InterfaceStats) -> Option<BandwidthStats> {
        let current_time = self.timestamp?;
        let previous_time = previous.timestamp?;
        
        let duration = current_time.duration_since(previous_time);
        if duration.as_secs_f64() == 0.0 {
            return None;
        }

        let bytes_received_diff = self.bytes_received.saturating_sub(previous.bytes_received);
        let bytes_sent_diff = self.bytes_sent.saturating_sub(previous.bytes_sent);

        let download_rate = bytes_received_diff as f64 / duration.as_secs_f64();
        let upload_rate = bytes_sent_diff as f64 / duration.as_secs_f64();

        Some(BandwidthStats {
            download_rate,
            upload_rate,
            total_downloaded: self.bytes_received,
            total_uploaded: self.bytes_sent,
            peak_download_rate: download_rate,
            peak_upload_rate: upload_rate,
        })
    }
}

pub fn get_interface_stats(interface_index: u32) -> Result<InterfaceStats> {
    crate::network::windows_api::get_interface_statistics(interface_index)
}