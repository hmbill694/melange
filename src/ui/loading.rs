use iced::widget::{column, container, text};
use iced::{Alignment, Element, Fill};

const SPINNER_FRAMES: [char; 8] = ['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'];

/// A reusable centered loading indicator with a braille spinner and label.
///
/// Generic over `Message` so it can be embedded in any module's view without coupling.
pub fn loading_indicator<'a, Message>(label: &str, tick_count: u32) -> Element<'a, Message>
where
    Message: 'a,
{
    let frame = SPINNER_FRAMES[tick_count as usize % SPINNER_FRAMES.len()];

    container(
        column![text(frame.to_string()).size(32), text(label.to_string()),]
            .align_x(Alignment::Center)
            .spacing(12),
    )
    .center(Fill)
    .into()
}
