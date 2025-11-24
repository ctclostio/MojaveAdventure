use crate::game::worldbook::{Location, WorldEvent, Worldbook, NPC};
use crate::templates;
use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use smartstring::alias::String as SmartString;
use std::time::Duration;

/// Function calling schema for entity extraction
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct FunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ExtractionRequest {
    prompt: String,
    temperature: f32,
    top_p: f32,
    top_k: i32,
    n_predict: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    grammar: Option<String>,
    stop: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExtractionResponse {
    content: String,
    #[serde(default)]
    error: Option<String>,
}

/// Extracted entities from AI narrative
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExtractedEntities {
    pub locations: Vec<ExtractedLocation>,
    pub npcs: Vec<ExtractedNPC>,
    pub events: Vec<ExtractedEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedLocation {
    pub name: String,
    pub description: String,
    pub location_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedNPC {
    pub name: String,
    pub role: String,
    pub personality: Vec<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEvent {
    pub event_type: String,
    pub description: String,
    pub location: Option<String>,
    pub entities: Vec<String>,
}

#[derive(Clone)]
pub struct ExtractionAI {
    server_url: String,
    client: reqwest::Client,
}

impl ExtractionAI {
    pub fn new(server_url: String) -> Self {
        ExtractionAI {
            server_url,
            client: reqwest::Client::new(),
        }
    }

    /// Extract entities from narrative text using a smaller AI model
    pub async fn extract_entities(&self, narrative: &str) -> Result<ExtractedEntities> {
        let prompt = self.build_extraction_prompt(narrative);

        let request = ExtractionRequest {
            prompt,
            temperature: 0.1, // Low temperature for consistent extraction
            top_p: 0.9,
            top_k: 40,
            n_predict: 1024,
            grammar: None,
            stop: vec!["</extraction>".to_string()],
        };

        let url = format!("{}/completion", self.server_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to connect to extraction AI at {}: {}",
                    self.server_url,
                    e
                )
            })?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Extraction AI returned error: {}",
                response.status()
            ));
        }

        let extraction_response: ExtractionResponse = response.json().await?;

        if let Some(error) = extraction_response.error {
            return Err(anyhow!("Extraction AI error: {}", error));
        }

        // Parse the JSON response
        self.parse_extraction(&extraction_response.content)
    }

    /// Build extraction prompt with examples using template
    fn build_extraction_prompt(&self, narrative: &str) -> String {
        match templates::render_extractor_prompt(narrative) {
            Ok(prompt) => prompt,
            Err(e) => {
                tracing::error!("Failed to render extractor template: {}", e);
                // Fallback to hardcoded prompt if template fails
                self.build_extraction_prompt_fallback(narrative)
            }
        }
    }

    /// Legacy fallback for extraction prompt
    fn build_extraction_prompt_fallback(&self, narrative: &str) -> String {
        format!(
            r#"You are an expert entity extractor for a Fallout RPG game. Extract all NPCs, locations, and events from the narrative text.

Output ONLY valid JSON in this exact format (no other text):
{{
  "locations": [
    {{"name": "Location Name", "description": "Brief description", "location_type": "settlement|ruin|vault|wasteland"}}
  ],
  "npcs": [
    {{"name": "NPC Name", "role": "merchant|guard|settler|raider|other", "personality": ["trait1", "trait2"], "location": "Location Name or null"}}
  ],
  "events": [
    {{"event_type": "npc_met|combat|discovery|dialogue", "description": "What happened", "location": "Location Name or null", "entities": ["entity1", "entity2"]}}
  ]
}}

Rules:
- Only extract NEW information not already known
- Capitalize proper nouns (names of people and places)
- Personality traits should be single descriptive words
- If no entities of a type, use empty array []
- Location type must be one of: settlement, ruin, vault, wasteland
- Event type must be one of: npc_met, combat, discovery, dialogue

Example 1:
Narrative: "You arrive at Megaton, a settlement built around an unexploded atomic bomb. Sheriff Lucas Simms, a stern lawman, greets you warily."
Output:
{{
  "locations": [
    {{"name": "Megaton", "description": "Settlement built around unexploded atomic bomb", "location_type": "settlement"}}
  ],
  "npcs": [
    {{"name": "Sheriff Lucas Simms", "role": "guard", "personality": ["stern", "wary"], "location": "Megaton"}}
  ],
  "events": [
    {{"event_type": "npc_met", "description": "Met Sheriff Lucas Simms", "location": "Megaton", "entities": ["Sheriff Lucas Simms"]}}
  ]
}}

Example 2:
Narrative: "The raider charges at you, firing wildly!"
Output:
{{
  "locations": [],
  "npcs": [],
  "events": [
    {{"event_type": "combat", "description": "Raider attacked", "location": null, "entities": ["raider"]}}
  ]
}}

Now extract from this narrative:
"{}"

Output JSON:"#,
            narrative.replace("\"", "\\\"")
        )
    }

    /// Parse extraction response
    pub fn parse_extraction(&self, content: &str) -> Result<ExtractedEntities> {
        // Try to find JSON in the response
        let json_str = if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        serde_json::from_str(json_str).map_err(|e| {
            anyhow!(
                "Failed to parse extraction JSON: {}. Content: {}",
                e,
                json_str
            )
        })
    }

    /// Test connection to extraction AI server
    pub async fn test_connection(&self) -> Result<()> {
        let url = format!("{}/health", self.server_url);

        self.client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|_| {
                anyhow!(
                    "Cannot connect to extraction AI at {}. Is it running?",
                    self.server_url
                )
            })?;

        Ok(())
    }
}

