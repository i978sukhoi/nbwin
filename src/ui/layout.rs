use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

pub struct AppLayout {
    pub header_height: u16,
    pub help_height: u16,
    pub details_height: u16,
}

impl Default for AppLayout {
    fn default() -> Self {
        Self {
            header_height: 3,
            help_height: 3,
            details_height: 6,
        }
    }
}

impl AppLayout {
    pub fn split_main(&self, area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(self.header_height),  // Header
                Constraint::Min(10),                     // Interface list (flexible)
                Constraint::Length(self.details_height), // Selected interface details
                Constraint::Length(self.help_height),    // Help
            ])
            .split(area)
            .to_vec()
    }

    pub fn split_interface_details(&self, area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Basic info
                Constraint::Percentage(50), // Statistics
            ])
            .split(area)
            .to_vec()
    }
}