use crate::api::client::RedditClient;
use crate::api::models::{CommentSummary, PostSummary, SearchResults};
use crate::error::Result;
use crate::nlp::router::NlpRouter;
use crate::tui::ui;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::cell::RefCell;
use std::time::Duration;

/// Current view/screen in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Home,
    SearchResults,
    PostDetail,
}

/// Input mode for the search bar
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// Main application state
pub struct App {
    pub running: bool,
    pub view: View,
    pub input_mode: InputMode,

    // Search state
    pub search_input: String,
    pub cursor_position: usize,
    pub search_sort: String,
    pub search_time: String,

    // Data
    pub home_posts: Vec<PostSummary>,
    pub search_results: Option<SearchResults>,
    pub selected_post_index: usize,
    pub current_post: Option<PostSummary>,
    pub comments: Vec<CommentSummary>,
    pub selected_comment_index: usize,

    // Loading state
    pub loading: bool,
    pub loading_message: String,
    pub error_message: Option<String>,

    // Debug info
    pub debug_info: Option<String>,

    // Scroll state for post detail
    pub scroll_offset: u16,

    // Image support
    pub image_picker: Option<Picker>,
    pub current_image: RefCell<Option<StatefulProtocol>>,
}

impl App {
    pub fn new() -> Self {
        // Try to detect terminal image capabilities
        let image_picker = Picker::from_query_stdio().ok();

        Self {
            running: true,
            view: View::Home,
            input_mode: InputMode::Normal,
            search_input: String::new(),
            cursor_position: 0,
            search_sort: "relevance".to_string(),
            search_time: "all".to_string(),
            home_posts: Vec::new(),
            search_results: None,
            selected_post_index: 0,
            current_post: None,
            comments: Vec::new(),
            selected_comment_index: 0,
            loading: true, // Start loading
            loading_message: "Loading...".to_string(),
            error_message: None,
            debug_info: None,
            scroll_offset: 0,
            image_picker,
            current_image: RefCell::new(None),
        }
    }

    /// Load r/all posts for homepage
    pub async fn load_home_posts(&mut self) -> Result<()> {
        self.loading = true;
        self.loading_message = "Loading r/all...".to_string();
        let client = RedditClient::new().await?;
        match client.get_subreddit_posts("all", "hot", "day", 25).await {
            Ok(posts) => {
                self.home_posts = posts;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load posts: {}", e));
            }
        }
        self.loading = false;
        Ok(())
    }

