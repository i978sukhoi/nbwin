// 향상된 네트워크 모니터링 TUI 애플리케이션
// Linux의 nload와 유사한 실시간 네트워크 대역폭 모니터링 도구

use anyhow::{Result, Context};               // 간편한 에러 처리 및 컨텍스트 추가
// thiserror 의존성이 없으므로 제거
use std::collections::VecDeque;              // 양방향 큐 (히스토리 데이터 저장용)
use std::time::{Duration, Instant};          // 시간 측정 및 간격 제어
use crossterm::event::{self, Event, KeyCode, KeyEventKind};  // 키보드 입력 처리
use ratatui::{
    backend::Backend,                        // 터미널 백엔드 trait
    layout::{Constraint, Direction, Layout, Rect, Alignment},  // 화면 레이아웃
    style::{Color, Style, Modifier},         // 색상과 스타일
    text::{Line, Span},                      // 텍스트 요소들
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},  // UI 위젯들
    Frame, Terminal,                         // 화면 그리기 관련
};

use crate::network::{interface::NetworkInterface, stats::InterfaceStats};
use crate::utils::format;                    // 데이터 포맷팅 유틸리티

// 애플리케이션 설정 상수들 (매직 넘버 제거)
const HISTORY_SIZE: usize = 60;                    // 히스토리 데이터를 60초간 보관
const UPDATE_INTERVAL_SECS: u64 = 1;               // 통계 업데이트 간격 (초)
const POLL_INTERVAL_MS: u64 = 100;                 // 키 입력 폴링 간격 (밀리초)
const DEFAULT_MAX_RATE_MBPS: f64 = 1.0;            // 기본 최대 속도 (MB/s)
const HEADER_HEIGHT: u16 = 4;                      // 헤더 영역 높이
const HELP_HEIGHT: u16 = 3;                        // 도움말 영역 높이
const MIN_SECTION_HEIGHT: u16 = 8;                 // 트래픽 섹션 최소 높이
const RATE_DISPLAY_HEIGHT: u16 = 1;                // 속도 표시 영역 높이
const MIN_GRAPH_HEIGHT: u16 = 4;                   // 그래프 최소 높이
const LEGEND_WIDTH: u16 = 10;                      // 범례 영역 너비
const MIN_GRAPH_WIDTH: u16 = 10;                   // 그래프 최소 너비
const RATE_SCALE_MULTIPLIER: f64 = 1.1;            // 최대 속도 스케일링 배수

// 에러 처리 개선을 위한 헬퍼 함수들은 메인 impl 블록에 통합됩니다

// 트래픽 섹션 렌더링을 위한 설정 구조체 (DRY 원칙 적용)
#[derive(Debug)]
struct TrafficSectionConfig {
    title: &'static str,           // 섹션 제목 ("Download" 또는 "Upload")
    label_prefix: &'static str,    // 게이지 라벨 접두사 ("Incoming:" 또는 "Outgoing:")
    color: Color,                  // 섹션 색상 (Green 또는 Red)
}

// 향상된 TUI 애플리케이션의 메인 구조체 (캡슐화 적용)
// 네트워크 인터페이스 정보와 실시간 통계를 관리
pub struct ImprovedApp {
    // private 필드들로 변경하여 내부 상태를 보호
    interfaces: Vec<NetworkInterface>,         // 모든 네트워크 인터페이스 목록
    active_interfaces: Vec<usize>,             // 활성화된(UP, 비-루프백) 인터페이스의 인덱스들
    current_interface_idx: usize,              // active_interfaces에서의 현재 선택된 인덱스
    interface_stats: Vec<InterfaceStats>,      // 각 인터페이스의 통계 정보
    download_history: VecDeque<u64>,           // 다운로드 속도 히스토리 (바이트/초)
    upload_history: VecDeque<u64>,             // 업로드 속도 히스토리 (바이트/초)
    last_update: Instant,                      // 마지막 업데이트 시간
    update_interval: Duration,                 // 업데이트 간격 (1초)
    should_quit: bool,                         // 애플리케이션 종료 플래그
    max_download_rate: f64,                    // 그래프 스케일링용 최대 다운로드 속도
    max_upload_rate: f64,                      // 그래프 스케일링용 최대 업로드 속도
}

