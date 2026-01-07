use crate::tui::app::{App, InputMode, View};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use ratatui_image::StatefulImage;

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = if app.view == View::Home {
        // Home view: logo + search + content + status
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Logo
                Constraint::Length(3), // Search bar
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(frame.area())
    } else {
        // Other views: search + content + status
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(frame.area())
    };

    if app.view == View::Home {
        render_logo(frame, chunks[0]);
        render_search_bar(frame, app, chunks[1]);
        render_main_content(frame, app, chunks[2]);
        render_status_bar(frame, app, chunks[3]);
    } else {
        render_search_bar(frame, app, chunks[0]);
        render_main_content(frame, app, chunks[1]);
        render_status_bar(frame, app, chunks[2]);
    }

    // Show error popup if present
    if let Some(ref error) = app.error_message {
        render_error_popup(frame, error);
    }

    // Show loading indicator
    if app.loading {
        render_loading(frame, &app.loading_message);
    }
}

fn render_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let style = match app.input_mode {
        InputMode::Normal => Style::default(),
        InputMode::Editing => Style::default().fg(Color::Cyan),
    };

    let input = Paragraph::new(app.search_input.as_str())
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search (press / or s) ")
                .border_style(style),
        );
    frame.render_widget(input, area);

    // Show cursor when editing
    if app.input_mode == InputMode::Editing {
        frame.set_cursor_position((
            area.x + app.cursor_position as u16 + 1,
            area.y + 1,
        ));
    }
}

fn render_main_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Home => render_home(frame, app, area),
        View::SearchResults => render_search_results(frame, app, area),
        View::PostDetail => render_post_detail(frame, app, area),
    }
}

fn render_logo(frame: &mut Frame, area: Rect) {
    let logo_color = Color::Rgb(255, 69, 0); // Reddit orange

    let logo = vec![
        Line::from(Span::styled("  ██████╗ ██████╗ ████████╗██╗   ██╗██╗", Style::default().fg(logo_color))),
        Line::from(Span::styled("  ██╔══██╗██╔══██╗╚══██╔══╝██║   ██║██║", Style::default().fg(logo_color))),
        Line::from(Span::styled("  ██████╔╝██║  ██║   ██║   ██║   ██║██║", Style::default().fg(logo_color))),
        Line::from(Span::styled("  ██╔══██╗██║  ██║   ██║   ██║   ██║██║", Style::default().fg(logo_color))),
        Line::from(Span::styled("  ██║  ██║██████╔╝   ██║   ╚██████╔╝██║", Style::default().fg(logo_color))),
        Line::from(Span::styled("  ╚═╝  ╚═╝╚═════╝    ╚═╝    ╚═════╝ ╚═╝", Style::default().fg(logo_color))),
        Line::from(Span::styled("  Reddit in your terminal", Style::default().fg(Color::Rgb(100, 100, 100)))),
    ];

    let paragraph = Paragraph::new(logo);
    frame.render_widget(paragraph, area);
}

fn render_home(frame: &mut Frame, app: &App, area: Rect) {
    // Posts list (logo is rendered separately above search bar)
    if app.home_posts.is_empty() {
        let loading = Paragraph::new("  Loading r/all...")
            .block(Block::default().borders(Borders::ALL).title(" r/all "));
        frame.render_widget(loading, area);
    } else {
        render_post_list(frame, &app.home_posts, app.selected_post_index, " r/all - Hot ", area);
    }
}

fn render_search_results(frame: &mut Frame, app: &App, area: Rect) {
    let posts = match &app.search_results {
        Some(results) => &results.posts,
        None => {
            let paragraph = Paragraph::new("No results")
                .block(Block::default().borders(Borders::ALL).title(" Results "));
            frame.render_widget(paragraph, area);
            return;
        }
    };

    // Split area for debug info + results
    let chunks = if app.debug_info.is_some() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(0), Constraint::Min(0)])
            .split(area)
    };

    // Debug info line
    if let Some(ref debug) = app.debug_info {
        let debug_text = Paragraph::new(debug.as_str())
            .style(Style::default().fg(Color::Rgb(128, 128, 128)));
        frame.render_widget(debug_text, chunks[0]);
    }

    let title = match &app.search_results {
        Some(r) => format!(
            " {} | sort:{} time:{} ({}) ",
            r.query, app.search_sort, app.search_time, r.count
        ),
        None => " Results ".to_string(),
    };

    render_post_list(frame, posts, app.selected_post_index, &title, chunks[1]);
}

