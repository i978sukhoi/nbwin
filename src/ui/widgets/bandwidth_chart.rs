use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::network::stats::BandwidthStats;
use crate::utils::format;

pub struct BandwidthChart {
    download_history: Vec<(f64, f64)>, // (time, bytes_per_sec)
    upload_history: Vec<(f64, f64)>,
    time_counter: f64,
    max_history: usize,
}

impl Default for BandwidthChart {
    fn default() -> Self {
        Self::new(60) // Keep 60 seconds of history
    }
}

impl BandwidthChart {
    pub fn new(max_history: usize) -> Self {
        Self {
            download_history: Vec::new(),
            upload_history: Vec::new(),
            time_counter: 0.0,
            max_history,
        }
    }

    pub fn update(&mut self, bandwidth: &BandwidthStats) {
        self.time_counter += 1.0;

        // Add new data points
        self.download_history
            .push((self.time_counter, bandwidth.download_rate));
        self.upload_history
            .push((self.time_counter, bandwidth.upload_rate));

        // Trim history if too long
        if self.download_history.len() > self.max_history {
            self.download_history.remove(0);
            self.upload_history.remove(0);
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if self.download_history.is_empty() {
            let no_data = Paragraph::new("No data available")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Bandwidth Chart"),
                )
                .style(Style::default().fg(Color::Gray));
            f.render_widget(no_data, area);
            return;
        }

        // Calculate bounds
        let (min_time, max_time) = if self.download_history.len() >= 2 {
            // Safe to access first and last since we checked length >= 2
            let min_t = self
                .download_history
                .first()
                .map(|(t, _)| *t)
                .unwrap_or(0.0);
            let max_t = self
                .download_history
                .last()
                .map(|(t, _)| *t)
                .unwrap_or(60.0);
            (min_t, max_t)
        } else {
            (0.0, 60.0)
        };

        let max_rate = self
            .download_history
            .iter()
            .chain(self.upload_history.iter())
            .map(|(_, rate)| *rate)
            .filter(|rate| rate.is_finite()) // Filter out NaN/Infinity
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(1024.0) // Default 1KB/s minimum
            .max(1024.0); // At least 1KB/s scale

        let datasets = vec![
            Dataset::default()
                .name("Download")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&self.download_history),
            Dataset::default()
                .name("Upload")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Red))
                .data(&self.upload_history),
        ];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Bandwidth History"),
            )
            .x_axis(
                Axis::default()
                    .title("Time (s)")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([min_time, max_time])
                    .labels(vec![
                        Span::raw(format!("{:.0}", min_time)),
                        Span::raw(format!("{:.0}", max_time)),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title("Speed")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, max_rate])
                    .labels(vec![
                        Span::raw("0"),
                        Span::raw(format::format_bytes_per_sec(max_rate / 2.0)),
                        Span::raw(format::format_bytes_per_sec(max_rate)),
                    ]),
            );

        f.render_widget(chart, area);
    }

    pub fn clear(&mut self) {
        self.download_history.clear();
        self.upload_history.clear();
        self.time_counter = 0.0;
    }
}