/// Convert extracted entities to worldbook entries
impl ExtractedEntities {
    /// Convert to worldbook entries (returns locations, npcs, events)
    pub fn to_worldbook_entries(&self) -> (Vec<Location>, Vec<NPC>, Vec<WorldEvent>) {
        let mut locations = Vec::new();
        let mut npcs = Vec::new();
        let mut events = Vec::new();

        // Convert locations
        for loc in &self.locations {
            let id = Worldbook::generate_id(&loc.name);
            locations.push(Location {
                id: id.clone(),
                name: loc.name.clone().into(),
                name_lowercase: loc.name.to_lowercase().into(),
                description: loc.description.clone().into(),
                location_type: loc.location_type.clone().into(),
                npcs_present: Vec::new(),
                atmosphere: None,
                first_visited: None,
                last_visited: None,
                visit_count: 0,
                notes: Vec::new(),
                state: std::collections::HashMap::new(),
            });
        }

        // Convert NPCs
        for npc in &self.npcs {
            let id = Worldbook::generate_id(&npc.name);
            npcs.push(NPC {
                id: id.clone(),
                name: npc.name.clone().into(),
                name_lowercase: npc.name.to_lowercase().into(),
                role: npc.role.clone().into(),
                personality: npc.personality.iter().map(|s| s.clone().into()).collect(),
                current_location: npc.location.as_ref().map(|l| Worldbook::generate_id(l)),
                disposition: 0,
                knowledge: Vec::new(),
                notes: SmartString::new(),
                alive: true,
            });
        }

        // Convert events
        for event in &self.events {
            events.push(WorldEvent {
                timestamp: chrono::Utc::now().to_rfc3339().into(),
                location: event.location.as_ref().map(|l| Worldbook::generate_id(l)),
                event_type: event.event_type.clone().into(),
                description: event.description.clone().into(),
                entities: event
                    .entities
                    .iter()
                    .map(|e| Worldbook::generate_id(e))
                    .collect(),
            });
        }

        (locations, npcs, events)
    }

