use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::time::{Duration, Instant};

use crate::network::parallel_stats::collect_all_stats_parallel;
use crate::network::{
    interface::NetworkInterface,
    public_ip,
    stats::{BandwidthStats, InterfaceStats},
};
use crate::utils::format;

pub struct App {
    pub interfaces: Vec<NetworkInterface>,
    pub interface_stats: Vec<InterfaceStats>,
    pub bandwidth_stats: Vec<Option<BandwidthStats>>,
    pub selected_interface: usize,
    pub last_update: Instant,
    pub update_interval: Duration,
    pub should_quit: bool,
}

impl App {
    pub fn new(interfaces: Vec<NetworkInterface>) -> Result<Self> {
        let interface_count = interfaces.len();
        let mut interface_stats = Vec::new();

        // Initialize stats for all interfaces
        for interface in &interfaces {
            match crate::network::stats::get_interface_stats(interface.index) {
                Ok(stats) => interface_stats.push(stats),
                Err(_) => interface_stats.push(InterfaceStats::new(interface.index)),
            }
        }

        Ok(Self {
            interfaces,
            interface_stats,
            bandwidth_stats: vec![None; interface_count],
            selected_interface: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1),
            should_quit: false,
        })
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => {
                                self.should_quit = true;
                                break;
                            }
                            KeyCode::Up => {
                                if self.selected_interface > 0 {
                                    self.selected_interface -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.selected_interface < self.interfaces.len().saturating_sub(1)
                                {
                                    self.selected_interface += 1;
                                }
                            }
                            KeyCode::Char(' ') => {
                                // Force update
                                self.update_stats()?;
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Auto-update stats
            if self.last_update.elapsed() >= self.update_interval {
                self.update_stats()?;
            }
        }
        Ok(())
    }

    fn update_stats(&mut self) -> Result<()> {
        let prev_stats = self.interface_stats.clone();

        // 병렬로 모든 인터페이스의 통계 수집
        match collect_all_stats_parallel(&self.interfaces) {
            Ok(new_stats) => {
                // 성공적으로 수집된 통계 업데이트
                for (i, current_stats) in new_stats.into_iter().enumerate() {
                    if i < prev_stats.len() {
                        self.bandwidth_stats[i] = current_stats.calculate_bandwidth(&prev_stats[i]);
                    }
                    if i < self.interface_stats.len() {
                        self.interface_stats[i] = current_stats;
                    }
                }
            }
            Err(e) => {
                // 병렬 수집 실패 시 순차 수집으로 폴백
                eprintln!(
                    "Warning: Parallel stats collection failed: {}, using sequential fallback",
                    e
                );
                for (i, interface) in self.interfaces.iter().enumerate() {
                    if let Ok(current_stats) =
                        crate::network::stats::get_interface_stats(interface.index)
                    {
                        if i < prev_stats.len() {
                            self.bandwidth_stats[i] =
                                current_stats.calculate_bandwidth(&prev_stats[i]);
                        }
                        self.interface_stats[i] = current_stats;
                    }
                }
            }
        }

        self.last_update = Instant::now();
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Interface list
                Constraint::Length(6), // Selected interface details
                Constraint::Length(3), // Help
            ])
            .split(f.size());

        self.render_header(f, chunks[0]);
        self.render_interface_list(f, chunks[1]);
        self.render_interface_details(f, chunks[2]);
        self.render_help(f, chunks[3]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new("NBMon - Cross-platform Network Bandwidth Monitor")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(header, area);
    }

    fn render_interface_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .interfaces
            .iter()
            .enumerate()
            .map(|(i, interface)| {
                let bandwidth_text = if let Some(Some(bandwidth)) = self.bandwidth_stats.get(i) {
                    format!(
                        " ↓{} ↑{}",
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

                let mut spans = vec![
                    Span::styled(
                        format!("[{}] ", if interface.is_up { "UP" } else { "DOWN" }),
                        Style::default().fg(status_color),
                    ),
                    Span::raw(format!("{} ", interface.display_name())),
                ];

                if !bandwidth_text.is_empty() {
                    spans.push(Span::styled(
                        bandwidth_text,
                        Style::default().fg(Color::Yellow),
                    ));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Network Interfaces"),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("> ");

        f.render_stateful_widget(
            list,
            area,
            &mut ratatui::widgets::ListState::default()
                .with_selected(Some(self.selected_interface)),
        );
    }

    fn render_interface_details(&self, f: &mut Frame, area: Rect) {
        if let Some(interface) = self.interfaces.get(self.selected_interface) {
            let mut lines = vec![
                Line::from(vec![
                    Span::raw("Name: "),
                    Span::styled(&interface.name, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("Index: "),
                    Span::raw(interface.index.to_string()),
                ]),
                Line::from(vec![Span::raw("MAC: "), Span::raw(&interface.mac_address)]),
            ];

            if interface.speed > 0 {
                lines.push(Line::from(vec![
                    Span::raw("Speed: "),
                    Span::styled(
                        format::format_bits_per_sec(interface.speed),
                        Style::default().fg(Color::Green),
                    ),
                ]));
            }

            // IP 주소 표시 - Private과 Public 구분
            let ip_text = if let Some(local_ip) = interface.ip_addresses.first() {
                let is_private = public_ip::is_private_ip(local_ip);
                let local_str = local_ip.to_string();

                if is_private {
                    // 사설 IP인 경우 Public IP도 함께 표시
                    if let Some(public_ip) = public_ip::get_public_ip() {
                        format!("{} (Public: {})", local_str, public_ip)
                    } else {
                        format!("{} (Fetching public IP...)", local_str)
                    }
                } else {
                    // 이미 공인 IP인 경우
                    local_str
                }
            } else if cfg!(unix) {
                "None (install 'ip' or 'ifconfig')".to_string()
            } else {
                "None".to_string()
            };

            lines.push(Line::from(vec![
                Span::raw("IP: "),
                Span::styled(ip_text, Style::default().fg(Color::Yellow)),
            ]));

            let text = Text::from(lines);
            let paragraph = Paragraph::new(text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Interface Details"),
            );
            f.render_widget(paragraph, area);
        }
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = "Controls: ↑/↓ Select interface | Space: Update | q: Quit";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, area);
    }
}
