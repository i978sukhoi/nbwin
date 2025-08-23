use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::network::{interface::NetworkInterface, stats::BandwidthStats};
use crate::utils::format;

pub struct InterfaceListWidget<'a> {
    pub interfaces: &'a [NetworkInterface],
    pub bandwidth_stats: &'a [Option<BandwidthStats>],
    pub selected: usize,
}

impl<'a> InterfaceListWidget<'a> {
    pub fn new(
        interfaces: &'a [NetworkInterface], 
        bandwidth_stats: &'a [Option<BandwidthStats>], 
        selected: usize
    ) -> Self {
        Self {
            interfaces,
            bandwidth_stats,
            selected,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .interfaces
            .iter()
            .enumerate()
            .map(|(i, interface)| {
                let bandwidth_text = if let Some(Some(bandwidth)) = self.bandwidth_stats.get(i) {
                    format!(" ↓{} ↑{}", 
                        format::format_bytes_per_sec(bandwidth.download_rate),
                        format::format_bytes_per_sec(bandwidth.upload_rate)
                    )
                } else {
                    String::new()
                };

                let status_color = if interface.is_up {
                    Color::Green
                } else {
                    Color::Red
                };

                let type_indicator = if interface.is_loopback {
                    "L"
                } else if interface.is_virtual() {
                    "V"
                } else {
                    "P"
                };

                let mut spans = vec![
                    Span::styled(
                        format!("[{}]", type_indicator),
                        Style::default().fg(Color::Blue)
                    ),
                    Span::styled(
                        format!(" {} ", if interface.is_up { "●" } else { "○" }),
                        Style::default().fg(status_color)
                    ),
                    Span::raw(format!("{} ", interface.display_name())),
                ];

                if !bandwidth_text.is_empty() {
                    spans.push(Span::styled(bandwidth_text, Style::default().fg(Color::Yellow)));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Network Interfaces (P=Physical, V=Virtual, L=Loopback)"))
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .highlight_symbol("► ");

        let mut state = ListState::default();
        state.select(Some(self.selected));
        f.render_stateful_widget(list, area, &mut state);
    }
}