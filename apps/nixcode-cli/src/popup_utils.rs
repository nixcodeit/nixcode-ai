use ratatui::layout::Constraint::Length;
use ratatui::layout::{Constraint, Flex, Layout, Rect};

pub fn popup_area(area: Rect, percent_x: u16) -> Rect {
    let vertical = Layout::vertical([Length(3)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

/// Helper function to create a centered rect using up certain percentage of the available rect
pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Length((r.height.saturating_sub(height)) / 2),
        Constraint::Length(height),
        Constraint::Length((r.height.saturating_sub(height)) / 2),
    ])
    .flex(Flex::Center)
    .split(r);

    Layout::horizontal([
        Constraint::Length((r.width.saturating_sub(width)) / 2),
        Constraint::Length(width),
        Constraint::Length((r.width.saturating_sub(width)) / 2),
    ])
    .flex(Flex::Center)
    .split(popup_layout[1])[1]
}
