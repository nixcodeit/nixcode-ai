use ratatui::layout::Constraint::Length;
use ratatui::layout::{Constraint, Flex, Layout, Rect};

pub fn popup_area(area: Rect, percent_x: u16) -> Rect {
    let vertical = Layout::vertical([Length(3)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
