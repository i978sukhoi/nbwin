# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`nbwin` is a Windows CLI network bandwidth monitoring tool written in Rust, inspired by Linux tools like nload and bmon. The project aims to provide real-time network traffic visualization and statistics in the terminal for Windows systems.

## Common Development Commands

### Build Commands
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build the project in release mode
- `cargo run` - Build and run the project
- `cargo clean` - Clean the target directory

### Testing and Quality
- `cargo test` - Run all tests
- `cargo test [test_name]` - Run a specific test
- `cargo check` - Check the code for compilation errors without building
- `cargo clippy` - Run the Rust linter for code quality checks
- `cargo fmt` - Format the code according to Rust style guidelines
- `cargo fmt --check` - Check if code is properly formatted without modifying files

## Project Structure

The project follows standard Rust/Cargo conventions:
- `src/main.rs` - Entry point for the application
- `Cargo.toml` - Project manifest with dependencies and metadata
- `target/` - Build output directory (auto-generated, not tracked in version control)

## Key Features (Planned)

- Real-time network bandwidth monitoring
- Per-interface traffic statistics
- Terminal-based graphical visualization
- Support for Windows network APIs
- Low resource consumption
- Configurable update intervals and display options

## Technical Considerations

- Use Windows-specific APIs for network interface statistics
- Terminal UI framework for real-time updates (e.g., crossterm, tui-rs)
- Efficient data collection to minimize CPU usage
- Handle multiple network interfaces gracefully