    /// Check if there are any entities
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty() && self.npcs.is_empty() && self.events.is_empty()
    }

    /// Get summary string for user display
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.locations.is_empty() {
            parts.push(format!("{} location(s)", self.locations.len()));
        }
        if !self.npcs.is_empty() {
            parts.push(format!("{} NPC(s)", self.npcs.len()));
        }
        if !self.events.is_empty() {
            parts.push(format!("{} event(s)", self.events.len()));
        }

        if parts.is_empty() {
            "No new entities".to_string()
        } else {
            format!("Found: {}", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // JSON PARSING TESTS
    // ============================================================================

    #[test]
    fn test_parse_extraction() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [{"name": "Megaton", "description": "Settlement", "location_type": "settlement"}],
            "npcs": [{"name": "Marcus", "role": "trader", "personality": ["gruff"], "location": "Megaton"}],
            "events": [{"event_type": "npc_met", "description": "Met Marcus", "location": "Megaton", "entities": ["Marcus"]}]
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.locations.len(), 1);
        assert_eq!(entities.npcs.len(), 1);
        assert_eq!(entities.events.len(), 1);
    }

    #[test]
    fn test_parse_valid_extraction_minimal() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{"locations": [], "npcs": [], "events": []}"#;
        let entities = extractor.parse_extraction(json).unwrap();

        assert!(entities.locations.is_empty());
        assert!(entities.npcs.is_empty());
        assert!(entities.events.is_empty());
    }

    #[test]
    fn test_parse_extraction_with_embedded_json() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let content = r#"Some narrative text and then the JSON:
        {
            "locations": [{"name": "Vault 101", "description": "Underground vault", "location_type": "vault"}],
            "npcs": [],
            "events": []
        }
        And some more text after"#;

        let entities = extractor.parse_extraction(content).unwrap();
        assert_eq!(entities.locations.len(), 1);
        assert_eq!(entities.locations[0].name, "Vault 101");
        assert_eq!(entities.locations[0].location_type, "vault");
    }

    #[test]
    fn test_parse_malformed_json() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{"locations": [{"name": "Test"}], invalid json"#;
        let result = extractor.parse_extraction(json);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_fields() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        // Missing 'location_type' field in location
        let json = r#"{
            "locations": [{"name": "Test", "description": "Test"}],
            "npcs": [],
            "events": []
        }"#;

        let result = extractor.parse_extraction(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json_completely() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let result = extractor.parse_extraction("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_with_extra_fields() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        // JSON with extra unknown fields (should still parse)
        let json = r#"{
            "locations": [{"name": "Test", "description": "Test", "location_type": "settlement", "extra_field": "should be ignored"}],
            "npcs": [],
            "events": [],
            "unknown_key": "value"
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.locations.len(), 1);
    }

    // ============================================================================
    // LOCATION EXTRACTION TESTS
    // ============================================================================

    #[test]
    fn test_extract_location_basic() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [{
                "name": "Megaton",
                "description": "Settlement built around an unexploded bomb",
                "location_type": "settlement"
            }],
            "npcs": [],
            "events": []
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.locations.len(), 1);
        assert_eq!(entities.locations[0].name, "Megaton");
        assert_eq!(
            entities.locations[0].description,
            "Settlement built around an unexploded bomb"
        );
        assert_eq!(entities.locations[0].location_type, "settlement");
    }

    #[test]
    fn test_extract_multiple_locations() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [
                {"name": "Megaton", "description": "Settlement", "location_type": "settlement"},
                {"name": "Super Duper Mart", "description": "Ruined store", "location_type": "ruin"},
                {"name": "Vault 101", "description": "Underground vault", "location_type": "vault"}
            ],
            "npcs": [],
            "events": []
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.locations.len(), 3);
        assert_eq!(entities.locations[0].location_type, "settlement");
        assert_eq!(entities.locations[1].location_type, "ruin");
        assert_eq!(entities.locations[2].location_type, "vault");
    }

    #[test]
    fn test_extract_wasteland_location() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [{
                "name": "Capital Wasteland",
                "description": "Irradiated zone",
                "location_type": "wasteland"
            }],
            "npcs": [],
            "events": []
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.locations[0].location_type, "wasteland");
    }

    // ============================================================================
    // NPC EXTRACTION TESTS
    // ============================================================================

    #[test]
    fn test_extract_npc_basic() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [],
            "npcs": [{
                "name": "Lucas Simms",
                "role": "guard",
                "personality": ["stern", "wary"],
                "location": "Megaton"
            }],
            "events": []
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.npcs.len(), 1);
        assert_eq!(entities.npcs[0].name, "Lucas Simms");
        assert_eq!(entities.npcs[0].role, "guard");
        assert_eq!(entities.npcs[0].personality.len(), 2);
        assert_eq!(entities.npcs[0].location, Some("Megaton".to_string()));
    }

    #[test]
    fn test_extract_npc_without_location() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [],
            "npcs": [{
                "name": "Unknown Trader",
                "role": "merchant",
                "personality": ["cautious"],
                "location": null
            }],
            "events": []
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.npcs[0].location, None);
    }

    #[test]
    fn test_extract_multiple_npcs() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [],
            "npcs": [
                {"name": "Marcus", "role": "trader", "personality": ["gruff", "honest"], "location": "Megaton"},
                {"name": "Sheriff Simms", "role": "guard", "personality": ["stern"], "location": "Megaton"},
                {"name": "Raider Chief", "role": "raider", "personality": ["aggressive", "cruel"], "location": null}
            ],
            "events": []
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.npcs.len(), 3);
        assert_eq!(entities.npcs[0].role, "trader");
        assert_eq!(entities.npcs[1].role, "guard");
        assert_eq!(entities.npcs[2].role, "raider");
    }

    #[test]
    fn test_extract_npc_with_multiple_personality_traits() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [],
            "npcs": [{
                "name": "Complex Character",
                "role": "settler",
                "personality": ["brave", "cautious", "friendly", "determined"],
                "location": null
            }],
            "events": []
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.npcs[0].personality.len(), 4);
        assert!(entities.npcs[0].personality.contains(&"brave".to_string()));
        assert!(entities.npcs[0]
            .personality
            .contains(&"determined".to_string()));
    }

    // ============================================================================
    // EVENT EXTRACTION TESTS
    // ============================================================================

    #[test]
    fn test_extract_npc_met_event() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [],
            "npcs": [],
            "events": [{
                "event_type": "npc_met",
                "description": "Met Sheriff Lucas Simms",
                "location": "Megaton",
                "entities": ["Sheriff Lucas Simms"]
            }]
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.events.len(), 1);
        assert_eq!(entities.events[0].event_type, "npc_met");
        assert_eq!(entities.events[0].location, Some("Megaton".to_string()));
        assert_eq!(entities.events[0].entities.len(), 1);
    }

    #[test]
    fn test_extract_combat_event() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [],
            "npcs": [],
            "events": [{
                "event_type": "combat",
                "description": "Engaged raiders",
                "location": null,
                "entities": ["raider", "raider", "raider"]
            }]
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.events[0].event_type, "combat");
        assert_eq!(entities.events[0].entities.len(), 3);
    }

    #[test]
    fn test_extract_discovery_event() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [],
            "npcs": [],
            "events": [{
                "event_type": "discovery",
                "description": "Found Vault Boy holotape",
                "location": "Vault 101",
                "entities": ["Vault Boy holotape"]
            }]
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.events[0].event_type, "discovery");
    }

    #[test]
    fn test_extract_dialogue_event() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [],
            "npcs": [],
            "events": [{
                "event_type": "dialogue",
                "description": "Learned about the Brotherhood of Steel",
                "location": "Rivet City",
                "entities": ["Brotherhood of Steel"]
            }]
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.events[0].event_type, "dialogue");
    }

    #[test]
    fn test_extract_multiple_events() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [],
            "npcs": [],
            "events": [
                {"event_type": "npc_met", "description": "Met Marcus", "location": "Megaton", "entities": ["Marcus"]},
                {"event_type": "combat", "description": "Fought raiders", "location": null, "entities": ["raider"]},
                {"event_type": "discovery", "description": "Found ammo", "location": "Ruin", "entities": ["ammo"]}
            ]
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.events.len(), 3);
        assert_eq!(entities.events[0].event_type, "npc_met");
        assert_eq!(entities.events[1].event_type, "combat");
        assert_eq!(entities.events[2].event_type, "discovery");
    }

    // ============================================================================
    // EMPTY/EDGE CASE TESTS
    // ============================================================================

    #[test]
    fn test_extracted_entities_is_empty() {
        let empty = ExtractedEntities::default();
        assert!(empty.is_empty());

        let not_empty = ExtractedEntities {
            locations: vec![ExtractedLocation {
                name: "Test".to_string(),
                description: "Test".to_string(),
                location_type: "wasteland".to_string(),
            }],
            npcs: vec![],
            events: vec![],
        };
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_extracted_entities_is_not_empty_with_npcs() {
        let entities = ExtractedEntities {
            locations: vec![],
            npcs: vec![ExtractedNPC {
                name: "Test".to_string(),
                role: "merchant".to_string(),
                personality: vec![],
                location: None,
            }],
            events: vec![],
        };
        assert!(!entities.is_empty());
    }

    #[test]
    fn test_extracted_entities_is_not_empty_with_events() {
        let entities = ExtractedEntities {
            locations: vec![],
            npcs: vec![],
            events: vec![ExtractedEvent {
                event_type: "combat".to_string(),
                description: "Test".to_string(),
                location: None,
                entities: vec![],
            }],
        };
        assert!(!entities.is_empty());
    }

    #[test]
    fn test_parse_json_with_whitespace_variations() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        // JSON with various whitespace
        let json = r#"{
  "locations"  :  [  {  "name"  :  "Test"  ,  "description"  :  "Desc"  ,  "location_type"  :  "settlement"  }  ]  ,
  "npcs"  :  [  ]  ,
  "events"  :  [  ]
}"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.locations.len(), 1);
    }

    #[test]
    fn test_parse_json_with_unicode_characters() {
        let extractor = ExtractionAI::new("http://localhost:8081".to_string());

        let json = r#"{
            "locations": [{"name": "Мегатон", "description": "Поселение", "location_type": "settlement"}],
            "npcs": [{"name": "José", "role": "trader", "personality": ["café"], "location": null}],
            "events": []
        }"#;

        let entities = extractor.parse_extraction(json).unwrap();
        assert_eq!(entities.locations[0].name, "Мегатон");
        assert_eq!(entities.npcs[0].name, "José");
    }

    // ============================================================================
    // WORLDBOOK CONVERSION TESTS
    // ============================================================================

    #[test]
    fn test_to_worldbook_entries_empty() {
        let entities = ExtractedEntities::default();
        let (locations, npcs, events) = entities.to_worldbook_entries();

        assert!(locations.is_empty());
        assert!(npcs.is_empty());
        assert!(events.is_empty());
    }

    #[test]
    fn test_to_worldbook_entries_with_location() {
        let entities = ExtractedEntities {
            locations: vec![ExtractedLocation {
                name: "Megaton".to_string(),
                description: "Settlement built around bomb".to_string(),
                location_type: "settlement".to_string(),
            }],
            npcs: vec![],
            events: vec![],
        };

        let (locations, npcs, events) = entities.to_worldbook_entries();

        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].name, "Megaton");
        assert_eq!(locations[0].description, "Settlement built around bomb");
        assert_eq!(locations[0].location_type, "settlement");
        assert_eq!(locations[0].visit_count, 0);
        assert!(npcs.is_empty());
        assert!(events.is_empty());
    }

    #[test]
    fn test_to_worldbook_entries_with_npc() {
        let entities = ExtractedEntities {
            locations: vec![],
            npcs: vec![ExtractedNPC {
                name: "Marcus".to_string(),
                role: "trader".to_string(),
                personality: vec!["gruff".to_string(), "honest".to_string()],
                location: Some("Megaton".to_string()),
            }],
            events: vec![],
        };

        let (locations, npcs, events) = entities.to_worldbook_entries();

        assert!(locations.is_empty());
        assert_eq!(npcs.len(), 1);
        assert_eq!(npcs[0].name, "Marcus");
        assert_eq!(npcs[0].role, "trader");
        assert_eq!(npcs[0].personality.len(), 2);
        assert_eq!(npcs[0].disposition, 0);
        assert!(npcs[0].alive);
        assert!(events.is_empty());
    }

    #[test]
    fn test_to_worldbook_entries_with_event() {
        let entities = ExtractedEntities {
            locations: vec![],
            npcs: vec![],
            events: vec![ExtractedEvent {
                event_type: "npc_met".to_string(),
                description: "Met Marcus at the gate".to_string(),
                location: Some("Megaton".to_string()),
                entities: vec!["Marcus".to_string()],
            }],
        };

        let (locations, npcs, events) = entities.to_worldbook_entries();

        assert!(locations.is_empty());
        assert!(npcs.is_empty());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "npc_met");
        assert_eq!(events[0].description, "Met Marcus at the gate");
    }

    #[test]
    fn test_to_worldbook_entries_complete() {
        let entities = ExtractedEntities {
            locations: vec![ExtractedLocation {
                name: "Megaton".to_string(),
                description: "Settlement".to_string(),
                location_type: "settlement".to_string(),
            }],
            npcs: vec![ExtractedNPC {
                name: "Marcus".to_string(),
                role: "trader".to_string(),
                personality: vec!["gruff".to_string()],
                location: Some("Megaton".to_string()),
            }],
            events: vec![ExtractedEvent {
                event_type: "npc_met".to_string(),
                description: "Met Marcus".to_string(),
                location: Some("Megaton".to_string()),
                entities: vec!["Marcus".to_string()],
            }],
        };

        let (locations, npcs, events) = entities.to_worldbook_entries();

        assert_eq!(locations.len(), 1);
        assert_eq!(npcs.len(), 1);
        assert_eq!(events.len(), 1);
    }

    // ============================================================================
    // SUMMARY TESTS
    // ============================================================================

    #[test]
    fn test_summary_empty() {
        let entities = ExtractedEntities::default();
        assert_eq!(entities.summary(), "No new entities");
    }

    #[test]
    fn test_summary_with_location_only() {
        let entities = ExtractedEntities {
            locations: vec![ExtractedLocation {
                name: "Test".to_string(),
                description: "Test".to_string(),
                location_type: "settlement".to_string(),
            }],
            npcs: vec![],
            events: vec![],
        };
        assert_eq!(entities.summary(), "Found: 1 location(s)");
    }

    #[test]
    fn test_summary_with_multiple_locations() {
        let entities = ExtractedEntities {
            locations: vec![
                ExtractedLocation {
                    name: "Loc1".to_string(),
                    description: "Test".to_string(),
                    location_type: "settlement".to_string(),
                },
                ExtractedLocation {
                    name: "Loc2".to_string(),
                    description: "Test".to_string(),
                    location_type: "ruin".to_string(),
                },
            ],
            npcs: vec![],
            events: vec![],
        };
        assert_eq!(entities.summary(), "Found: 2 location(s)");
    }

    #[test]
    fn test_summary_with_npcs_only() {
        let entities = ExtractedEntities {
            locations: vec![],
            npcs: vec![ExtractedNPC {
                name: "Test".to_string(),
                role: "merchant".to_string(),
                personality: vec![],
                location: None,
            }],
            events: vec![],
        };
        assert_eq!(entities.summary(), "Found: 1 NPC(s)");
    }

    #[test]
    fn test_summary_with_events_only() {
        let entities = ExtractedEntities {
            locations: vec![],
            npcs: vec![],
            events: vec![ExtractedEvent {
                event_type: "combat".to_string(),
                description: "Test".to_string(),
                location: None,
                entities: vec![],
            }],
        };
        assert_eq!(entities.summary(), "Found: 1 event(s)");
    }

    #[test]
    fn test_summary_with_all_types() {
        let entities = ExtractedEntities {
            locations: vec![ExtractedLocation {
                name: "Loc".to_string(),
                description: "Test".to_string(),
                location_type: "settlement".to_string(),
            }],
            npcs: vec![ExtractedNPC {
                name: "NPC".to_string(),
                role: "merchant".to_string(),
                personality: vec![],
                location: None,
            }],
            events: vec![ExtractedEvent {
                event_type: "combat".to_string(),
                description: "Test".to_string(),
                location: None,
                entities: vec![],
            }],
        };
        let summary = entities.summary();
        assert!(summary.contains("1 location(s)"));
        assert!(summary.contains("1 NPC(s)"));
        assert!(summary.contains("1 event(s)"));
    }

    #[test]
    fn test_summary_with_mixed_counts() {
        let entities = ExtractedEntities {
            locations: vec![
                ExtractedLocation {
                    name: "Loc1".to_string(),
                    description: "Test".to_string(),
                    location_type: "settlement".to_string(),
                },
                ExtractedLocation {
                    name: "Loc2".to_string(),
                    description: "Test".to_string(),
                    location_type: "ruin".to_string(),
                },
            ],
            npcs: vec![
                ExtractedNPC {
                    name: "NPC1".to_string(),
                    role: "merchant".to_string(),
                    personality: vec![],
                    location: None,
                },
                ExtractedNPC {
                    name: "NPC2".to_string(),
                    role: "guard".to_string(),
                    personality: vec![],
                    location: None,
                },
                ExtractedNPC {
                    name: "NPC3".to_string(),
                    role: "settler".to_string(),
                    personality: vec![],
                    location: None,
                },
            ],
            events: vec![ExtractedEvent {
                event_type: "combat".to_string(),
                description: "Test".to_string(),
                location: None,
                entities: vec![],
            }],
        };
        let summary = entities.summary();
        assert!(summary.contains("2 location(s)"));
        assert!(summary.contains("3 NPC(s)"));
        assert!(summary.contains("1 event(s)"));
    }
}
