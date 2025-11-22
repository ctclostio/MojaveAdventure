/// Comprehensive tests for worldbook system
use fallout_dnd::ai::extractor::{
    ExtractedEntities, ExtractedEvent, ExtractedLocation, ExtractedNPC, ExtractionAI,
};
use fallout_dnd::game::worldbook::{Location, WorldEvent, Worldbook, NPC};
use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

// ========== Snapshot Tests for AI Extraction ==========

#[test]
fn snapshot_extraction_simple_megaton() {
    let extractor = ExtractionAI::new("http://localhost:8081".to_string());

    let json = r#"{
        "locations": [{
            "name": "Megaton",
            "description": "Settlement built around unexploded atomic bomb",
            "location_type": "settlement"
        }],
        "npcs": [{
            "name": "Sheriff Lucas Simms",
            "role": "guard",
            "personality": ["stern", "wary"],
            "location": "Megaton"
        }],
        "events": [{
            "event_type": "npc_met",
            "description": "Met Sheriff Lucas Simms",
            "location": "Megaton",
            "entities": ["Sheriff Lucas Simms"]
        }]
    }"#;

    let entities = extractor.parse_extraction(json).unwrap();
    insta::assert_json_snapshot!(entities, @r###"
    {
      "locations": [
        {
          "name": "Megaton",
          "description": "Settlement built around unexploded atomic bomb",
          "location_type": "settlement"
        }
      ],
      "npcs": [
        {
          "name": "Sheriff Lucas Simms",
          "role": "guard",
          "personality": [
            "stern",
            "wary"
          ],
          "location": "Megaton"
        }
      ],
      "events": [
        {
          "event_type": "npc_met",
          "description": "Met Sheriff Lucas Simms",
          "location": "Megaton",
          "entities": [
            "Sheriff Lucas Simms"
          ]
        }
      ]
    }
    "###);
}

#[test]
fn snapshot_extraction_combat_encounter() {
    let extractor = ExtractionAI::new("http://localhost:8081".to_string());

    let json = r#"{
        "locations": [],
        "npcs": [],
        "events": [{
            "event_type": "combat",
            "description": "Attacked by raiders in the wasteland",
            "location": null,
            "entities": ["raider"]
        }]
    }"#;

    let entities = extractor.parse_extraction(json).unwrap();
    insta::assert_json_snapshot!(entities, @r###"
    {
      "locations": [],
      "npcs": [],
      "events": [
        {
          "event_type": "combat",
          "description": "Attacked by raiders in the wasteland",
          "location": null,
          "entities": [
            "raider"
          ]
        }
      ]
    }
    "###);
}

#[test]
fn snapshot_extraction_complex_scenario() {
    let extractor = ExtractionAI::new("http://localhost:8081".to_string());

    let json = r#"{
        "locations": [
            {
                "name": "Vault 101",
                "description": "Underground vault where you grew up",
                "location_type": "vault"
            },
            {
                "name": "Springvale",
                "description": "Abandoned town near Vault 101",
                "location_type": "ruin"
            }
        ],
        "npcs": [
            {
                "name": "Amata",
                "role": "settler",
                "personality": ["loyal", "brave"],
                "location": "Vault 101"
            },
            {
                "name": "Moriarty",
                "role": "merchant",
                "personality": ["greedy", "cunning"],
                "location": "Megaton"
            }
        ],
        "events": [
            {
                "event_type": "discovery",
                "description": "Discovered Springvale ruins",
                "location": "Springvale",
                "entities": []
            },
            {
                "event_type": "dialogue",
                "description": "Negotiated with Moriarty for information",
                "location": "Megaton",
                "entities": ["Moriarty"]
            }
        ]
    }"#;

    let entities = extractor.parse_extraction(json).unwrap();
    insta::assert_json_snapshot!(entities, @r###"
    {
      "locations": [
        {
          "name": "Vault 101",
          "description": "Underground vault where you grew up",
          "location_type": "vault"
        },
        {
          "name": "Springvale",
          "description": "Abandoned town near Vault 101",
          "location_type": "ruin"
        }
      ],
      "npcs": [
        {
          "name": "Amata",
          "role": "settler",
          "personality": [
            "loyal",
            "brave"
          ],
          "location": "Vault 101"
        },
        {
          "name": "Moriarty",
          "role": "merchant",
          "personality": [
            "greedy",
            "cunning"
          ],
          "location": "Megaton"
        }
      ],
      "events": [
        {
          "event_type": "discovery",
          "description": "Discovered Springvale ruins",
          "location": "Springvale",
          "entities": []
        },
        {
          "event_type": "dialogue",
          "description": "Negotiated with Moriarty for information",
          "location": "Megaton",
          "entities": [
            "Moriarty"
          ]
        }
      ]
    }
    "###);
}

