//! View functions for the project bounded context.
//!
//! All functions are pure view logic — no I/O, no async.

use iced::widget::{column, container, row, text, text_input};
use iced::Element;

use crate::modules::project::domain::Project;
use crate::modules::project::message::ProjectMessage;

/// Render the home screen project browser.
///
/// # Arguments
/// - `projects`     — full list of loaded projects
/// - `search_query` — current value of the search input
/// - `window_width` — current window width in logical pixels; controls grid vs. list layout
pub fn home_screen<'a>(
    projects: &'a [Project],
    search_query: &'a str,
    window_width: f32,
) -> Element<'a, ProjectMessage> {
    // 1. Build search bar.
    let search_bar =
        text_input("Search projects…", search_query).on_input(ProjectMessage::SearchChanged);

    // 2. Filter projects by search query.
    let query = search_query.to_lowercase();
    let filtered: Vec<&Project> = projects
        .iter()
        .filter(|p| {
            if query.is_empty() {
                return true;
            }
            let name_matches = p.name.to_lowercase().contains(&query);
            let desc_matches = p
                .description
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(&query);
            name_matches || desc_matches
        })
        .collect();

    // 3. Build project cards.
    let cards: Vec<Element<'a, ProjectMessage>> =
        filtered.iter().map(|p| project_card(p)).collect();

    // 4. Choose layout based on window width.
    let content: Element<'a, ProjectMessage> = if window_width >= 900.0 {
        grid_layout(cards)
    } else {
        list_layout(cards)
    };

    // 5. Compose final element.
    column![search_bar, content].spacing(16).padding(24).into()
}

/// Render a single project card.
fn project_card<'a>(p: &'a Project) -> Element<'a, ProjectMessage> {
    // Name — primary identifier, larger text.
    let name_widget = text(p.name.as_str()).size(18);

    // Optional description.
    let mut card_items: Vec<Element<'a, ProjectMessage>> = vec![name_widget.into()];

    if let Some(ref desc) = p.description {
        card_items.push(text(desc.as_str()).size(13).into());
    }

    // File path — small supporting text.
    let path_widget = text(p.file_path.to_string_lossy().to_string()).size(12);
    card_items.push(path_widget.into());

    let card_column = column(card_items).spacing(4);

    container(card_column).padding(12).into()
}

/// Arrange cards in a 3-column grid.
fn grid_layout<'a>(cards: Vec<Element<'a, ProjectMessage>>) -> Element<'a, ProjectMessage> {
    // Consume the vec into owned chunks of 3 without borrowing.
    let mut remaining = cards;
    let mut rows: Vec<Element<'a, ProjectMessage>> = Vec::new();

    while !remaining.is_empty() {
        let take = remaining.len().min(3);
        let mut row_items: Vec<Element<'a, ProjectMessage>> = remaining.drain(..take).collect();

        // Pad incomplete rows with empty containers so columns stay aligned.
        while row_items.len() < 3 {
            row_items.push(container(column([])).padding(12).into());
        }

        rows.push(row(row_items).spacing(12).into());
    }

    column(rows).spacing(12).into()
}

/// Arrange cards in a single vertical list.
fn list_layout<'a>(cards: Vec<Element<'a, ProjectMessage>>) -> Element<'a, ProjectMessage> {
    column(cards).spacing(8).into()
}