// ImprovedApp 구조체의 메서드 구현
impl ImprovedApp {
    // 새로운 애플리케이션 인스턴스를 생성하는 생성자
    pub fn new(interfaces: Vec<NetworkInterface>) -> Result<Self> {
        // 활성화되고 루프백이 아닌 인터페이스들만 필터링
        // iterator 체인: iter() → enumerate() → filter() → map() → collect()
        let active_interfaces: Vec<usize> = interfaces
            .iter()                                                    // immutable 참조 iterator
            .enumerate()                                               // (index, item) 형태로 변환
            .filter(|(_, iface)| iface.is_up && !iface.is_loopback)  // UP 상태이고 루프백이 아닌 것만
            .map(|(idx, _)| idx)                                      // 인덱스만 추출
            .collect();                                                // Vec<usize>로 수집

        // 활성 인터페이스가 없으면 에러 반환
        if active_interfaces.is_empty() {
            // anyhow::anyhow!: 에러 메시지로 에러 생성
            return Err(anyhow::anyhow!("No active network interfaces found"));
        }

        let mut interface_stats = Vec::new();
        for interface in &interfaces {
            match crate::network::stats::get_interface_stats(interface.index) {
                Ok(stats) => interface_stats.push(stats),
                Err(_) => interface_stats.push(InterfaceStats::new(interface.index)),
            }
        }

        Ok(Self {
            interfaces,
            active_interfaces,
            current_interface_idx: 0,
            interface_stats,
            download_history: VecDeque::with_capacity(HISTORY_SIZE),
            upload_history: VecDeque::with_capacity(HISTORY_SIZE),
            last_update: Instant::now(),
            update_interval: Duration::from_secs(UPDATE_INTERVAL_SECS),
            should_quit: false,
            max_download_rate: DEFAULT_MAX_RATE_MBPS * 1024.0 * 1024.0, // MB/s를 bytes/s로 변환
            max_upload_rate: DEFAULT_MAX_RATE_MBPS * 1024.0 * 1024.0,
        })
    }

    // 메인 애플리케이션 루프 - TUI를 실행하고 사용자 입력 처리
    // <B: Backend>: 제네릭 타입 매개변수, Backend trait를 구현한 타입
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // 첫 번째 통계 읽기로 초기화
        self.update_stats()?;