#[test]
fn snapshot_extraction_to_worldbook_conversion() {
    let extracted = ExtractedEntities {
        locations: vec![ExtractedLocation {
            name: "Rivet City".to_string(),
            description: "Aircraft carrier settlement".to_string(),
            location_type: "settlement".to_string(),
        }],
        npcs: vec![ExtractedNPC {
            name: "Doctor Li".to_string(),
            role: "scientist".to_string(),
            personality: vec!["intelligent".to_string(), "dedicated".to_string()],
            location: Some("Rivet City".to_string()),
        }],
        events: vec![ExtractedEvent {
            event_type: "npc_met".to_string(),
            description: "Met Doctor Li at Rivet City".to_string(),
            location: Some("Rivet City".to_string()),
            entities: vec!["Doctor Li".to_string()],
        }],
    };

    let (locations, npcs, events) = extracted.to_worldbook_entries();

    // Snapshot the converted worldbook entries
    insta::assert_json_snapshot!((locations, npcs, events), {
        ".**[].timestamp" => insta::dynamic_redaction(|value, _path| {
            // Redact timestamps since they'll change every test run
            assert!(value.as_str().is_some());
            "<redacted timestamp>"
        }),
    }, @r###"
    [
      [
        {
          "id": "rivet_city",
          "name": "Rivet City",
          "description": "Aircraft carrier settlement",
          "location_type": "settlement",
          "npcs_present": [],
          "atmosphere": null,
          "first_visited": null,
          "last_visited": null,
          "visit_count": 0,
          "notes": [],
          "state": {}
        }
      ],
      [
        {
          "id": "doctor_li",
          "name": "Doctor Li",
          "role": "scientist",
          "personality": [
            "intelligent",
            "dedicated"
          ],
          "current_location": "rivet_city",
          "disposition": 0,
          "knowledge": [],
          "notes": "",
          "alive": true
        }
      ],
      [
        {
          "timestamp": "<redacted timestamp>",
          "location": "rivet_city",
          "event_type": "npc_met",
          "description": "Met Doctor Li at Rivet City",
          "entities": [
            "doctor_li"
          ]
        }
      ]
    ]
    "###);
}

// Helper function to create a test location
fn create_test_location(id: &str, name: &str, desc: &str, loc_type: &str) -> Location {
    Location {
        id: id.into(),
        name: name.into(),
        name_lowercase: String::new().into(),
        description: desc.into(),
        location_type: loc_type.into(),
        npcs_present: vec![],
        atmosphere: None,
        first_visited: None,
        last_visited: None,
        visit_count: 0,
        notes: vec![],
        state: HashMap::new(),
    }
}

// Helper function to create a test NPC
fn create_test_npc(id: &str, name: &str, role: &str) -> NPC {
    NPC {
        id: id.into(),
        name: name.into(),
        name_lowercase: String::new().into(),
        role: role.into(),
        personality: vec![],
        current_location: None,
        disposition: 0,
        knowledge: vec![],
        notes: String::new().into(),
        alive: true,
    }
}

// Helper function to create a test event
fn create_test_event(location: Option<String>, event_type: &str, desc: &str) -> WorldEvent {
    WorldEvent {
        timestamp: chrono::Utc::now().to_rfc3339().into(),
        location: location.map(|s| s.into()),
        event_type: event_type.into(),
        description: desc.into(),
        entities: vec![],
    }
}

#[test]
fn test_worldbook_initialization() {
    let worldbook = Worldbook::new();

    assert_eq!(worldbook.locations.len(), 0);
    assert_eq!(worldbook.npcs.len(), 0);
    assert_eq!(worldbook.events.len(), 0);
    assert!(worldbook.current_location.is_none());
}

#[test]
fn test_add_location() {
    let mut worldbook = Worldbook::new();

    let location = create_test_location(
        "megaton_01",
        "Megaton",
        "A settlement built around an atomic bomb",
        "settlement",
    );

    worldbook.add_location(location);

    assert_eq!(worldbook.locations.len(), 1);
    assert!(worldbook.locations.contains_key("megaton_01"));
}

#[test]
fn test_add_multiple_locations() {
    let mut worldbook = Worldbook::new();

    worldbook.add_location(create_test_location(
        "loc1",
        "Location 1",
        "Desc 1",
        "settlement",
    ));
    worldbook.add_location(create_test_location("loc2", "Location 2", "Desc 2", "ruin"));
    worldbook.add_location(create_test_location(
        "loc3",
        "Location 3",
        "Desc 3",
        "vault",
    ));

    assert_eq!(worldbook.locations.len(), 3);
}

#[test]
fn test_update_existing_location() {
    let mut worldbook = Worldbook::new();

    let mut location = create_test_location("test", "Test", "Original", "settlement");
    worldbook.add_location(location.clone());

    // Update the location
    location.description = "Updated description".into();
    worldbook.add_location(location);

    assert_eq!(
        worldbook.locations.len(),
        1,
        "Should still have only 1 location"
    );
    let stored = worldbook.locations.get("test").unwrap();
    assert_eq!(stored.description, "Updated description");
}

#[test]
fn test_add_npc() {
    let mut worldbook = Worldbook::new();

    let npc = create_test_npc("lucas_01", "Lucas Simms", "sheriff");

    worldbook.add_npc(npc);

    assert_eq!(worldbook.npcs.len(), 1);
    assert!(worldbook.npcs.contains_key("lucas_01"));
}

#[test]
fn test_add_multiple_npcs() {
    let mut worldbook = Worldbook::new();

    worldbook.add_npc(create_test_npc("npc1", "NPC 1", "merchant"));
    worldbook.add_npc(create_test_npc("npc2", "NPC 2", "guard"));
    worldbook.add_npc(create_test_npc("npc3", "NPC 3", "settler"));

    assert_eq!(worldbook.npcs.len(), 3);
}

#[test]
fn test_npc_disposition() {
    let mut npc = create_test_npc("test", "Test NPC", "merchant");

    assert_eq!(npc.disposition, 0, "Should start neutral");

    npc.disposition = 50;
    assert_eq!(npc.disposition, 50, "Should be friendly");

    npc.disposition = -50;
    assert_eq!(npc.disposition, -50, "Should be hostile");
}

#[test]
fn test_npc_alive_status() {
    let mut npc = create_test_npc("test", "Test NPC", "raider");

    assert!(npc.alive, "Should start alive");

    npc.alive = false;
    assert!(!npc.alive, "Should be dead");
}

#[test]
fn test_npc_knowledge() {
    let mut npc = create_test_npc("test", "Test NPC", "merchant");

    assert_eq!(npc.knowledge.len(), 0);

    npc.knowledge.push("location_of_vault".into());
    npc.knowledge.push("password_to_armory".into());

    assert_eq!(npc.knowledge.len(), 2);
    assert!(npc.knowledge.contains(&"location_of_vault".into()));
}

#[test]
fn test_npc_personality_traits() {
    let mut npc = create_test_npc("test", "Test NPC", "settler");

    npc.personality.push("gruff".into());
    npc.personality.push("honest".into());
    npc.personality.push("paranoid".into());

    assert_eq!(npc.personality.len(), 3);
}

#[test]
fn test_record_event() {
    let mut worldbook = Worldbook::new();

    worldbook.add_event(create_test_event(
        Some("megaton_01".into()),
        "npc_met",
        "Met Sheriff Lucas Simms",
    ));

    assert_eq!(worldbook.events.len(), 1);

    let event = &worldbook.events[0];
    assert_eq!(event.location, Some("megaton_01".into()));
    assert_eq!(event.event_type, "npc_met");
    assert_eq!(event.description, "Met Sheriff Lucas Simms");
}

#[test]
fn test_multiple_events() {
    let mut worldbook = Worldbook::new();

    worldbook.add_event(create_test_event(
        Some("loc1".into()),
        "discovery",
        "Found a vault",
    ));
    worldbook.add_event(create_test_event(
        Some("loc2".into()),
        "combat",
        "Fought raiders",
    ));
    worldbook.add_event(create_test_event(
        None,
        "dialogue",
        "Talked to mysterious stranger",
    ));

    assert_eq!(worldbook.events.len(), 3);
}

#[test]
fn test_event_timeline_order() {
    let mut worldbook = Worldbook::new();

    worldbook.add_event(create_test_event(None, "event1", "First event"));
    std::thread::sleep(std::time::Duration::from_millis(10));
    worldbook.add_event(create_test_event(None, "event2", "Second event"));
    std::thread::sleep(std::time::Duration::from_millis(10));
    worldbook.add_event(create_test_event(None, "event3", "Third event"));

    assert_eq!(worldbook.events.len(), 3);

    // Events should be in chronological order
    let timestamps: Vec<_> = worldbook.events.iter().map(|e| &e.timestamp).collect();
    for i in 1..timestamps.len() {
        assert!(
            timestamps[i] >= timestamps[i - 1],
            "Events should be in chronological order"
        );
    }
}

#[test]
fn test_location_visit_tracking() {
    let mut worldbook = Worldbook::new();
    let location = create_test_location("test", "Test", "Description", "settlement");
    worldbook.add_location(location);

    let loc = worldbook.get_location("test").unwrap();
    assert_eq!(loc.visit_count, 0);
    assert!(loc.first_visited.is_none());
    assert!(loc.last_visited.is_none());

    worldbook.visit_location("test");

    let loc = worldbook.get_location("test").unwrap();
    assert_eq!(loc.visit_count, 1);
    assert!(loc.first_visited.is_some());
    assert!(loc.last_visited.is_some());

    let first_visit = loc.first_visited.clone();

    std::thread::sleep(std::time::Duration::from_millis(10));
    worldbook.visit_location("test");

    let loc = worldbook.get_location("test").unwrap();
    assert_eq!(loc.visit_count, 2);
    assert_eq!(
        loc.first_visited, first_visit,
        "First visit should not change"
    );
    assert!(
        loc.last_visited > first_visit,
        "Last visit should be updated"
    );
}

#[test]
fn test_location_types() {
    let settlement = create_test_location("s1", "Settlement", "Desc", "settlement");
    let ruin = create_test_location("r1", "Ruin", "Desc", "ruin");
    let vault = create_test_location("v1", "Vault", "Desc", "vault");
    let wasteland = create_test_location("w1", "Wasteland", "Desc", "wasteland");

    assert_eq!(settlement.location_type, "settlement");
    assert_eq!(ruin.location_type, "ruin");
    assert_eq!(vault.location_type, "vault");
    assert_eq!(wasteland.location_type, "wasteland");
}

#[test]
fn test_location_npcs_present() {
    let mut location = create_test_location("test", "Test", "Desc", "settlement");

    assert_eq!(location.npcs_present.len(), 0);

    location.npcs_present.push("npc1".into());
    location.npcs_present.push("npc2".into());

    assert_eq!(location.npcs_present.len(), 2);
}

#[test]
fn test_location_notes() {
    let mut location = create_test_location("test", "Test", "Desc", "settlement");

    location.notes.push("Found a secret stash here".into());
    location.notes.push("Guard mentioned a password".into());

    assert_eq!(location.notes.len(), 2);
}

#[test]
fn test_location_state() {
    let mut location = create_test_location("test", "Test", "Desc", "settlement");

    location
        .state
        .insert("quest_completed".into(), "true".into());
    location
        .state
        .insert("door_unlocked".into(), "false".into());

    assert_eq!(location.state.len(), 2);
    assert_eq!(location.state.get("quest_completed"), Some(&"true".into()));
}

#[test]
fn test_get_location() {
    let mut worldbook = Worldbook::new();

    worldbook.add_location(create_test_location(
        "test",
        "Test Location",
        "Desc",
        "settlement",
    ));

    let location = worldbook.get_location("test");
    assert!(location.is_some());
    assert_eq!(location.unwrap().name, "Test Location");

    let missing = worldbook.get_location("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn test_get_npc() {
    let mut worldbook = Worldbook::new();

    worldbook.add_npc(create_test_npc("test", "Test NPC", "merchant"));

    let npc = worldbook.get_npc("test");
    assert!(npc.is_some());
    assert_eq!(npc.unwrap().name, "Test NPC");

    let missing = worldbook.get_npc("nonexistent");
    assert!(missing.is_none());
}

// Note: find_location_by_name and find_npc_by_name methods don't exist yet
// These tests are commented out until those methods are implemented
/*
#[test]
fn test_find_location_by_name() {
    let mut worldbook = Worldbook::new();

    worldbook.add_location(create_test_location("meg_01", "Megaton", "Desc", "settlement"));
    worldbook.add_location(create_test_location("riv_01", "Rivet City", "Desc", "settlement"));

    let found = worldbook.find_location_by_name("megaton");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, "meg_01");

    let not_found = worldbook.find_location_by_name("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_find_npc_by_name() {
    let mut worldbook = Worldbook::new();

    worldbook.add_npc(create_test_npc("lucas_01", "Lucas Simms", "sheriff"));
    worldbook.add_npc(create_test_npc("moira_01", "Moira Brown", "merchant"));

    let found = worldbook.find_npc_by_name("lucas simms");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, "lucas_01");

    let not_found = worldbook.find_npc_by_name("nonexistent");
    assert!(not_found.is_none());
}
*/

#[test]
fn test_worldbook_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_worldbook.json");

    let mut worldbook = Worldbook::new();
    worldbook.add_location(create_test_location("test", "Test", "Desc", "settlement"));
    worldbook.add_npc(create_test_npc("npc1", "NPC", "merchant"));
    worldbook.add_event(create_test_event(
        Some("test".into()),
        "discovery",
        "Found location",
    ));

    // Save
    let save_result = worldbook.save_to_file(&file_path);
    assert!(save_result.is_ok(), "Should save successfully");

    // Load
    let loaded_result = Worldbook::load_from_file(&file_path);
    assert!(loaded_result.is_ok(), "Should load successfully");

    let loaded = loaded_result.unwrap();
    assert_eq!(loaded.locations.len(), 1);
    assert_eq!(loaded.npcs.len(), 1);
    assert_eq!(loaded.events.len(), 1);
}

#[test]
fn test_worldbook_load_nonexistent_file() {
    let result = Worldbook::load_from_file(Path::new("nonexistent.json"));
    // Should either return error or empty worldbook depending on implementation
    // Check that it doesn't panic
    let _ = result;
}

#[test]
fn test_current_location_tracking() {
    let mut worldbook = Worldbook::new();

    assert!(worldbook.current_location.is_none());

    worldbook.current_location = Some("megaton_01".into());
    assert_eq!(worldbook.current_location, Some("megaton_01".into()));

    worldbook.current_location = None;
    assert!(worldbook.current_location.is_none());
}

#[test]
fn test_npc_current_location() {
    let mut npc = create_test_npc("test", "Test", "merchant");

    assert!(npc.current_location.is_none());

    npc.current_location = Some("megaton_01".into());
    assert_eq!(npc.current_location, Some("megaton_01".into()));
}

#[test]
fn test_location_atmosphere() {
    let mut location = create_test_location("test", "Test", "Desc", "settlement");

    assert!(location.atmosphere.is_none());

    location.atmosphere = Some("Tense and hostile".into());
    assert_eq!(location.atmosphere, Some("Tense and hostile".into()));
}

#[test]
fn test_event_entities() {
    let mut worldbook = Worldbook::new();

    let event = WorldEvent {
        timestamp: chrono::Utc::now().to_rfc3339().into(),
        location: Some("loc1".into()),
        event_type: "combat".into(),
        description: "Battle with raiders".into(),
        entities: vec!["raider_01".into(), "raider_02".into()],
    };

    worldbook.add_event(event);

    assert_eq!(worldbook.events.len(), 1);
    assert_eq!(worldbook.events[0].entities.len(), 2);
}

#[test]
fn test_complex_worldbook_scenario() {
    let mut worldbook = Worldbook::new();

    // Add location
    let megaton = create_test_location(
        "megaton_01",
        "Megaton",
        "A settlement built around an atomic bomb",
        "settlement",
    );
    worldbook.add_location(megaton);
    worldbook.visit_location("megaton_01");

    // Add NPCs
    let mut lucas = create_test_npc("lucas_01", "Lucas Simms", "sheriff");
    lucas.current_location = Some("megaton_01".into());
    lucas.disposition = 25;
    worldbook.add_npc(lucas);

    let mut moira = create_test_npc("moira_01", "Moira Brown", "merchant");
    moira.current_location = Some("megaton_01".into());
    moira.disposition = 50;
    worldbook.add_npc(moira);

    // Record events
    worldbook.add_event(create_test_event(
        Some("megaton_01".into()),
        "arrival",
        "Entered Megaton for the first time",
    ));
    worldbook.add_event(create_test_event(
        Some("megaton_01".into()),
        "npc_met",
        "Met Sheriff Lucas Simms",
    ));
    worldbook.add_event(create_test_event(
        Some("megaton_01".into()),
        "dialogue",
        "Talked to Moira about the wasteland",
    ));

    // Verify state
    assert_eq!(worldbook.locations.len(), 1);
    assert_eq!(worldbook.npcs.len(), 2);
    assert_eq!(worldbook.events.len(), 3);

    let location = worldbook.get_location("megaton_01").unwrap();
    assert_eq!(location.visit_count, 1);
}
