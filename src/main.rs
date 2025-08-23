// Rust에서 외부 라이브러리(crate)를 import하는 방법
// anyhow: 에러 처리를 간단하게 해주는 라이브러리
use anyhow::Result;
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
use nbwin::{App, ImprovedApp, network::interface};

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
            "--simple" => return run_simple_mode(),    // 간단한 콘솔 모드
            "--classic" => return run_classic_tui(),   // 클래식 TUI 모드
            _ => {}  // 와일드카드 패턴: 나머지 모든 경우 (기본값)
        }
    }

    // 네트워크 인터페이스 목록 가져오기
    // ? 연산자: Result가 Err이면 함수에서 바로 에러를 반환
    let interfaces = interface::list_interfaces()?;
    
    // Vec이 비어있는지 확인
    if interfaces.is_empty() {
        // eprintln!: stderr로 출력하는 매크로
        eprintln!("No network interfaces found!");
        // Ok(()): 성공적인 결과 반환
        return Ok(());
    }

    // 터미널을 TUI 모드로 설정
    // raw mode: 터미널이 입력을 즉시 프로그램에 전달 (Enter 없이도)
    enable_raw_mode()?;
    
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
    let mut app = ImprovedApp::new(interfaces)?;
    // app.run()의 결과를 res 변수에 저장
    let res = app.run(&mut terminal);

    // 터미널 상태 복원 (정리 작업)
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(), // backend에 대한 mutable 참조 가져오기
        LeaveAlternateScreen,    // 원래 화면으로 복귀
        DisableMouseCapture     // 마우스 캡처 비활성화
    )?;
    terminal.show_cursor()?;    // 커서 다시 보이게 하기

    // res가 에러인 경우 출력
    // if let: 패턴 매칭으로 특정 케이스만 처리
    if let Err(err) = res {
        // {:?}: Debug trait를 사용한 포맷팅
        println!("{err:?}");
    }

    Ok(())  // 성공적으로 완료
}

// 간단한 콘솔 출력 모드 - TUI 없이 텍스트만 출력
fn run_simple_mode() -> Result<()> {
    // 이 함수에서만 사용할 모듈들을 지역적으로 import
    // use 문을 함수 내부에 쓰면 해당 함수에서만 사용 가능
    use nbwin::network::stats;
    use nbwin::utils::format;
    use std::{thread, time::Duration};  // 스레드와 시간 관련 기능

    // println!: 콘솔에 텍스트 출력하는 매크로
    println!("NBWin - Network Bandwidth Monitor for Windows (Simple Mode)");
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
        println!("    Type: {}", 
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
                println!("        - {}", ip);
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
                        println!("  {} ({}):", 
                            iface.display_name(),
                            if iface.is_virtual() { "Virtual" } else { "Physical" }
                        );
                        println!("    ↓ Download: {}", 
                            format::format_bytes_per_sec(bandwidth.download_rate));
                        println!("    ↑ Upload:   {}", 
                            format::format_bytes_per_sec(bandwidth.upload_rate));
                        println!("    Total: {} down / {} up", 
                            format::format_bytes(bandwidth.total_downloaded),
                            format::format_bytes(bandwidth.total_uploaded));
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