        // 메인 이벤트 루프 - 사용자가 종료할 때까지 계속 실행
        loop {
            // 화면 그리기: |f|는 클로저(익명 함수)
            terminal.draw(|f| self.ui(f))?;

            // event::poll(): 설정된 간격동안 키 입력 대기
            if event::poll(Duration::from_millis(POLL_INTERVAL_MS))? {
                // 키 이벤트가 있으면 읽기
                if let Event::Key(key) = event::read()? {
                    // 키를 누를 때만 처리 (키를 뗄 때는 무시)
                    if key.kind == KeyEventKind::Press {
                        // 키 이벤트 처리를 별도 메서드로 분리
                        if self.handle_key_event(key.code)? {
                            break;  // 종료 요청시 루프 탈출
                        }
                    }
                }
            }

            // 자동 통계 업데이트 - 마지막 업데이트로부터 설정된 간격이 지나면 업데이트
            if self.last_update.elapsed() >= self.update_interval {
                self.update_stats()?;
            }
        }
        Ok(())  // 정상 종료
    }

    // === Getter 메서드들 (캡슐화) ===
    
    // 현재 선택된 인터페이스 정보 반환
    pub fn current_interface(&self) -> &NetworkInterface {
        let interface_idx = self.active_interfaces[self.current_interface_idx];
        &self.interfaces[interface_idx]
    }
    
    // 현재 인터페이스의 통계 정보 반환
    pub fn current_interface_stats(&self) -> &InterfaceStats {
        let interface_idx = self.active_interfaces[self.current_interface_idx];
        &self.interface_stats[interface_idx]
    }
    
    // 다운로드 히스토리에 대한 읽기 전용 접근
    pub fn download_history(&self) -> &VecDeque<u64> {
        &self.download_history
    }
    
    // 업로드 히스토리에 대한 읽기 전용 접근
    pub fn upload_history(&self) -> &VecDeque<u64> {
        &self.upload_history
    }
    
    // 최대 다운로드 속도 반환
    pub fn max_download_rate(&self) -> f64 {
        self.max_download_rate
    }
    
    // 최대 업로드 속도 반환  
    pub fn max_upload_rate(&self) -> f64 {
        self.max_upload_rate
    }
    
    // 애플리케이션 종료 상태 확인
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    
    // 현재 인터페이스 인덱스 반환 (UI 표시용)
    pub fn current_interface_display_info(&self) -> (usize, usize) {
        (self.current_interface_idx + 1, self.active_interfaces.len())
    }

    // === 에러 처리 개선을 위한 헬퍼 함수들 ===
    
    // 안전한 현재 인터페이스 인덱스 접근
    fn get_current_interface_index(&self) -> Result<usize> {
        if self.current_interface_idx >= self.active_interfaces.len() {
            return Err(anyhow::anyhow!(
                "Current interface index {} is out of bounds (max: {})",
                self.current_interface_idx,
                self.active_interfaces.len().saturating_sub(1)
            ));
        }
        Ok(self.active_interfaces[self.current_interface_idx])
    }
    
    // 컨텍스트가 포함된 통계 정보 가져오기
    fn get_interface_stats_with_context(&self, interface_index: u32) -> Result<crate::network::stats::InterfaceStats> {
        crate::network::stats::get_interface_stats(interface_index)
            .with_context(|| format!("Failed to get network statistics for interface {}", interface_index))
    }

    // 키 이벤트 처리를 분리한 메서드 (대형 메서드 분할)
    // 반환값: true면 애플리케이션 종료, false면 계속 실행
    fn handle_key_event(&mut self, key_code: KeyCode) -> Result<bool> {
        match key_code {
            KeyCode::Char('q') => {  // 'q' 키: 종료
                self.should_quit = true;
                Ok(true)  // 종료 요청
            }
            KeyCode::Left | KeyCode::Char('h') => {  // 왼쪽 화살표 또는 'h': 이전 인터페이스
                self.switch_to_previous_interface();
                Ok(false)
            }
            KeyCode::Right | KeyCode::Char('l') => {  // 오른쪽 화살표 또는 'l': 다음 인터페이스
                self.switch_to_next_interface();
                Ok(false)
            }
            KeyCode::Char(' ') => {  // 스페이스바: 수동 업데이트
                self.update_stats()?;
                Ok(false)
            }
            KeyCode::Char('r') => {  // 'r' 키: 히스토리 리셋
                self.clear_history();
                Ok(false)
            }
            _ => Ok(false)  // 다른 키는 무시
        }
    }

    // 이전 인터페이스로 전환 (메서드 분할)
    fn switch_to_previous_interface(&mut self) {
        if self.current_interface_idx > 0 {
            self.current_interface_idx -= 1;
            self.clear_history();  // 히스토리 초기화
        }
    }

    // 다음 인터페이스로 전환 (메서드 분할)
    fn switch_to_next_interface(&mut self) {
        // saturating_sub(): 언더플로우 방지 (0보다 작아지면 0)
        if self.current_interface_idx < self.active_interfaces.len().saturating_sub(1) {
            self.current_interface_idx += 1;
            self.clear_history();
        }
    }

    fn update_stats(&mut self) -> Result<()> {
        // 개선된 에러 처리: 안전한 인덱스 접근
        let interface_idx = self.get_current_interface_index()?;
        let interface = &self.interfaces[interface_idx];
        
        // 개선된 에러 처리: 컨텍스트가 포함된 통계 정보 가져오기
        match self.get_interface_stats_with_context(interface.index) {
            Ok(current_stats) => {
                if let Some(bandwidth) = current_stats.calculate_bandwidth(&self.interface_stats[interface_idx]) {
                    // Add to history
                    self.download_history.push_back(bandwidth.download_rate as u64);
                    self.upload_history.push_back(bandwidth.upload_rate as u64);

                    // Trim history
                    if self.download_history.len() > HISTORY_SIZE {
                        self.download_history.pop_front();
                        self.upload_history.pop_front();
                    }

                    // Update max rates for scaling (스케일링 배수 상수 사용)
                    self.max_download_rate = self.max_download_rate.max(bandwidth.download_rate * RATE_SCALE_MULTIPLIER);
                    self.max_upload_rate = self.max_upload_rate.max(bandwidth.upload_rate * RATE_SCALE_MULTIPLIER);
                }
                
                self.interface_stats[interface_idx] = current_stats;
            }
            Err(e) => {
                // 에러가 발생해도 애플리케이션을 중단하지 않고 로그만 남김
                eprintln!("Warning: {}", e);
            }
        }
        
        self.last_update = Instant::now();
        Ok(())
    }

    fn clear_history(&mut self) {
        self.download_history.clear();
        self.upload_history.clear();
        // 기본값으로 리셋 (상수 사용)
        self.max_download_rate = DEFAULT_MAX_RATE_MBPS * 1024.0 * 1024.0;
        self.max_upload_rate = DEFAULT_MAX_RATE_MBPS * 1024.0 * 1024.0;
    }

    // 메인 UI 렌더링 함수 - 화면을 4개 섹션으로 나누어 구성
    fn ui(&mut self, f: &mut Frame) {
        // Layout::default(): 기본 레이아웃 생성
        // Direction::Vertical: 수직으로 분할
        // constraints: 각 영역의 크기 제약 조건
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(HEADER_HEIGHT),      // 고정 높이: 헤더 (인터페이스 정보)
                Constraint::Min(MIN_SECTION_HEIGHT),     // 최소 높이: 다운로드 섹션 (가변)  
                Constraint::Min(MIN_SECTION_HEIGHT),     // 최소 높이: 업로드 섹션 (가변)
                Constraint::Length(HELP_HEIGHT),         // 고정 높이: 도움말
            ])
            .split(f.size());  // 전체 터미널 크기를 위 조건으로 분할

        // 각 섹션을 순서대로 렌더링
        self.render_combined_header(f, chunks[0]);    // 헤더 영역
        // Download와 Upload 섹션을 통합된 메서드로 렌더링
        self.render_traffic_section(
            f, 
            chunks[1], 
            &TrafficSectionConfig {
                title: "Download",
                label_prefix: "Incoming:",
                color: Color::Green,
            },
            self.download_history(),
            self.max_download_rate()
        );
        
        self.render_traffic_section(
            f,
            chunks[2],
            &TrafficSectionConfig {
                title: "Upload", 
                label_prefix: "Outgoing:",
                color: Color::Red,
            },
            self.upload_history(),
            self.max_upload_rate()
        );
        self.render_help(f, chunks[3]);               // 도움말 영역
    }

    fn render_combined_header(&self, f: &mut Frame, area: Rect) {
        // 캡슐화된 getter 메서드 사용
        let interface = self.current_interface();
        let stats = self.current_interface_stats();
        
        // Create single unified header block with program title
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title("nbmon: Cross-platform Network Bandwidth Monitor")
            .style(Style::default().fg(Color::Cyan));
        let inner_area = main_block.inner(area);
        f.render_widget(main_block, area);
        
        // Split inner area into two lines
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title line
                Constraint::Length(1), // Details line
            ])
            .split(inner_area);

        // Interface name with navigation and speed (getter 메서드 사용)
        let (current_idx, total_count) = self.current_interface_display_info();
        let mut interface_line = format!("{} ({}/{})", 
            interface.display_name(),
            current_idx,
            total_count
        );
        
        if interface.speed > 0 {
            interface_line.push_str(&format!(" - {}", format::format_bits_per_sec(interface.speed)));
        }
        
        let interface_paragraph = Paragraph::new(interface_line)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(interface_paragraph, chunks[0]);

        // Single line with all essential info
        let details_line = vec![
            Span::styled("MAC: ", Style::default().fg(Color::Yellow)),
            Span::raw(&interface.mac_address),
            Span::raw("  "),
            Span::styled("IP: ", Style::default().fg(Color::Yellow)),
            Span::raw(interface.ip_addresses.get(0).map(|ip| ip.to_string()).unwrap_or("None".to_string())),
            Span::raw("  "),
            Span::styled("Total: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} ↓ {} ↑", 
                format::format_bytes(stats.bytes_received),
                format::format_bytes(stats.bytes_sent)
            )),
        ];

        let details_paragraph = Paragraph::new(Line::from(details_line))
            .alignment(Alignment::Center);
        f.render_widget(details_paragraph, chunks[1]);
    }

    // 통합된 트래픽 섹션 렌더링 메서드 (DRY 원칙 적용)
    // Download와 Upload 섹션의 중복 코드를 제거하고 하나의 메서드로 통합
    fn render_traffic_section(
        &self,
        f: &mut Frame,
        area: Rect,
        config: &TrafficSectionConfig,
        history: &VecDeque<u64>,
        max_rate: f64,
    ) {
        let current_rate = history.back().copied().unwrap_or(0) as f64;
        
        // Create unified box with internal divisions
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(config.title);  // 설정에서 제목 가져오기
        let inner_area = main_block.inner(area);
        f.render_widget(main_block, area);
        
        // Split the inner area into rate display and sparkline
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(RATE_DISPLAY_HEIGHT), // 속도 표시 영역
                Constraint::Min(MIN_GRAPH_HEIGHT),       // 그래프 영역 (가변)
            ])
            .split(inner_area);

        // Current rate gauge (설정에서 라벨과 색상 가져오기)
        let rate_text = format!("{} {}/s", config.label_prefix, format::format_bytes(current_rate as u64));
        let rate_gauge = Gauge::default()
            .gauge_style(Style::default().fg(config.color))
            .ratio((current_rate / max_rate).min(1.0))
            .label(rate_text);
        f.render_widget(rate_gauge, chunks[0]);

        // Add horizontal separator line
        let separator_area = Rect::new(inner_area.x, chunks[0].bottom(), inner_area.width, 1);
        let separator = Block::default().borders(Borders::TOP);
        f.render_widget(separator, separator_area);

        // Traffic history sparkline with left legend (in remaining area)
        let graph_area = Rect::new(
            inner_area.x, 
            chunks[0].bottom() + 1, 
            inner_area.width, 
            inner_area.height.saturating_sub(chunks[0].height + 1)
        );
        
        // 히스토리가 있으면 스파크라인과 범례 렌더링 (메서드 분할)
        if !history.is_empty() {
            self.render_sparkline_with_legend(f, graph_area, config, history, max_rate);
        }
    }

    // 스파크라인과 범례를 렌더링하는 분할된 메서드
    fn render_sparkline_with_legend(
        &self,
        f: &mut Frame,
        graph_area: Rect,
        config: &TrafficSectionConfig,
        history: &VecDeque<u64>,
        max_rate: f64,
    ) {
        // Split sparkline area into left legend and graph
        let sparkline_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(LEGEND_WIDTH),    // 범례 영역 너비
                Constraint::Min(MIN_GRAPH_WIDTH),    // 그래프 영역 (가변)
            ])
            .split(graph_area);

        // 범례 렌더링 (메서드 분할)
        self.render_legend(f, sparkline_chunks[0], config.color, history, max_rate);
        
        // 스파크라인 그래프 렌더링 (메서드 분할)
        self.render_sparkline_graph(f, sparkline_chunks[1], config.color, history, max_rate);
    }

    // 범례 렌더링 메서드 (Max/Min 표시)
    fn render_legend(
        &self,
        f: &mut Frame,
        legend_area: Rect,
        color: Color,
        history: &VecDeque<u64>,
        max_rate: f64,
    ) {
        // Left legend split vertically for Max/Min
        let legend_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Top - Max
                Constraint::Percentage(50), // Bottom - Min
            ])
            .split(legend_area);

        // Max legend (top)
        let max_legend = Paragraph::new(format!("Max\n{}/s", format::format_bytes(max_rate as u64)))
            .style(Style::default().fg(color))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::RIGHT));
        f.render_widget(max_legend, legend_chunks[0]);

        // Min legend (bottom)
        let min_rate = *history.iter().min().unwrap_or(&0) as f64;
        let min_legend = Paragraph::new(format!("Min\n{}/s", format::format_bytes(min_rate as u64)))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(min_legend, legend_chunks[1]);
    }

    // 스파크라인 그래프 렌더링 메서드
    fn render_sparkline_graph(
        &self,
        f: &mut Frame,
        graph_area: Rect,
        color: Color,
        history: &VecDeque<u64>,
        max_rate: f64,
    ) {
        let data: Vec<u64> = history.iter().copied().collect();
        let sparkline = Sparkline::default()
            .data(&data)
            .max(max_rate as u64)
            .style(Style::default().fg(color));
        f.render_widget(sparkline, graph_area);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = "←/→ or h/l: Switch interface | Space: Update | r: Reset history | q: Quit";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, area);
    }
}