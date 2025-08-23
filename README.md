# nbwin

Linux의 nload와 bmon에서 영감을 받은 Windows용 CLI 네트워크 대역폭 모니터링 도구입니다. Rust로 작성되었습니다.

## 개요

`nbwin`은 Windows 시스템을 위한 실시간 네트워크 트래픽 시각화 및 통계를 터미널에서 제공합니다. 성능과 사용성에 중점을 둔 Linux 네트워크 모니터링 도구의 친숙한 경험을 Windows 사용자에게 제공하는 것을 목표로 합니다.

## 주요 기능 ✨

- 📊 **실시간 네트워크 대역폭 모니터링** - 즉시 확인 가능한 트래픽 상태
- 🔍 **인터페이스별 트래픽 통계** - 각 네트워크 어댑터 개별 모니터링  
- 📈 **터미널 기반 그래픽 시각화** - 스파크라인 그래프로 실시간 표시
- 🪟 **Windows 네트워크 API 네이티브 지원** - 정확한 시스템 통합
- ⚡ **낮은 리소스 사용량** - 가벼운 실행으로 시스템 부담 최소화
- ⚙️ **3가지 실행 모드** - 향상된 TUI, 클래식 TUI, 단순 콘솔 출력
- 🎯 **키보드 탐색** - 직관적인 인터페이스 전환 (←/→, h/l, Space, r, q)
- 🔧 **물리적/가상 인터페이스 구분** - 네트워크 어댑터 타입 자동 감지

## 설치 방법

### 소스코드로부터 빌드

```bash
# 저장소 복제
git clone https://github.com/i978sukhoi/nbwin.git
cd nbwin

# 프로젝트 빌드
cargo build --release

# 실행 파일 실행
./target/release/nbwin
```

### 시스템 요구사항

- Rust 1.70+ ([rustup.rs](https://rustup.rs/)에서 설치)
- Windows 10/11

## 사용법 💡

```bash
# 기본 향상된 TUI 모드로 실행
cargo run

# 클래식 TUI 모드로 실행  
cargo run -- --classic

# 단순 콘솔 출력 모드로 실행
cargo run -- --simple

# 도움말 보기
cargo run -- --help
```

### 키보드 조작법

- **←/→ 또는 h/l**: 네트워크 인터페이스 전환
- **Space**: 다음 인터페이스로 이동
- **r**: 화면 새로고침
- **q**: 프로그램 종료

## 개발 🛠️

### 빌드

```bash
# 디버그 빌드
cargo build

# 릴리즈 빌드
cargo build --release

# 직접 실행
cargo run
```

### 테스트

```bash
# 모든 테스트 실행
cargo test

# 특정 테스트 실행
cargo test test_name

# 빌드 없이 코드 검사
cargo check
```

### 코드 품질

```bash
# 코드 포맷팅
cargo fmt

# 포맷팅 검사
cargo fmt --check

# 린터 실행
cargo clippy
```

## 프로젝트 구조 📁

```
nbwin/
├── src/
│   ├── main.rs                    # 애플리케이션 진입점
│   ├── lib.rs                     # 라이브러리 모듈 구조
│   ├── network/                   # 네트워크 레이어
│   │   ├── interface.rs           # 인터페이스 관리
│   │   ├── stats.rs               # 통계 수집
│   │   └── windows_api.rs         # Windows API 통합
│   ├── ui/                        # 사용자 인터페이스 레이어
│   │   ├── app.rs                 # 기본 TUI 앱
│   │   ├── app_improved.rs        # 향상된 TUI 앱 (메인)
│   │   ├── layout.rs              # 레이아웃 유틸리티
│   │   └── widgets/               # 커스텀 위젯들
│   └── utils/                     # 유틸리티 함수들
│       └── format.rs              # 데이터 포맷팅
├── examples/
│   └── basic_usage.rs             # 사용 예제 코드
├── Cargo.toml                     # 프로젝트 매니페스트
├── Cargo.lock                     # 의존성 락 파일
└── README.md                      # 이 파일
```

## 기술 아키텍처 🏗️

이 프로젝트는 다음을 활용합니다:
- **Windows 네트워크 API** - 정확한 인터페이스 통계 수집
- **터미널 UI 프레임워크** (ratatui/crossterm) - 실시간 화면 업데이트
- **효율적인 데이터 수집** - CPU 오버헤드 최소화
- **다중 인터페이스 지원** - 포괄적인 네트워크 모니터링
- **SOLID 원칙** - 깨끗한 코드 아키텍처와 유지보수성

## 기여하기 🤝

기여를 환영합니다! 언제든지 Pull Request를 보내주세요.

1. 저장소 포크하기
2. 기능 브랜치 생성 (`git checkout -b feature/awesome-feature`)
3. 변경사항 커밋 (`git commit -m 'Add some awesome feature'`)
4. 브랜치에 푸시 (`git push origin feature/awesome-feature`)
5. Pull Request 열기

## 학습 리소스 📚

이 프로젝트는 **Rust 학습을 위한 교육적 목적**으로 설계되었습니다:

- ✅ **포괄적인 주석**: 모든 소스파일에 Rust 초보자용 상세 설명
- ✅ **핵심 개념 설명**: 소유권, 차용, 패턴 매칭, trait 사용법
- ✅ **실습 예제**: `examples/basic_usage.rs`로 라이브러리 사용법 학습
- ✅ **현대적 패턴**: Iterator 체인, 에러 처리, 함수형 프로그래밍
- ✅ **시스템 프로그래밍**: Windows API 통합과 실시간 데이터 처리

## 추가 개발 가능 기능 💡

선택적 확장 기능들 (현재 상태로도 완전히 실용적):
- [ ] 설정 파일 지원 (.toml 기반)
- [ ] 추가 그래프 타입 (바 차트, 히스토그램)  
- [ ] 데이터 내보내기 기능 (CSV, JSON)
- [ ] 네트워크 알림 기능
- [ ] 다중 인터페이스 동시 표시
- [ ] 색상 테마 지원

## 라이센스 📄

이 프로젝트는 MIT 라이센스 하에 있습니다. 자세한 내용은 LICENSE 파일을 참조하세요.

## 감사의 말 🙏

- [nload](https://github.com/rolandriegel/nload)와 [bmon](https://github.com/tgraf/bmon)에서 영감을 받았습니다
- [Rust](https://www.rust-lang.org/)로 개발되었습니다

## 지원 💬

문제점, 질문 또는 제안사항이 있으시면 GitHub에서 이슈를 열어주세요.