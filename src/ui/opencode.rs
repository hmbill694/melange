use iced::widget::{column, container, text};
use iced::{Alignment, Element, Fill};

const OPENCODE_INSTALL_URL: &str = "https://opencode.ai";

/// Renders a centered, full-screen "opencode not installed" block screen.
///
/// Generic over `Message` so it can be embedded in any module's view without coupling.
pub fn opencode_not_found_screen<'a, Message>() -> Element<'a, Message>
where
    Message: 'a,
{
    container(
        column![
            text("opencode is required").size(28),
            text("Melange requires opencode to be installed and available on your PATH."),
            text("To install, visit:"),
            text(OPENCODE_INSTALL_URL).size(16),
        ]
        .spacing(12)
        .align_x(Alignment::Center),
    )
    .center(Fill)
    .into()
}
