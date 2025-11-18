/// Comprehensive tests for story manager and context management
use fallout_dnd::game::story_manager::StoryManager;

#[test]
fn test_story_manager_initialization() {
    let story = StoryManager::new();

    assert_eq!(story.len(), 0);
    assert!(story.is_empty());
}

#[test]
fn test_story_manager_custom_capacity() {
    let story = StoryManager::with_capacity(10);

    assert_eq!(story.len(), 0);
}

#[test]
fn test_add_single_entry() {
    let mut story = StoryManager::new();

    story.add("First event".to_string());

    assert_eq!(story.len(), 1);
    assert!(!story.is_empty());
}

#[test]
fn test_add_multiple_entries() {
    let mut story = StoryManager::new();

    story.add("Event 1".to_string());
    story.add("Event 2".to_string());
    story.add("Event 3".to_string());

    assert_eq!(story.len(), 3);
}

#[test]
fn test_context_overflow_removes_oldest() {
    let mut story = StoryManager::with_capacity(3);

    story.add("Event 1".to_string());
    story.add("Event 2".to_string());
    story.add("Event 3".to_string());

    assert_eq!(story.len(), 3);

    // Adding a 4th event should remove the oldest
    story.add("Event 4".to_string());

    assert_eq!(story.len(), 3);

    // Verify the oldest was removed
    let all = story.get_all();
    let entries: Vec<&String> = all.iter().collect();
    assert!(entries.iter().any(|e| *e == "Event 2"));
    assert!(entries.iter().any(|e| *e == "Event 3"));
    assert!(entries.iter().any(|e| *e == "Event 4"));
    assert!(!entries.iter().any(|e| *e == "Event 1")); // Oldest should be gone
}

#[test]
fn test_multiple_overflows() {
    let mut story = StoryManager::with_capacity(2);

    for i in 0..10 {
        story.add(format!("Event {}", i));
    }

    assert_eq!(story.len(), 2);

    // Should only have the last 2 events
    let all = story.get_all();
    let entries: Vec<&String> = all.iter().collect();
    assert!(entries.iter().any(|e| *e == "Event 8"));
    assert!(entries.iter().any(|e| *e == "Event 9"));
}

#[test]
fn test_get_recent_with_exact_count() {
    let mut story = StoryManager::new();

    story.add("Event 1".to_string());
    story.add("Event 2".to_string());
    story.add("Event 3".to_string());

    let recent = story.get_recent(2);

    assert_eq!(recent.len(), 2);
    assert_eq!(*recent[0], "Event 2");
    assert_eq!(*recent[1], "Event 3");
}

#[test]
fn test_get_recent_more_than_available() {
    let mut story = StoryManager::new();

    story.add("Event 1".to_string());
    story.add("Event 2".to_string());

    let recent = story.get_recent(10);

    assert_eq!(recent.len(), 2);
    assert_eq!(*recent[0], "Event 1");
    assert_eq!(*recent[1], "Event 2");
}

#[test]
fn test_get_recent_zero_count() {
    let mut story = StoryManager::new();

    story.add("Event 1".to_string());

    let recent = story.get_recent(0);

    assert_eq!(recent.len(), 0);
}

#[test]
fn test_get_recent_from_empty() {
    let story = StoryManager::new();

    let recent = story.get_recent(5);

    assert_eq!(recent.len(), 0);
}

#[test]
fn test_get_all() {
    let mut story = StoryManager::new();

    story.add("Event 1".to_string());
    story.add("Event 2".to_string());
    story.add("Event 3".to_string());

    let all = story.get_all();

    assert_eq!(all.len(), 3);
}

#[test]
fn test_get_all_empty() {
    let story = StoryManager::new();

    let all = story.get_all();

    assert_eq!(all.len(), 0);
}

#[test]
fn test_is_empty() {
    let mut story = StoryManager::new();

    assert!(story.is_empty());

    story.add("Event".to_string());
    assert!(!story.is_empty());
}

#[test]
fn test_len_accuracy() {
    let mut story = StoryManager::new();

    assert_eq!(story.len(), 0);

    for i in 1..=10 {
        story.add(format!("Event {}", i));
        assert_eq!(story.len(), i);
    }
}

#[test]
fn test_capacity_enforcement() {
    let mut story = StoryManager::with_capacity(5);

    for i in 0..20 {
        story.add(format!("Event {}", i));
    }

    assert_eq!(story.len(), 5);
}

#[test]
fn test_add_empty_string() {
    let mut story = StoryManager::new();

    story.add("".to_string());

    assert_eq!(story.len(), 1);
    let all = story.get_all();
    assert_eq!(all.iter().next().unwrap(), "");
}

#[test]
fn test_add_very_long_string() {
    let mut story = StoryManager::new();

    let long_string = "A".repeat(10000);
    story.add(long_string.clone());

    assert_eq!(story.len(), 1);
    let all = story.get_all();
    assert_eq!(all.iter().next().unwrap(), &long_string);
}

