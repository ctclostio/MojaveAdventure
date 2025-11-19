//! # Worldbook Module
//!
//! Persistent world knowledge tracking locations, NPCs, and events.
//!
//! ## Overview
//!
//! The worldbook system maintains a living history of the game world:
//! - **Locations**: Places the player has visited with descriptions and state
//! - **NPCs**: Characters the player has met with disposition and knowledge
//! - **Events**: Timeline of significant happenings in the world
//! - **Relationships**: How NPCs feel about the player
//!
//! ## Location Tracking
//!
//! Each location stores:
//! - Name, description, and type (settlement, ruin, vault, wasteland)
//! - NPCs present at the location
//! - Visit history (first visit, last visit, visit count)
//! - Custom state (quest flags, environmental changes)
//! - Player notes
//!
//! ## NPC System
//!
//! NPCs have:
//! - Name, role, and personality traits
//! - Current location
//! - Disposition toward player (-100 hostile to +100 friendly)
//! - Knowledge (things they know about)
//! - Alive/dead status
//!
//! ## Event Timeline
//!
//! Events record:
//! - Timestamp of when they occurred
//! - Location where they happened
//! - Event type (npc_met, combat, discovery, dialogue)
//! - Description and involved entities
//!
//! ## AI Integration
//!
//! The worldbook provides context to the AI dungeon master, allowing it to:
//! - Remember past interactions and locations
//! - Reference NPCs the player has met
//! - Maintain continuity across play sessions
//! - Generate contextually appropriate responses
//!
//! ## Persistence
//!
//! The worldbook is saved with the game state and persists across sessions,
//! creating a living, evolving wasteland that remembers your actions.
//!
//! ## Example
//!
//! ```no_run
//! use fallout_dnd::game::worldbook::{Worldbook, Location, NPC, WorldEvent};
//! use std::collections::HashMap;
//!
//! let mut worldbook = Worldbook::new();
//!
//! // Add a location
//! let mut megaton = Location {
//!    id: "megaton_01".to_string(),
//!    name: "Megaton".to_string(),
//!    name_lowercase: "megaton".to_string(),
//!    description: "A settlement built around an undetonated atomic bomb".to_string(),
//!    location_type: "settlement".to_string(),
//!    npcs_present: vec![],
//!    atmosphere: None,
//!    first_visited: None,
//!    last_visited: None,
//!    visit_count: 0,
//!    notes: vec![],
//!    state: HashMap::new(),
//! };
//! worldbook.add_location(megaton);
//!
//! // Add an NPC
//! let mut lucas = NPC {
//!     id: "lucas_simms_01".to_string(),
//!     name: "Lucas Simms".to_string(),
//!     name_lowercase: "lucas simms".to_string(),
//!     role: "sheriff".to_string(),
//!     personality: vec![],
//!     current_location: None,
//!     disposition: 50, // Neutral-friendly
//!     knowledge: vec![],
//!     notes: "".to_string(),
//!     alive: true,
//! };
//! worldbook.add_npc(lucas);
//!
//! // Record an event
//! let event = WorldEvent {
//!    timestamp: "2277-10-23T10:00:00Z".to_string(),
//!    location: Some("megaton_01".to_string()),
//!    event_type: "npc_met".to_string(),
//!    description: "Met Sheriff Lucas Simms at the town gates".to_string(),
//!    entities: vec!["lucas_simms_01".to_string()],
//! };
//! worldbook.add_event(event);
//!
//! println!("Known locations: {}", worldbook.locations.len());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Persistent world knowledge base.
///
/// The worldbook maintains the state of the game world including
/// all discovered locations, met NPCs, and significant events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worldbook {
    pub locations: HashMap<String, Location>,
    pub npcs: HashMap<String, NPC>,
    pub events: Vec<WorldEvent>,
    pub current_location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    #[serde(skip)] // Don't serialize - computed from name
    pub name_lowercase: String,
    pub description: String,
    pub location_type: String, // "settlement", "ruin", "vault", "wasteland"
    pub npcs_present: Vec<String>, // NPC IDs
    pub atmosphere: Option<String>,
    pub first_visited: Option<String>,
    pub last_visited: Option<String>,
    pub visit_count: u32,
    pub notes: Vec<String>,
    pub state: HashMap<String, String>, // Custom key-value state
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    pub id: String,
    pub name: String,
    #[serde(skip)] // Don't serialize - computed from name
    pub name_lowercase: String,
    pub role: String,             // "merchant", "guard", "quest_giver", "settler"
    pub personality: Vec<String>, // ["gruff", "honest", "paranoid"]
    pub current_location: Option<String>,
    pub disposition: i32,       // -100 to +100
    pub knowledge: Vec<String>, // Things they know about
    pub notes: String,
    pub alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEvent {
    pub timestamp: String,
    pub location: Option<String>,
    pub event_type: String, // "npc_met", "combat", "discovery", "dialogue"
    pub description: String,
    pub entities: Vec<String>, // NPC/location IDs involved
}

