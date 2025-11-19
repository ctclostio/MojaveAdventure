//! Interactive Worldbook Browser
//!
//! Provides a rich UI for exploring the worldbook with tabs, tree navigation,
//! and detailed views of locations, NPCs, and events.

use crate::game::worldbook::{Location, WorldEvent, Worldbook, NPC};
use std::collections::HashMap;

/// Focus state for worldbook navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldbookFocus {
    TabBar,
    List,
}

/// Worldbook browser tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldbookTab {
    Locations,
    NPCs,
    Events,
    Search,
}

impl WorldbookTab {
    pub fn next(&self) -> Self {
        match self {
            Self::Locations => Self::NPCs,
            Self::NPCs => Self::Events,
            Self::Events => Self::Search,
            Self::Search => Self::Locations,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Locations => Self::Search,
            Self::NPCs => Self::Locations,
            Self::Events => Self::NPCs,
            Self::Search => Self::Events,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Locations => "Locations",
            Self::NPCs => "NPCs",
            Self::Events => "Events",
            Self::Search => "Search",
        }
    }
}

/// Tree node for hierarchical location display
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LocationTreeNode {
    pub id: String,
    pub name: String,
    pub expanded: bool,
    pub children: Vec<String>, // IDs of child locations
}

/// Worldbook browser state
#[derive(Debug, Clone)]
pub struct WorldbookBrowser {
    /// Currently active tab
    pub active_tab: WorldbookTab,

    /// Current focus (tab bar or list)
    pub focus: WorldbookFocus,

    /// Selected index in the current list
    pub selected_index: usize,

    /// Scroll offset for lists
    pub scroll_offset: usize,

    /// Search query
    #[allow(dead_code)]
    pub search_query: String,

    /// Whether search input is active
    #[allow(dead_code)]
    pub search_active: bool,

    /// Expanded location IDs (for tree view)
    pub expanded_locations: HashMap<String, bool>,

    /// Detail scroll offset
    pub detail_scroll: usize,
}

impl WorldbookBrowser {
    pub fn new() -> Self {
        Self {
            active_tab: WorldbookTab::Locations,
            focus: WorldbookFocus::TabBar,
            selected_index: 0,
            scroll_offset: 0,
            search_query: String::new(),
            search_active: false,
            expanded_locations: HashMap::new(),
            detail_scroll: 0,
        }
    }

    /// Switch to the next tab
    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.detail_scroll = 0;
    }

    /// Switch to the previous tab
    pub fn prev_tab(&mut self) {
        self.active_tab = self.active_tab.prev();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.detail_scroll = 0;
    }

    /// Move focus to tab bar
    pub fn focus_tab_bar(&mut self) {
        self.focus = WorldbookFocus::TabBar;
    }

    /// Move focus to list
    pub fn focus_list(&mut self) {
        self.focus = WorldbookFocus::List;
    }

    /// Check if tab bar is focused
    pub fn is_tab_bar_focused(&self) -> bool {
        self.focus == WorldbookFocus::TabBar
    }

    /// Check if list is focused
    pub fn is_list_focused(&self) -> bool {
        self.focus == WorldbookFocus::List
    }

    /// Move selection up
    pub fn select_prev(&mut self, max_items: usize) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if max_items > 0 {
            self.selected_index = max_items - 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self, max_items: usize) {
        if max_items == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1) % max_items;
    }

    /// Toggle expansion of the currently selected location
    pub fn toggle_expansion(&mut self, location_id: &str) {
        let expanded = self
            .expanded_locations
            .entry(location_id.to_string())
            .or_insert(false);
        *expanded = !*expanded;
    }

    /// Check if a location is expanded
    pub fn is_expanded(&self, location_id: &str) -> bool {
        self.expanded_locations
            .get(location_id)
            .copied()
            .unwrap_or(false)
    }

    /// Get sorted locations for display
    pub fn get_sorted_locations<'a>(
        &self,
        worldbook: &'a Worldbook,
    ) -> Vec<(&'a String, &'a Location)> {
        let mut locations: Vec<_> = worldbook.locations.iter().collect();

        // Sort by last visited (most recent first), then by name
        locations.sort_by(|a, b| match (&b.1.last_visited, &a.1.last_visited) {
            (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.1.name.cmp(&b.1.name),
        });

        locations
    }

    /// Get sorted NPCs for display
    pub fn get_sorted_npcs<'a>(&self, worldbook: &'a Worldbook) -> Vec<(&'a String, &'a NPC)> {
        let mut npcs: Vec<_> = worldbook.npcs.iter().collect();

        // Sort by name
        npcs.sort_by(|a, b| a.1.name.cmp(&b.1.name));

        npcs
    }

    /// Get events in reverse chronological order
    pub fn get_sorted_events<'a>(&self, worldbook: &'a Worldbook) -> Vec<&'a WorldEvent> {
        worldbook.events.iter().rev().collect()
    }

    /// Filter items by search query
    #[allow(dead_code)]
    pub fn matches_search(&self, text: &str) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        text.to_lowercase()
            .contains(&self.search_query.to_lowercase())
    }

    /// Scroll detail view up
    pub fn scroll_detail_up(&mut self) {
        if self.detail_scroll > 0 {
            self.detail_scroll -= 1;
        }
    }

    /// Scroll detail view down
    pub fn scroll_detail_down(&mut self, max_scroll: usize) {
        if self.detail_scroll < max_scroll {
            self.detail_scroll += 1;
        }
    }

    /// Reset detail scroll
    #[allow(dead_code)]
    pub fn reset_detail_scroll(&mut self) {
        self.detail_scroll = 0;
    }
}

impl Default for WorldbookBrowser {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to format relative time
pub fn format_relative_time(timestamp: &str) -> String {
    // For now, just return "Recently" or the raw timestamp
    // You could use chrono to calculate actual relative time
    if timestamp.is_empty() {
        "Unknown".to_string()
    } else {
        // Parse the RFC3339 timestamp and format nicely
        match chrono::DateTime::parse_from_rfc3339(timestamp) {
            Ok(dt) => {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(dt);

                if duration.num_seconds() < 60 {
                    "Just now".to_string()
                } else if duration.num_minutes() < 60 {
                    format!("{}m ago", duration.num_minutes())
                } else if duration.num_hours() < 24 {
                    format!("{}h ago", duration.num_hours())
                } else if duration.num_days() < 7 {
                    format!("{}d ago", duration.num_days())
                } else {
                    dt.format("%Y-%m-%d").to_string()
                }
            }
            Err(_) => timestamp.to_string(),
        }
    }
}

/// Get visit status string
pub fn get_visit_status(location: &Location) -> String {
    if location.visit_count == 0 {
        "Unexplored".to_string()
    } else if location.visit_count == 1 {
        "Discovered".to_string()
    } else if let Some(last_visited) = &location.last_visited {
        format!("Last: {}", format_relative_time(last_visited))
    } else {
        format!("Visited: {}x", location.visit_count)
    }
}

/// Get NPC disposition string
pub fn get_disposition_string(disposition: i32) -> (&'static str, &'static str) {
    match disposition {
        d if d >= 75 => ("Friendly", "💚"),
        d if d >= 25 => ("Neutral", "💛"),
        d if d >= -25 => ("Cautious", "🟠"),
        d if d >= -75 => ("Hostile", "🔴"),
        _ => ("Hated", "💀"),
    }
}