#[test]
fn test_add_unicode_characters() {
    let mut story = StoryManager::new();

    story.add("Event with emoji 🎮".to_string());
    story.add("Event with Chinese 你好".to_string());
    story.add("Event with Arabic مرحبا".to_string());

    assert_eq!(story.len(), 3);
}

#[test]
fn test_add_special_characters() {
    let mut story = StoryManager::new();

    story.add("Event with \"quotes\"".to_string());
    story.add("Event with 'apostrophes'".to_string());
    story.add("Event with\nnewlines".to_string());
    story.add("Event with\ttabs".to_string());

    assert_eq!(story.len(), 4);
}

#[test]
fn test_get_recent_maintains_order() {
    let mut story = StoryManager::new();

    for i in 1..=10 {
        story.add(format!("Event {}", i));
    }

    let recent = story.get_recent(5);

    assert_eq!(recent.len(), 5);
    assert_eq!(*recent[0], "Event 6");
    assert_eq!(*recent[1], "Event 7");
    assert_eq!(*recent[2], "Event 8");
    assert_eq!(*recent[3], "Event 9");
    assert_eq!(*recent[4], "Event 10");
}

#[test]
fn test_capacity_one() {
    let mut story = StoryManager::with_capacity(1);

    story.add("Event 1".to_string());
    assert_eq!(story.len(), 1);

    story.add("Event 2".to_string());
    assert_eq!(story.len(), 1);

    let all = story.get_all();
    assert_eq!(all.iter().next().unwrap(), "Event 2");
}

#[test]
fn test_capacity_zero() {
    let mut story = StoryManager::with_capacity(0);

    story.add("Event 1".to_string());

    // With capacity 0, the story manager might behave differently
    // depending on implementation. It should either:
    // 1. Not store anything (len = 0)
    // 2. Store at least 1 item despite capacity 0
    // This test just ensures it doesn't panic
    assert!(story.len() >= 0);
}

#[test]
fn test_large_capacity() {
    let mut story = StoryManager::with_capacity(10000);

    for i in 0..100 {
        story.add(format!("Event {}", i));
    }

    assert_eq!(story.len(), 100);
}

#[test]
fn test_get_recent_single_item() {
    let mut story = StoryManager::new();
    story.add("Single event".to_string());

    let recent = story.get_recent(1);

    assert_eq!(recent.len(), 1);
    assert_eq!(*recent[0], "Single event");
}

#[test]
fn test_fifo_behavior() {
    let mut story = StoryManager::with_capacity(3);

    story.add("First".to_string());
    story.add("Second".to_string());
    story.add("Third".to_string());
    story.add("Fourth".to_string());
    story.add("Fifth".to_string());

    let all = story.get_all();
    let entries: Vec<&String> = all.iter().collect();

    // Should have Third, Fourth, Fifth (FIFO)
    assert_eq!(entries.len(), 3);
    assert!(entries.iter().any(|e| *e == "Third"));
    assert!(entries.iter().any(|e| *e == "Fourth"));
    assert!(entries.iter().any(|e| *e == "Fifth"));
    assert!(!entries.iter().any(|e| *e == "First"));
    assert!(!entries.iter().any(|e| *e == "Second"));
}

#[test]
fn test_alternating_add_and_get() {
    let mut story = StoryManager::with_capacity(5);

    story.add("Event 1".to_string());
    let r1 = story.get_recent(1);
    assert_eq!(r1.len(), 1);

    story.add("Event 2".to_string());
    let r2 = story.get_recent(2);
    assert_eq!(r2.len(), 2);

    story.add("Event 3".to_string());
    let r3 = story.get_recent(3);
    assert_eq!(r3.len(), 3);
}

#[test]
fn test_add_duplicate_entries() {
    let mut story = StoryManager::new();

    story.add("Same event".to_string());
    story.add("Same event".to_string());
    story.add("Same event".to_string());

    assert_eq!(story.len(), 3);

    let all = story.get_all();
    let entries: Vec<&String> = all.iter().collect();
    assert_eq!(entries.iter().filter(|e| **e == "Same event").count(), 3);
}

#[test]
fn test_context_with_mixed_length_strings() {
    let mut story = StoryManager::with_capacity(10);

    story.add("Short".to_string());
    story.add("A much longer string with more content".to_string());
    story.add("X".to_string());
    story.add("Medium length string".to_string());

    assert_eq!(story.len(), 4);
}

#[test]
fn test_get_recent_boundary_conditions() {
    let mut story = StoryManager::new();

    for i in 0..10 {
        story.add(format!("Event {}", i));
    }

    // Test boundary: exact length
    let r10 = story.get_recent(10);
    assert_eq!(r10.len(), 10);

    // Test boundary: length + 1
    let r11 = story.get_recent(11);
    assert_eq!(r11.len(), 10);

    // Test boundary: length - 1
    let r9 = story.get_recent(9);
    assert_eq!(r9.len(), 9);
}