    /// Load an image from URL
    pub async fn load_image(&mut self, url: &str) {
        if let Some(ref picker) = self.image_picker {
            // Fetch image bytes
            let client = reqwest::Client::new();
            match client.get(url).send().await {
                Ok(response) => {
                    if let Ok(bytes) = response.bytes().await {
                        // Decode image
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let protocol = picker.new_resize_protocol(img);
                            *self.current_image.borrow_mut() = Some(protocol);
                        }
                    }
                }
                Err(_) => {
                    *self.current_image.borrow_mut() = None;
                }
            }
        }
    }

    /// Main event loop
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // Load r/all posts on startup
        self.load_home_posts().await?;

        while self.running {
            // Draw UI
            terminal.draw(|frame| ui::render(frame, self))
                .map_err(|e| crate::error::RdtError::Tui(e.to_string()))?;

            // Handle events with timeout to allow async operations
            if crossterm::event::poll(Duration::from_millis(100))
                .map_err(|e| crate::error::RdtError::Tui(e.to_string()))?
            {
                if let Event::Key(key) = crossterm::event::read()
                    .map_err(|e| crate::error::RdtError::Tui(e.to_string()))?
                {
                    self.handle_key(key.code, key.modifiers).await?;
                }
            }
        }
        Ok(())
    }

    /// Handle keyboard input
    async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        // Clear error on any key press
        self.error_message = None;

        match self.input_mode {
            InputMode::Editing => self.handle_editing_key(key).await?,
            InputMode::Normal => self.handle_normal_key(key, modifiers).await?,
        }
        Ok(())
    }

    /// Handle keys in editing mode (search input)
    async fn handle_editing_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.perform_search().await?;
            }
            KeyCode::Char(c) => {
                self.search_input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.search_input.remove(self.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.search_input.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in normal mode
    async fn handle_normal_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        match key {
            // Quit
            KeyCode::Char('q') => {
                if self.view == View::Home {
                    self.running = false;
                } else {
                    self.go_back();
                }
            }
            KeyCode::Esc => {
                self.go_back();
            }

            // Search
            KeyCode::Char('/') | KeyCode::Char('s') => {
                self.input_mode = InputMode::Editing;
            }

            // Navigation
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
            }
            KeyCode::Enter => {
                self.select_item().await?;
            }

            // Scrolling in post detail
            KeyCode::Char('d') => {
                if self.view == View::PostDetail {
                    let max_scroll = self.visible_comments().len().saturating_sub(1);
                    self.scroll_offset = self.scroll_offset.saturating_add(10).min(max_scroll as u16);
                }
            }
            KeyCode::Char('u') => {
                if self.view == View::PostDetail {
                    self.scroll_offset = self.scroll_offset.saturating_sub(10);
                }
            }

            // Sort/time filters (in SearchResults view)
            KeyCode::Char('o') => {
                if self.view == View::SearchResults {
                    self.cycle_sort();
                    self.rerun_search().await?;
                }
            }
            KeyCode::Char('t') => {
                if self.view == View::SearchResults {
                    self.cycle_time();
                    self.rerun_search().await?;
                }
            }

            _ => {}
        }
        Ok(())
    }

    fn go_back(&mut self) {
        match self.view {
            View::Home => {
                self.running = false;
            }
            View::SearchResults => {
                self.view = View::Home;
                self.search_results = None;
                self.selected_post_index = 0;
            }
            View::PostDetail => {
                // Go back to wherever we came from
                if self.search_results.is_some() {
                    self.view = View::SearchResults;
                } else {
                    self.view = View::Home;
                }
                self.current_post = None;
                self.comments.clear();
                self.selected_comment_index = 0;
                self.scroll_offset = 0;
                *self.current_image.borrow_mut() = None;
            }
        }
    }

    fn move_down(&mut self) {
        match self.view {
            View::Home => {
                if self.selected_post_index < self.home_posts.len().saturating_sub(1) {
                    self.selected_post_index += 1;
                }
            }
            View::SearchResults => {
                if let Some(ref results) = self.search_results {
                    if self.selected_post_index < results.posts.len().saturating_sub(1) {
                        self.selected_post_index += 1;
                    }
                }
            }
            View::PostDetail => {
                let visible_count = self.visible_comments().len();
                if self.selected_comment_index < visible_count.saturating_sub(1) {
                    self.selected_comment_index += 1;
                    // Auto-scroll to keep selection visible (assume ~3 lines per comment, 10 visible)
                    let visible_window = 10usize;
                    if self.selected_comment_index > self.scroll_offset as usize + visible_window {
                        self.scroll_offset = (self.selected_comment_index - visible_window) as u16;
                    }
                }
            }
        }
    }

    fn move_up(&mut self) {
        match self.view {
            View::Home | View::SearchResults => {
                if self.selected_post_index > 0 {
                    self.selected_post_index -= 1;
                }
            }
            View::PostDetail => {
                if self.selected_comment_index > 0 {
                    self.selected_comment_index -= 1;
                    // Auto-scroll to keep selection visible
                    if self.selected_comment_index < self.scroll_offset as usize {
                        self.scroll_offset = self.selected_comment_index as u16;
                    }
                }
            }
        }
    }

    async fn select_item(&mut self) -> Result<()> {
        // In PostDetail view, Enter toggles comment expansion
        if self.view == View::PostDetail {
            self.toggle_comment_expansion();
            return Ok(());
        }

        let post = match self.view {
            View::Home => self.home_posts.get(self.selected_post_index).cloned(),
            View::SearchResults => self
                .search_results
                .as_ref()
                .and_then(|r| r.posts.get(self.selected_post_index).cloned()),
            View::PostDetail => return Ok(()),
        };

        if let Some(post) = post {
            self.current_post = Some(post.clone());
            self.loading = true;
            *self.current_image.borrow_mut() = None; // Clear previous image

            // Load image if post has one
            if let Some(ref image_url) = post.image_url {
                self.load_image(image_url).await;
            }

            // Fetch comments
            match self.fetch_comments(&post.id).await {
                Ok(comments) => {
                    self.comments = comments;
                    self.view = View::PostDetail;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load comments: {}", e));
                }
            }
            self.loading = false;
        }
        Ok(())
    }

    /// Toggle expansion of the currently selected comment
    fn toggle_comment_expansion(&mut self) {
        if let Some(comment) = self.get_visible_comment_mut(self.selected_comment_index) {
            if comment.reply_count > 0 {
                comment.expanded = !comment.expanded;
            }
        }
    }

    /// Get mutable reference to a comment by its visible index
    fn get_visible_comment_mut(&mut self, index: usize) -> Option<&mut CommentSummary> {
        let mut current_index = 0;
        Self::find_comment_mut(&mut self.comments, index, &mut current_index)
    }

    fn find_comment_mut<'a>(
        comments: &'a mut [CommentSummary],
        target_index: usize,
        current_index: &mut usize,
    ) -> Option<&'a mut CommentSummary> {
        for comment in comments.iter_mut() {
            if *current_index == target_index {
                return Some(comment);
            }
            *current_index += 1;

            if comment.expanded && !comment.replies.is_empty() {
                if let Some(found) = Self::find_comment_mut(&mut comment.replies, target_index, current_index) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Get flattened visible comments (respecting expansion state)
    pub fn visible_comments(&self) -> Vec<&CommentSummary> {
        let mut result = Vec::new();
        Self::collect_visible(&self.comments, &mut result);
        result
    }

    fn collect_visible<'a>(comments: &'a [CommentSummary], result: &mut Vec<&'a CommentSummary>) {
        for comment in comments {
            result.push(comment);
            if comment.expanded {
                Self::collect_visible(&comment.replies, result);
            }
        }
    }

    async fn perform_search(&mut self) -> Result<()> {
        use crate::nlp::router::ParseMethod;

        if self.search_input.is_empty() {
            return Ok(());
        }

        self.loading = true;
        self.loading_message = "Parsing query...".to_string();
        self.error_message = None;

        let router = NlpRouter::new();
        let mut params = router.parse_query(&self.search_input).await?;

        // Build debug info
        let method_str = match params.parse_method {
            Some(ParseMethod::Pattern) => "pattern",
            Some(ParseMethod::AI) => "AI",
            Some(ParseMethod::Fallback) => "fallback (no AI)",
            None => "unknown",
        };
        self.debug_info = Some(format!(
            "[{}] query=\"{}\" sub={:?}",
            method_str,
            params.query,
            params.subreddit
        ));

        // Apply UI sort/time overrides
        params.sort = self.search_sort.clone();
        params.time = self.search_time.clone();

        self.loading_message = "Searching Reddit...".to_string();
        let client = RedditClient::new().await?;
        match client.search(&params).await {
            Ok(results) => {
                self.search_results = Some(results);
                self.view = View::SearchResults;
                self.selected_post_index = 0;
            }
            Err(e) => {
                self.error_message = Some(format!("Search failed: {}", e));
            }
        }

        self.loading = false;
        Ok(())
    }

    /// Cycle through sort options
    fn cycle_sort(&mut self) {
        const SORTS: &[&str] = &["relevance", "hot", "top", "new"];
        let current = SORTS.iter().position(|&s| s == self.search_sort).unwrap_or(0);
        let next = (current + 1) % SORTS.len();
        self.search_sort = SORTS[next].to_string();
    }

    /// Cycle through time options
    fn cycle_time(&mut self) {
        const TIMES: &[&str] = &["all", "day", "week", "month", "year"];
        let current = TIMES.iter().position(|&t| t == self.search_time).unwrap_or(0);
        let next = (current + 1) % TIMES.len();
        self.search_time = TIMES[next].to_string();
    }

    /// Re-run current search with new filters
    async fn rerun_search(&mut self) -> Result<()> {
        if self.search_input.is_empty() {
            return Ok(());
        }
        self.perform_search().await
    }

    async fn fetch_comments(&self, post_id: &str) -> Result<Vec<CommentSummary>> {
        let client = RedditClient::new().await?;
        client.get_comments(post_id, "best", 50).await
    }
}
