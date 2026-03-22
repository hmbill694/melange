use iced::widget::{container, text};
use iced::{Alignment, Element, Fill};

/// Full-width app bar displaying the application name centered at the top of every screen.
///
/// Generic over `Message` so it can be embedded in any screen's view without coupling.
pub fn app_bar<'a, Message>() -> Element<'a, Message>
where
    Message: 'a,
{
    container(text("Melange"))
        .width(Fill)
        .align_x(Alignment::Center)
        .into()
}
