mod ai;
mod config;
mod error;
mod game;
mod tui;
mod ui;

use ai::extractor::ExtractionAI;
use ai::AIDungeonMaster;
use config::Config;
use game::handlers::{create_new_character, game_loop, load_game};
use game::tui_game_loop::run_game_with_tui;
use ui::UI;

#[tokio::main]
async fn main() {
    UI::clear_screen();
    UI::print_header();

    // Load config
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(_) => {
            UI::print_info("No config.toml found, using defaults.");
            Config::default()
        }
    };

    // Initialize AI DM
    let ai_dm = AIDungeonMaster::new(config.llama.clone());

    // Test llama.cpp connection
    UI::print_info("Testing connection to llama.cpp server...");
    match ai_dm.test_connection().await {
        Ok(_) => UI::print_success(&format!(
            "Connected to narrative AI at {}",
            config.llama.server_url
        )),
        Err(e) => {
            UI::print_error(&format!("{}", e));
            UI::print_info(
                "You can continue without AI (manual mode), or fix the connection and restart.",
            );
            UI::print_info("To start llama.cpp server: ./llama-server -m <model_path> --port 8080");
        }
    }

    // Test extraction AI connection
    UI::print_info("Testing connection to extraction AI server...");
    let extractor = ExtractionAI::new(config.llama.extraction_url.clone());
    match extractor.test_connection().await {
        Ok(_) => UI::print_success(&format!(
            "Connected to extraction AI at {}",
            config.llama.extraction_url
        )),
        Err(e) => {
            UI::print_error(&format!("{}", e));
            UI::print_info("Worldbook features will be limited without extraction AI.");
            UI::print_info(
                "To start extraction server: ./llama-server -m <model_path> --port 8081",
            );
        }
    }

    println!();

    // Main menu loop
    loop {
        use colored::*;
        println!("{}", "MAIN MENU".bold());
        println!("  1. New Game (TUI Mode) - Recommended!");
        println!("  2. New Game (Classic Mode)");
        println!("  3. Load Game (TUI Mode)");
        println!("  4. Load Game (Classic Mode)");
        println!("  5. Exit");
        println!();

        let choice = UI::prompt(">").to_lowercase();

        match choice.as_str() {
            "1" | "new" | "new game" => {
                let game_state = create_new_character(&config);
                if let Err(e) = run_game_with_tui(game_state, &ai_dm, config.clone()).await {
                    UI::print_error(&format!("TUI error: {}", e));
                }
                UI::clear_screen();
                UI::print_header();
            }
            "2" | "classic" => {
                let game_state = create_new_character(&config);
                game_loop(game_state, &ai_dm, config.clone()).await;
            }
            "3" | "load" => {
                if let Some(game_state) = load_game() {
                    if let Err(e) = run_game_with_tui(game_state, &ai_dm, config.clone()).await {
                        UI::print_error(&format!("TUI error: {}", e));
                    }
                    UI::clear_screen();
                    UI::print_header();
                }
            }
            "4" | "load classic" => {
                if let Some(game_state) = load_game() {
                    game_loop(game_state, &ai_dm, config.clone()).await;
                }
            }
            "5" | "exit" | "quit" => {
                println!("Thanks for playing!");
                break;
            }
            _ => UI::print_error("Invalid choice"),
        }
    }
}
