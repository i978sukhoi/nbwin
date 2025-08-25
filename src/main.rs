// Rust에서 외부 라이브러리(crate)를 import하는 방법
// anyhow: 에러 처리를 간단하게 해주는 라이브러리
use anyhow::{Context, Result};
// crossterm: 크로스플랫폼 터미널 조작 라이브러리
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, // 터미널 명령어를 실행하는 매크로
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
// ratatui: 터미널 UI 라이브러리
use ratatui::{backend::CrosstermBackend, Terminal};
// std::io: Rust 표준 라이브러리의 입출력 모듈
use std::io;

// 우리가 만든 라이브러리에서 필요한 구조체들을 import
use nbmon::{network::{interface, public_ip}, App, ImprovedApp};

// fn main() -> Result<()>: 메인 함수
// Result<()>는 성공시 (), 실패시 에러를 반환하는 타입
fn main() -> Result<()> {
    // 커맨드라인 인자들을 Vec<String>으로 수집
    // std::env::args(): 프로그램 실행시 전달된 인자들을 iterator로 반환
    // .collect(): iterator를 Vec로 변환
    let args: Vec<String> = std::env::args().collect();

    // 인자가 하나 이상 있으면 처리
    if args.len() > 1 {
        // match 표현식: 패턴 매칭으로 값에 따라 다른 동작 수행
        match args[1].as_str() {
            "--simple" => return run_simple_mode(),  // 간단한 콘솔 모드
            "--classic" => return run_classic_tui(), // 클래식 TUI 모드
            "--help" | "-h" => {
                show_help();
                return Ok(());
            }
            "--version" | "-v" => {
                show_version();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Use --help for usage information.");
                return Ok(());
            }
        }
    }

    // 네트워크 인터페이스 목록 가져오기
    // ? 연산자: Result가 Err이면 함수에서 바로 에러를 반환
    let interfaces =
        interface::list_interfaces().context("Failed to get network interfaces list")?;

    // Vec이 비어있는지 확인
    if interfaces.is_empty() {
        // eprintln!: stderr로 출력하는 매크로
        eprintln!("No network interfaces found!");
        // Ok(()): 성공적인 결과 반환
        return Ok(());
    }

    // 터미널을 TUI 모드로 설정
    // raw mode: 터미널이 입력을 즉시 프로그램에 전달 (Enter 없이도)
    enable_raw_mode().context("Failed to enable terminal raw mode")?;

    // stdout 핸들 가져오기
    // mut: 변경 가능한(mutable) 변수로 선언
    let mut stdout = io::stdout();

    // execute! 매크로: 여러 터미널 명령어를 한번에 실행
    // EnterAlternateScreen: 별도 화면 버퍼 사용 (프로그램 종료시 원래 화면으로 복구)
    // EnableMouseCapture: 마우스 이벤트 캡처 활성화
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // 터미널 백엔드와 Terminal 객체 생성
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ImprovedApp 인스턴스 생성하고 실행
    let mut app = ImprovedApp::new(interfaces).context("Failed to initialize the application")?;
    // app.run()의 결과를 res 변수에 저장
    let res = app.run(&mut terminal);

    // 터미널 상태 복원 (정리 작업)
    disable_raw_mode().context("Failed to restore terminal state")?;
    execute!(
        terminal.backend_mut(), // backend에 대한 mutable 참조 가져오기
        LeaveAlternateScreen,   // 원래 화면으로 복귀
        DisableMouseCapture     // 마우스 캡처 비활성화
    )?;
    terminal.show_cursor()?; // 커서 다시 보이게 하기

    // res가 에러인 경우 출력
    // if let: 패턴 매칭으로 특정 케이스만 처리
    if let Err(err) = res {
        // 에러 체인 전체를 출력하여 근본 원인 표시
        eprintln!("Application error: {err:?}");
        eprintln!("\nError chain:");
        for cause in err.chain() {
            eprintln!("  - {}", cause);
        }
    }

    Ok(()) // 성공적으로 완료
}

