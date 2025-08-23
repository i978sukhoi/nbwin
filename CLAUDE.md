# CLAUDE.md

이 파일은 Claude Code (claude.ai/code)가 이 저장소에서 코드 작업을 할 때 참고할 가이드라인을 제공합니다.

## 프로젝트 개요

`nbwin`은 Linux의 nload와 bmon에서 영감을 받은 Windows용 CLI 네트워크 대역폭 모니터링 도구입니다. Rust로 작성되었으며, Windows 시스템에서 실시간 네트워크 트래픽 시각화 및 통계를 터미널로 제공합니다.

### 현재 상태: 🚀 **프로덕션 준비 완료**
- ✅ 완전 기능 TUI 애플리케이션 
- ✅ 대규모 리팩토링 완료 (DRY, 캡슐화, 에러처리)
- ✅ Rust 학습용 교육 주석 200+ 줄 추가
- ✅ 3가지 실행 모드 지원

## 주요 개발 명령어

### 빌드 명령어
- `cargo build` - 디버그 모드로 프로젝트 빌드
- `cargo build --release` - 릴리즈 모드로 프로젝트 빌드
- `cargo run` - 기본 향상된 TUI 모드로 빌드 및 실행
- `cargo run -- --classic` - 클래식 TUI 모드로 실행
- `cargo run -- --simple` - 단순 콘솔 출력 모드로 실행
- `cargo clean` - target 디렉터리 정리

### 테스트 및 품질 관리
- `cargo test` - 모든 테스트 실행
- `cargo test [test_name]` - 특정 테스트 실행
- `cargo check` - 빌드 없이 컴파일 에러 검사
- `cargo clippy` - Rust 린터로 코드 품질 검사
- `cargo fmt` - Rust 스타일 가이드라인에 따라 코드 포맷팅
- `cargo fmt --check` - 파일 수정 없이 포맷팅 검사만 수행

## 프로젝트 구조

표준 Rust/Cargo 규칙을 따르며, 완성된 아키텍처:
```
src/
├── main.rs                    # 애플리케이션 진입점 (완전 주석화)
├── lib.rs                     # 라이브러리 모듈 구조 정의
├── network/                   # 네트워크 레이어 (완성)
│   ├── mod.rs                 
│   ├── interface.rs           # 네트워크 인터페이스 관리
│   ├── stats.rs               # 대역폭 통계 수집
│   └── windows_api.rs         # Windows API 통합
├── ui/                        # UI 레이어 (완성)
│   ├── mod.rs
│   ├── app.rs                 # 기본 TUI 애플리케이션
│   ├── app_improved.rs        # 향상된 TUI (Linux nload 스타일)
│   ├── layout.rs              # 레이아웃 유틸리티
│   └── widgets/               # 커스텀 위젯들
│       ├── mod.rs
│       ├── interface_list.rs  # 인터페이스 목록 위젯
│       └── bandwidth_chart.rs # 대역폭 차트 위젯
└── utils/                     # 유틸리티 (완성)
    ├── mod.rs
    └── format.rs              # 데이터 포맷팅 함수들
```

- `Cargo.toml` - 프로젝트 매니페스트와 의존성 메타데이터
- `target/` - 빌드 출력 디렉터리 (자동 생성, 버전 관리 제외)

## 완료된 주요 기능 ✅

- ✅ **실시간 네트워크 대역폭 모니터링** - Windows API 통합 완료
- ✅ **인터페이스별 트래픽 통계** - 물리적/가상/루프백 구분
- ✅ **터미널 기반 그래픽 시각화** - 스파크라인 그래프와 동적 범례
- ✅ **Windows 네트워크 API 지원** - winapi + windows crate 활용
- ✅ **낮은 리소스 사용량** - 효율적인 데이터 수집 구현
- ✅ **3가지 실행 모드** - 향상된 TUI, 클래식 TUI, 단순 콘솔
- ✅ **키보드 탐색** - ←/→, h/l, Space, r, q 지원

## 기술적 구현 상세

- ✅ **Windows API 통합** - `winapi`와 `windows` crate로 정확한 인터페이스 통계
- ✅ **터미널 UI 프레임워크** - `ratatui`와 `crossterm`으로 실시간 업데이트
- ✅ **효율적인 데이터 수집** - CPU 사용량 최소화된 폴링 방식
- ✅ **다중 인터페이스 처리** - 안전한 인덱스 접근과 graceful 에러 처리
- ✅ **SOLID 원칙 적용** - DRY, 캡슐화, 단일 책임 원칙 준수
- ✅ **포괄적인 에러 처리** - `anyhow`를 활용한 context 기반 에러 관리

## 교육적 가치 📚

이 프로젝트는 **Rust 학습 리소스**로 설계됨:
- ✅ 모든 소스파일에 Rust 초보자용 상세 주석
- ✅ 소유권, 차용, 패턴 매칭 개념 설명
- ✅ Iterator 체인, trait 사용법, 에러 처리 패턴 문서화
- ✅ `examples/basic_usage.rs`로 실습 가능한 예제 제공