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
}
