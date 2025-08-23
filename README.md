# nbwin

A Windows CLI network bandwidth monitoring tool written in Rust, inspired by Linux tools like nload and bmon.

## Overview

`nbwin` provides real-time network traffic visualization and statistics in the terminal for Windows systems. It aims to bring the familiar experience of Linux network monitoring tools to Windows users with a focus on performance and usability.

## Features (Planned)

- 📊 Real-time network bandwidth monitoring
- 🔍 Per-interface traffic statistics  
- 📈 Terminal-based graphical visualization
- 🪟 Native Windows network API support
- ⚡ Low resource consumption
- ⚙️ Configurable update intervals and display options

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/nbwin.git
cd nbwin

# Build the project
cargo build --release

# Run the executable
./target/release/nbwin
```

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Windows 10/11

## Usage

```bash
# Run with default settings
nbwin

# Show help
nbwin --help

# Monitor specific interface (planned)
nbwin --interface ethernet

# Set update interval (planned)  
nbwin --interval 500
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run directly
cargo run
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Check code without building
cargo check
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

## Project Structure

```
nbwin/
├── src/
│   └── main.rs        # Application entry point
├── Cargo.toml         # Project manifest
├── Cargo.lock         # Dependency lock file
└── README.md          # This file
```

## Technical Architecture

The project leverages:
- **Windows Network APIs** for accurate interface statistics
- **Terminal UI Framework** (crossterm/tui-rs) for real-time display updates
- **Efficient Data Collection** to minimize CPU overhead
- **Multi-interface Support** for comprehensive network monitoring

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Roadmap

- [ ] Basic bandwidth monitoring
- [ ] Multiple interface support
- [ ] Graph visualization
- [ ] Historical data tracking
- [ ] Export statistics
- [ ] Configuration file support
- [ ] Color themes

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by [nload](https://github.com/rolandriegel/nload) and [bmon](https://github.com/tgraf/bmon)
- Built with [Rust](https://www.rust-lang.org/)

## Support

For issues, questions, or suggestions, please open an issue on GitHub.