// 간단한 콘솔 출력 모드 - TUI 없이 텍스트만 출력
fn run_simple_mode() -> Result<()> {
    // 이 함수에서만 사용할 모듈들을 지역적으로 import
    // use 문을 함수 내부에 쓰면 해당 함수에서만 사용 가능
    use nbmon::network::stats;
    use nbmon::utils::format;
    use std::{thread, time::Duration}; // 스레드와 시간 관련 기능

    // println!: 콘솔에 텍스트 출력하는 매크로
    println!("NBMon - Cross-platform Network Bandwidth Monitor (Simple Mode)");
    // "=".repeat(50): 문자열을 50번 반복
    println!("{}", "=".repeat(50));

    // 네트워크 인터페이스 목록 가져오기
    let interfaces = interface::list_interfaces()?;

    // 인터페이스가 없으면 종료
    if interfaces.is_empty() {
        println!("No network interfaces found!");
        return Ok(());
    }

    println!("\nDetected Network Interfaces:");
    println!("{}", "-".repeat(50));

    // 모든 네트워크 인터페이스 정보 출력
    // .iter(): 벡터의 각 요소에 대한 immutable 참조를 반환하는 iterator 생성
    // .enumerate(): iterator에 인덱스 번호를 추가 (idx, item) 형태로 반환
    for (idx, iface) in interfaces.iter().enumerate() {
        // idx는 0부터 시작하므로 +1해서 사용자에게 1부터 보여줌
        println!("\n[{}] {}", idx + 1, iface.display_name());
        println!("    Index: {}", iface.index);
        println!("    Name: {}", iface.name);
        println!("    MAC: {}", iface.mac_address);

        // if 표현식을 이용한 조건부 문자열 선택
        println!("    Status: {}", if iface.is_up { "UP" } else { "DOWN" });

        // 중첩된 if 표현식으로 인터페이스 타입 판별
        println!(
            "    Type: {}",
            if iface.is_loopback {
                "Loopback"
            } else if iface.is_virtual() {
                "Virtual"
            } else {
                "Physical"
            }
        );

        // 속도 정보가 있는 경우에만 출력
        if iface.speed > 0 {
            println!("    Speed: {}", format::format_bits_per_sec(iface.speed));
        }

        // IP 주소 목록이 비어있지 않은 경우 출력
        if !iface.ip_addresses.is_empty() {
            println!("    IP Addresses:");
            // &iface.ip_addresses: 벡터에 대한 참조 (소유권을 이동시키지 않음)
            for ip in &iface.ip_addresses {
                let is_private = public_ip::is_private_ip(ip);
                if is_private {
                    println!("        - {} (Private)", ip);
                } else {
                    println!("        - {} (Public)", ip);
                }
            }
            
            // 사설 IP가 있는 경우 Public IP도 표시
            if iface.ip_addresses.iter().any(|ip| public_ip::is_private_ip(ip)) {
                print!("    Public IP: ");
                if let Some(public_ip_addr) = public_ip::get_public_ip() {
                    println!("{}", public_ip_addr);
                } else {
                    println!("Unable to fetch (check internet connection)");
                }
            }
        }
    }

    // Test statistics collection for active interfaces
    println!("\n{}", "=".repeat(50));
    println!("Testing bandwidth monitoring (5 seconds)...");
    println!("{}", "-".repeat(50));

    let active_interfaces: Vec<_> = interfaces
        .iter()
        .filter(|i| i.is_up && !i.is_loopback)
        .collect();

    if active_interfaces.is_empty() {
        println!("No active non-loopback interfaces found!");
        return Ok(());
    }

    // Get initial stats
    let mut prev_stats = Vec::new();
    for iface in &active_interfaces {
        match stats::get_interface_stats(iface.index) {
            Ok(stat) => prev_stats.push(stat),
            Err(e) => println!("Failed to get stats for {}: {}", iface.display_name(), e),
        }
    }

    // Monitor for 5 seconds with 1-second intervals
    for i in 1..=5 {
        thread::sleep(Duration::from_secs(1));
        println!("\n[Update {}]", i);

        for (idx, iface) in active_interfaces.iter().enumerate() {
            if let Ok(current_stats) = stats::get_interface_stats(iface.index) {
                if idx < prev_stats.len() {
                    if let Some(bandwidth) = current_stats.calculate_bandwidth(&prev_stats[idx]) {
                        println!(
                            "  {} ({}):",
                            iface.display_name(),
                            if iface.is_virtual() {
                                "Virtual"
                            } else {
                                "Physical"
                            }
                        );
                        println!(
                            "    ↓ Download: {}",
                            format::format_bytes_per_sec(bandwidth.download_rate)
                        );
                        println!(
                            "    ↑ Upload:   {}",
                            format::format_bytes_per_sec(bandwidth.upload_rate)
                        );
                        println!(
                            "    Total: {} down / {} up",
                            format::format_bytes(bandwidth.total_downloaded),
                            format::format_bytes(bandwidth.total_uploaded)
                        );
                    }
                    prev_stats[idx] = current_stats;
                }
            }
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("Monitoring complete!");

    Ok(())
}

// Classic TUI version (original implementation)
fn run_classic_tui() -> Result<()> {
    let interfaces = interface::list_interfaces()?;

    if interfaces.is_empty() {
        eprintln!("No network interfaces found!");
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create classic app and run
    let mut app = App::new(interfaces)?;
    let res = app.run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

/// 도움말 메시지를 출력하는 함수
fn show_help() {
    println!("NBMon v{} - Cross-platform Network Bandwidth Monitor", env!("CARGO_PKG_VERSION"));
    println!("Linux의 nload와 bmon에서 영감을 받은 실시간 네트워크 트래픽 모니터링 도구");
    println!();
    println!("사용법:");
    println!("    nbmon [OPTIONS]");
    println!();
    println!("옵션:");
    println!("    (기본)        향상된 TUI 모드 - Linux nload 스타일의 실시간 그래프");
    println!("    --classic     클래식 TUI 모드 - 단순한 리스트 형태의 인터페이스");
    println!("    --simple      단순 콘솔 모드 - 한 번 출력 후 종료");
    println!("    -h, --help    이 도움말 메시지 출력");
    println!("    -v, --version 버전 정보 출력");
    println!();
    println!("키보드 단축키 (TUI 모드):");
    println!("    ←/h           이전 네트워크 인터페이스");
    println!("    →/l           다음 네트워크 인터페이스");
    println!("    Space         수동 업데이트");
    println!("    r             히스토리 초기화");
    println!("    q             프로그램 종료");
    println!();
    println!("기능:");
    println!("    • 실시간 네트워크 대역폭 모니터링");
    println!("    • 업로드/다운로드 속도 그래프 (스파크라인)");
    println!("    • 다중 네트워크 인터페이스 지원");
    println!("    • Private/Public IP 자동 감지 및 표시");
    println!("    • 크로스플랫폼 지원 (Windows/Linux)");
    println!("    • 낮은 리소스 사용량");
    println!();
    println!("예제:");
    println!("    nbmon                # 기본 향상된 TUI 모드 실행");
    println!("    nbmon --classic      # 클래식 TUI 모드 실행");
    println!("    nbmon --simple       # 간단한 정보 출력 후 종료");
    println!();
    println!("저장소: https://github.com/i978sukhoi/nbmon");
}

/// 버전 정보를 출력하는 함수
fn show_version() {
    println!("NBMon v{}", env!("CARGO_PKG_VERSION"));
    println!("Cross-platform Network Bandwidth Monitor");
    println!("Built with Rust");
}
