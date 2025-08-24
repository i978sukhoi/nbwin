# nbmon

**Cross-platform Network Bandwidth Monitor**

A fast, cross-platform network bandwidth monitoring tool inspired by Linux's `nload` and `bmon`, written in Rust.

`nbmon` provides real-time network traffic visualization and statistics in your terminal with a focus on performance and usability. It brings the familiar experience of Linux network monitoring tools to both Windows and Linux users.

## ✨ Features

- **🚀 Real-time network bandwidth monitoring** with live charts
- **📊 Multiple display modes**: Enhanced TUI, Classic TUI, and Simple console output
- **🖥️ Cross-platform support**: Windows and Linux
- **⚡ High performance**: Parallel statistics collection with 44% performance improvement  
- **🎯 Interface selection**: Navigate between network interfaces with keyboard shortcuts
- **📈 Historical data**: 60-second bandwidth history with sparkline graphs
- **🛠️ Robust error handling**: Comprehensive error reporting and graceful fallbacks
- **🔍 Performance benchmarking**: Built-in tools to measure collection efficiency

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/username/nbmon.git
cd nbmon
cargo build --release
```

### Usage

```bash
# Default enhanced TUI mode
./target/release/nbmon

# Classic TUI mode  
./target/release/nbmon --classic

# Simple console output
./target/release/nbmon --simple

# Performance benchmark
cargo run --example benchmark_parallel
```

## 🎮 Controls

### Enhanced TUI Mode
- **←/→ or h/l**: Switch between network interfaces
- **Space**: Refresh statistics manually
- **r**: Reset bandwidth history and peak rates
- **q**: Quit application

### Classic TUI Mode
- **↑/↓**: Navigate interface list
- **q**: Quit application

## 📋 System Requirements

- **Windows**: Windows 10/11 with administrative privileges for network access
- **Linux**: Any modern distribution with `/proc/net/dev` and `/sys/class/net` support
- **CPU**: Multi-core recommended for optimal parallel performance
- **Memory**: Minimal (< 10MB typical usage)

## 🏗️ Architecture

```
nbmon/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root and exports
│   ├── error.rs             # Error handling and debugging
│   ├── network/             # Network monitoring layer
│   │   ├── interface.rs     # Network interface management
│   │   ├── stats.rs         # Statistics collection and calculation
│   │   ├── parallel_stats.rs # High-performance parallel collection
│   │   ├── windows_api.rs   # Windows-specific network APIs
│   │   └── linux_api.rs     # Linux-specific network APIs  
│   ├── ui/                  # User interface layer
│   │   ├── app.rs           # Classic TUI application
│   │   ├── app_improved.rs  # Enhanced TUI with charts
│   │   └── widgets/         # Custom UI components
│   └── utils/               # Utility functions
└── examples/                # Usage examples and benchmarks
```

## 🔧 Development

### Build Commands
- `cargo build` - Debug build
- `cargo build --release` - Optimized release build
- `cargo run` - Run enhanced TUI mode
- `cargo run -- --classic` - Run classic TUI mode
- `cargo run -- --simple` - Run simple console mode
- `cargo run --example benchmark_parallel` - Performance benchmark
- `cargo clean` - Clean build artifacts

### Testing & Quality
- `cargo test` - Run all tests
- `cargo check` - Check for compile errors
- `cargo clippy` - Lint code quality
- `cargo fmt` - Format code

## 📊 Performance

NBMon uses parallel statistics collection for optimal performance:

- **44% faster** than sequential collection on multi-core systems
- **2.5x speedup** for active interfaces monitoring
- Automatic scaling based on CPU core count
- Graceful fallback to sequential processing if needed

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license.

## 🙏 Acknowledgments

- Inspired by Linux `nload` and `bmon` tools
- Built with [Rust](https://rust-lang.org/), [Ratatui](https://github.com/ratatui/ratatui), and [Rayon](https://github.com/rayon-rs/rayon)
- Cross-platform networking powered by Windows APIs and Linux `/proc` filesystem