/// Shared post list renderer
fn render_post_list(
    frame: &mut Frame,
    posts: &[crate::api::models::PostSummary],
    selected_index: usize,
    title: &str,
    area: Rect,
) {
    let items: Vec<ListItem> = posts
        .iter()
        .enumerate()
        .map(|(i, post)| {
            let style = if i == selected_index {
                Style::default()
                    .bg(Color::Rgb(40, 44, 52))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let age = format_age(post.created_utc);
            let content = Line::from(vec![
                Span::styled(
                    format!("{:>5} ", post.score),
                    Style::default().fg(Color::Rgb(255, 139, 61)), // Orange for scores
                ),
                Span::styled(
                    format!("r/{:<15} ", post.subreddit),
                    Style::default().fg(Color::Rgb(70, 130, 180)), // Steel blue for subreddits
                ),
                Span::styled(
                    format!("{:<4} ", age),
                    Style::default().fg(Color::Rgb(100, 100, 100)), // Gray for age
                ),
                Span::raw(&post.title),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(list, area);
}

fn render_post_detail(frame: &mut Frame, app: &App, area: Rect) {
    let has_image = app.current_image.borrow().is_some();

    // Header at top, then content below
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Post header (compact)
            Constraint::Min(0),    // Content area
        ])
        .split(area);

    // Post header
    if let Some(ref post) = app.current_post {
        let header_text = vec![
            Line::from(Span::styled(
                &post.title,
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled(
                    format!("r/{}", post.subreddit),
                    Style::default().fg(Color::Rgb(70, 130, 180)),
                ),
                Span::raw(" by "),
                Span::styled(
                    format!("u/{}", post.author),
                    Style::default().fg(Color::Rgb(100, 149, 237)),
                ),
                Span::raw(" | "),
                Span::styled(
                    format!("{} pts", post.score),
                    Style::default().fg(Color::Rgb(255, 139, 61)),
                ),
                Span::raw(format!(" | {} comments", post.num_comments)),
            ]),
        ];

        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL).title(" Post "))
            .wrap(Wrap { trim: true });
        frame.render_widget(header, main_chunks[0]);
    }

    // Content area: image on top (if present), comments below
    let content_area = main_chunks[1];
    let comments_area = if has_image {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Image (half the content area)
                Constraint::Percentage(50), // Comments
            ])
            .split(content_area);

        // Render image
        let mut image_state = app.current_image.borrow_mut();
        if let Some(ref mut protocol) = *image_state {
            let image_widget = StatefulImage::default();
            frame.render_stateful_widget(image_widget, content_chunks[0], protocol);
        }
        content_chunks[1]
    } else {
        content_area
    };

    // Comments with scroll support
    let visible_height = comments_area.height.saturating_sub(2) as usize;
    let scroll = app.scroll_offset as usize;
    let visible_comments = app.visible_comments();

    let comment_items: Vec<ListItem> = visible_comments
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height / 3 + 1) // ~3 lines per comment
        .map(|(i, comment)| {
            let indent = "  ".repeat(comment.depth.min(4) as usize);
            let style = if i == app.selected_comment_index {
                Style::default().bg(Color::Rgb(40, 44, 52))
            } else {
                Style::default()
            };

            // Build reply indicator
            let reply_indicator = if comment.reply_count > 0 {
                if comment.expanded {
                    format!(" [−{}]", comment.reply_count)
                } else {
                    format!(" [+{}]", comment.reply_count)
                }
            } else {
                String::new()
            };

            let age = format_age(comment.created_utc);
            let lines = vec![
                Line::from(vec![
                    Span::raw(indent.clone()),
                    Span::styled(
                        format!("u/{}", comment.author),
                        Style::default().fg(Color::Rgb(100, 149, 237)),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{} pts", comment.score),
                        Style::default().fg(Color::Rgb(255, 139, 61)),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        age,
                        Style::default().fg(Color::Rgb(100, 100, 100)),
                    ),
                    Span::styled(
                        reply_indicator,
                        Style::default().fg(Color::Rgb(100, 100, 100)),
                    ),
                ]),
                Line::from(vec![
                    Span::raw(indent),
                    Span::raw(truncate_comment(&comment.body, 80)),
                ]),
                Line::from(""),
            ];

            ListItem::new(lines).style(style)
        })
        .collect();

    let total_visible = visible_comments.len();
    let scroll_info = if total_visible > 0 {
        format!(" Comments ({}/{}) ", scroll + 1, total_visible)
    } else {
        " Comments (0) ".to_string()
    };

    let comments_list = List::new(comment_items)
        .block(Block::default().borders(Borders::ALL).title(scroll_info));
    frame.render_widget(comments_list, comments_area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = match app.view {
        View::Home => "j/k: Navigate | Enter: View | /: Search | q: Quit",
        View::SearchResults => "j/k: Nav | Enter: View | o: Sort | t: Time | /: Search | q: Back",
        View::PostDetail => "j/k: Navigate | Enter: Expand | d/u: Scroll | q/Esc: Back",
    };

    let mode_indicator = match app.input_mode {
        InputMode::Normal => "",
        InputMode::Editing => "[EDITING] ",
    };

    let text = format!("{}{}", mode_indicator, status);
    let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::Rgb(30, 30, 30)).fg(Color::Rgb(180, 180, 180)));
    frame.render_widget(paragraph, area);
}

fn render_error_popup(frame: &mut Frame, error: &str) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);

    let text = Text::from(vec![
        Line::from(Span::styled(
            "Error",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(error),
    ]);

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(" Error "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_loading(frame: &mut Frame, message: &str) {
    let area = centered_rect(40, 5, frame.area());
    frame.render_widget(Clear, area);

    // Simple spinner using frame count (approximated by time)
    let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() / 100) as usize % spinners.len();

    let text = format!("{} {}", spinners[idx], message);
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Loading "))
        .style(Style::default().fg(Color::Rgb(100, 149, 237))); // Cornflower blue
    frame.render_widget(paragraph, area);
}

/// Helper to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Truncate comment body for display
fn truncate_comment(s: &str, max_len: usize) -> String {
    let s = s.replace('\n', " ");
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s
    }
}

/// Format a timestamp as relative age (e.g., "2h", "3d", "1w")
fn format_age(created_utc: f64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let age_secs = (now - created_utc).max(0.0) as u64;

    if age_secs < 60 {
        format!("{}s", age_secs)
    } else if age_secs < 3600 {
        format!("{}m", age_secs / 60)
    } else if age_secs < 86400 {
        format!("{}h", age_secs / 3600)
    } else if age_secs < 604800 {
        format!("{}d", age_secs / 86400)
    } else if age_secs < 2592000 {
        format!("{}w", age_secs / 604800)
    } else if age_secs < 31536000 {
        format!("{}mo", age_secs / 2592000)
    } else {
        format!("{}y", age_secs / 31536000)
    }
}