impl Worldbook {
    pub fn new() -> Self {
        Worldbook {
            locations: HashMap::new(),
            npcs: HashMap::new(),
            events: Vec::new(),
            current_location: None,
        }
    }

    /// Creates a new worldbook with default Fallout universe locations.
    ///
    /// Includes Vault 13 as the starting location with appropriate description.
    pub fn with_defaults() -> Self {
        let mut worldbook = Self::new();

        // Add Vault 13 as the default starting location
        let vault_13 = Location {
            id: "vault_13".to_string(),
            name: "Vault 13".to_string(),
            name_lowercase: "vault 13".to_string(),
            description: "One of the great underground Vaults built before the Great War. Vault 13 was designed to remain sealed for 200 years as a test of prolonged isolation. The massive gear-shaped door stands as a testament to pre-war engineering.".to_string(),
            location_type: "vault".to_string(),
            npcs_present: vec![],
            atmosphere: Some("Safe but claustrophobic. The air recyclers hum steadily in the background.".to_string()),
            first_visited: None,
            last_visited: None,
            visit_count: 0,
            notes: vec![],
            state: HashMap::new(),
        };

        worldbook.add_location(vault_13);

        worldbook
    }

    #[allow(dead_code)]
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let json = fs::read_to_string(path)?;
            let mut worldbook: Worldbook = serde_json::from_str(&json)?;
            worldbook.populate_caches();
            Ok(worldbook)
        } else {
            Ok(Worldbook::new())
        }
    }

    /// Populate cached lowercase name fields for all locations and NPCs
    /// Call this after deserialization since cached fields are not serialized
    #[allow(dead_code)]
    fn populate_caches(&mut self) {
        for location in self.locations.values_mut() {
            location.name_lowercase = location.name.to_lowercase();
        }
        for npc in self.npcs.values_mut() {
            npc.name_lowercase = npc.name.to_lowercase();
        }
    }

    pub fn save_to_file(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    // Add a new location or update existing
    pub fn add_location(&mut self, mut location: Location) {
        // Ensure cache is populated
        location.name_lowercase = location.name.to_lowercase();
        self.locations.insert(location.id.clone(), location);
    }

    // Add a new NPC or update existing
    pub fn add_npc(&mut self, mut npc: NPC) {
        // Ensure cache is populated
        npc.name_lowercase = npc.name.to_lowercase();
        self.npcs.insert(npc.id.clone(), npc);
    }

    // Record an event
    pub fn add_event(&mut self, event: WorldEvent) {
        self.events.push(event);
    }

    // Get location by ID
    pub fn get_location(&self, id: &str) -> Option<&Location> {
        self.locations.get(id)
    }

    // Get NPC by ID
    pub fn get_npc(&self, id: &str) -> Option<&NPC> {
        self.npcs.get(id)
    }

    // Get all NPCs at a location
    pub fn get_npcs_at_location(&self, location_id: &str) -> Vec<&NPC> {
        self.npcs
            .values()
            .filter(|npc| npc.current_location.as_deref() == Some(location_id))
            .collect()
    }

    // Get recent events at a location
    pub fn get_location_events(&self, location_id: &str, limit: usize) -> Vec<&WorldEvent> {
        self.events
            .iter()
            .rev()
            .filter(|e| e.location.as_deref() == Some(location_id))
            .take(limit)
            .collect()
    }

    // Update current location
    pub fn set_current_location(&mut self, location_id: Option<String>) {
        self.current_location = location_id;
    }

    // Visit a location (update visit count and timestamp)
    pub fn visit_location(&mut self, location_id: &str) {
        if let Some(location) = self.locations.get_mut(location_id) {
            let now = chrono::Utc::now().to_rfc3339();

            if location.first_visited.is_none() {
                location.first_visited = Some(now.clone());
            }
            location.last_visited = Some(now);
            location.visit_count += 1;
        }
    }

    // Build context string for AI prompt
    pub fn build_context(&self) -> String {
        let mut context = String::new();

        // Current location context
        if let Some(loc_id) = &self.current_location {
            if let Some(location) = self.get_location(loc_id) {
                context.push_str(&format!("\n=== CURRENT LOCATION: {} ===\n", location.name));
                context.push_str(&format!("Type: {}\n", location.location_type));
                context.push_str(&format!("Description: {}\n", location.description));
                context.push_str(&format!("Visits: {}\n", location.visit_count));

                if let Some(atmosphere) = &location.atmosphere {
                    context.push_str(&format!("Atmosphere: {}\n", atmosphere));
                }

                // NPCs present
                let npcs = self.get_npcs_at_location(loc_id);
                if !npcs.is_empty() {
                    context.push_str("NPCs present:\n");
                    for npc in npcs {
                        context.push_str(&format!(
                            "  - {} ({}), disposition: {}\n",
                            npc.name, npc.role, npc.disposition
                        ));
                    }
                }

                // Recent events
                let events = self.get_location_events(loc_id, 3);
                if !events.is_empty() {
                    context.push_str("Recent events here:\n");
                    for event in events {
                        context.push_str(&format!("  - {}\n", event.description));
                    }
                }

                // Notes
                if !location.notes.is_empty() {
                    context.push_str("Notes:\n");
                    for note in &location.notes {
                        context.push_str(&format!("  - {}\n", note));
                    }
                }

                context.push_str("===\n");
            }
        }

        context
    }

    // Generate unique ID from name
    pub fn generate_id(name: &str) -> String {
        name.to_lowercase()
            .replace(" ", "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }
}

impl Default for Worldbook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worldbook_creation() {
        let wb = Worldbook::new();
        assert!(wb.locations.is_empty());
        assert!(wb.npcs.is_empty());
        assert!(wb.events.is_empty());
    }

    #[test]
    fn test_worldbook_with_defaults() {
        let wb = Worldbook::with_defaults();

        // Should have Vault 13 by default
        assert_eq!(wb.locations.len(), 1);
        assert!(wb.locations.contains_key("vault_13"));

        // Verify Vault 13 details
        let vault_13 = wb.get_location("vault_13").unwrap();
        assert_eq!(vault_13.name, "Vault 13");
        assert_eq!(vault_13.location_type, "vault");
        assert!(vault_13.description.contains("underground Vault"));
        assert!(vault_13.atmosphere.is_some());

        // Should still have empty NPCs and events
        assert!(wb.npcs.is_empty());
        assert!(wb.events.is_empty());
    }

    #[test]
    fn test_add_location() {
        let mut wb = Worldbook::new();
        let loc = Location {
            id: "megaton".to_string(),
            name: "Megaton".to_string(),
            name_lowercase: String::new(), // Will be populated by add_location
            description: "Settlement built around bomb".to_string(),
            location_type: "settlement".to_string(),
            npcs_present: vec![],
            atmosphere: Some("tense".to_string()),
            first_visited: None,
            last_visited: None,
            visit_count: 0,
            notes: vec![],
            state: HashMap::new(),
        };

        wb.add_location(loc);
        assert_eq!(wb.locations.len(), 1);
        let added_loc = wb.get_location("megaton").unwrap();
        assert_eq!(added_loc.name_lowercase, "megaton");
    }

    #[test]
    fn test_add_npc() {
        let mut wb = Worldbook::new();
        let npc = NPC {
            id: "marcus".to_string(),
            name: "Marcus".to_string(),
            name_lowercase: String::new(), // Will be populated by add_npc
            role: "trader".to_string(),
            personality: vec!["gruff".to_string()],
            current_location: Some("megaton".to_string()),
            disposition: 0,
            knowledge: vec![],
            notes: String::new(),
            alive: true,
        };

        wb.add_npc(npc);
        assert_eq!(wb.npcs.len(), 1);
        let added_npc = wb.get_npc("marcus").unwrap();
        assert_eq!(added_npc.name_lowercase, "marcus");
    }

    #[test]
    fn test_generate_id() {
        assert_eq!(Worldbook::generate_id("Red Rocket"), "red_rocket");
        assert_eq!(Worldbook::generate_id("Vault 13"), "vault_13");
        assert_eq!(Worldbook::generate_id("Marcus O'Brien"), "marcus_obrien");
    }

    #[test]
    fn test_get_npcs_at_location() {
        let mut wb = Worldbook::new();

        let npc1 = NPC {
            id: "marcus".to_string(),
            name: "Marcus".to_string(),
            name_lowercase: String::new(), // Will be populated by add_npc
            role: "trader".to_string(),
            personality: vec![],
            current_location: Some("megaton".to_string()),
            disposition: 0,
            knowledge: vec![],
            notes: String::new(),
            alive: true,
        };

        let npc2 = NPC {
            id: "sheriff".to_string(),
            name: "Sheriff Simms".to_string(),
            name_lowercase: String::new(), // Will be populated by add_npc
            role: "lawman".to_string(),
            personality: vec![],
            current_location: Some("megaton".to_string()),
            disposition: 10,
            knowledge: vec![],
            notes: String::new(),
            alive: true,
        };

        wb.add_npc(npc1);
        wb.add_npc(npc2);

        let npcs = wb.get_npcs_at_location("megaton");
        assert_eq!(npcs.len(), 2);
    }
}
