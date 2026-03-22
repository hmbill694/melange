//! View functions for the project bounded context.
//!
//! All functions are pure view logic — no I/O, no async.

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::modules::project::domain::Project;
use crate::modules::project::message::ProjectMessage;

/// Render the home screen project browser.
///
/// # Arguments
/// - `projects`     — full list of loaded projects
/// - `search_query` — current value of the search input
/// - `window_width` — current window width in logical pixels; controls grid vs. list layout
/// - `on_create_project` — message to send when Create Project button is clicked
pub fn home_screen<'a>(
    projects: &'a [Project],
    search_query: &'a str,
    window_width: f32,
    on_create_project: ProjectMessage,
) -> Element<'a, ProjectMessage> {
    // 1. Build search bar with Create Project button.
    let search_bar =
        text_input("Search projects…", search_query).on_input(ProjectMessage::SearchChanged);

    let create_project_button = button("Create Project").on_press(on_create_project);

    let search_row = row![search_bar, create_project_button]
        .spacing(16)
        .align_y(iced::Alignment::Center);

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
    column![search_row, content].spacing(16).padding(24).into()
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

/// Render the Create Project form screen.
///
/// # Arguments
/// - `state` — reference to CreateProjectState for form field values
/// - `on_name_changed` — callback for project name input changes
/// - `on_path_changed` — callback for file path input changes
/// - `on_submit` — message to send when form is submitted
/// - `on_cancel` — message to send when Cancel button is clicked
/// - `on_back` — message to send when Back button is clicked
pub fn create_project_screen<'a, F1, F2>(
    state: &'a crate::ui::app::state::CreateProjectState,
    on_name_changed: F1,
    on_path_changed: F2,
    on_browse: ProjectMessage,
    on_submit: ProjectMessage,
    on_cancel: ProjectMessage,
    on_back: ProjectMessage,
) -> Element<'a, ProjectMessage>
where
    F1: 'a + Fn(String) -> ProjectMessage,
    F2: 'a + Fn(String) -> ProjectMessage,
{
    // Header section with Back button and title
    let back_button = button("< Back").on_press(on_back);

    let title = text("Create Project").size(24);

    let header_row = row![back_button, title]
        .spacing(16)
        .align_y(iced::Alignment::Center);

    // Project Name input
    let name_label = text("Project Name").size(14);

    let name_input =
        text_input("Enter project name", &state.project_name).on_input(on_name_changed);

    let name_section = column![name_label, name_input].spacing(8);

    // File Path input with Browse button
    let path_label = text("File Path").size(14);

    let path_input = text_input("/path/to/project", &state.file_path)
        .on_input(on_path_changed)
        .width(Length::Fill); // Input takes available space

    let browse_button = button("Browse").on_press(on_browse); // Fires ProjectMessage::BrowseForFilePath

    let path_input_row = row![path_input, browse_button]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    let path_section = column![path_label, path_input_row].spacing(8);

    // Error message display (if present)
    let error_display: Element<'a, ProjectMessage> = if let Some(ref error) = state.error_message {
        text(error).size(14).into()
    } else {
        container(column![]).into()
    };

    // Button row with Cancel and Create Project buttons
    let cancel_button = button("Cancel").on_press(on_cancel);

    let create_button = button("Create Project").on_press(on_submit);

    let button_row = row![cancel_button, create_button].spacing(16);

    // Compose all sections
    let content = column![
        header_row,
        name_section,
        path_section,
        error_display,
        button_row
    ]
    .spacing(24)
    .padding(24);

    content.into()
}
