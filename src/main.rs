mod ai;
mod config;
mod error;
mod game;
mod templates;
mod tui;
mod ui;
mod validation;

use ai::extractor::ExtractionAI;
use ai::server_manager::{ServerConfig, ServerManager};
use ai::AIDungeonMaster;
use config::Config;
use game::handlers::{create_new_character, game_loop, load_game};
use game::tui_game_loop::run_game_with_tui;
use std::path::PathBuf;
use ui::UI;

// Use mimalloc as the global allocator for improved performance
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    // Initialize logging
    init_logging();

    tracing::info!("Starting Fallout D&D game");

    UI::clear_screen();
    UI::print_header();

    // Load config
    let config = match Config::load_with_env() {
        Ok(cfg) => cfg,
        Err(_) => {
            UI::print_info("No config.toml found, using defaults.");
            Config::default()
        }
    };

    // Initialize server manager and auto-start servers if configured
    let server_manager = if config.llama.auto_start {
        UI::print_info("Auto-start enabled. Checking AI servers...");

        let narrative_config = if let (Some(exe_path), Some(model_path)) = (
            &config.llama.llama_server_path,
            &config.llama.narrative_model_path,
        ) {
            Some(ServerConfig {
                executable: PathBuf::from(exe_path),
                model_path: PathBuf::from(model_path),
                port: 8080,
                ctx_size: config.llama.narrative_ctx_size as usize,
                threads: config.llama.narrative_threads as usize,
                url: config.llama.server_url.clone(),
                name: "Narrative AI".to_string(),
            })
        } else {
            None
        };

        let extraction_config = if let (Some(exe_path), Some(model_path)) = (
            &config.llama.llama_server_path,
            &config.llama.extraction_model_path,
        ) {
            Some(ServerConfig {
                executable: PathBuf::from(exe_path),
                model_path: PathBuf::from(model_path),
                port: 8081,
                ctx_size: config.llama.extraction_ctx_size as usize,
                threads: config.llama.extraction_threads as usize,
                url: config.llama.extraction_url.clone(),
                name: "Extraction AI".to_string(),
            })
        } else {
            None
        };

        let mut manager = ServerManager::new(narrative_config, extraction_config);

        match manager.ensure_servers_running().await {
            Ok(_) => UI::print_success("AI servers are ready!"),
            Err(e) => {
                UI::print_error(&format!("Failed to start AI servers: {}", e));
                UI::print_info("You can continue without AI (manual mode).");
                UI::print_info(
                    "Check that model files exist and paths in config.toml are correct.",
                );
            }
        }

        Some(manager)
    } else {
        None
    };

    // Initialize AI DM
    let ai_dm = AIDungeonMaster::new(config.llama.clone());

    // Test llama.cpp connection
    UI::print_info("Verifying connection to narrative AI server...");
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
            if !config.llama.auto_start {
                UI::print_info(
                    "To start llama.cpp server: ./llama-server -m <model_path> --port 8080",
                );
                UI::print_info("Or set auto_start = true in config.toml");
            }
        }
    }

    // Test extraction AI connection
    UI::print_info("Verifying connection to extraction AI server...");
    let extractor = ExtractionAI::new(config.llama.extraction_url.clone());
    match extractor.test_connection().await {
        Ok(_) => UI::print_success(&format!(
            "Connected to extraction AI at {}",
            config.llama.extraction_url
        )),
        Err(e) => {
            UI::print_error(&format!("{}", e));
            UI::print_info("Worldbook features will be limited without extraction AI.");
            if !config.llama.auto_start {
                UI::print_info(
                    "To start extraction server: ./llama-server -m <model_path> --port 8081",
                );
                UI::print_info("Or set auto_start = true in config.toml");
            }
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
                if let Err(e) =
                    run_game_with_tui(game_state, &ai_dm, &extractor, config.clone()).await
                {
                    UI::print_error(&format!("TUI error: {}", e));
                }
                // Clean transition back to main menu
                println!(); // Add spacing
                UI::wait_for_enter(); // Give user control
                UI::clear_screen();
                UI::print_header();
            }
            "2" | "classic" => {
                let game_state = create_new_character(&config);
                game_loop(game_state, &ai_dm, config.clone()).await;
            }
            "3" | "load" => {
                if let Some(game_state) = load_game() {
                    if let Err(e) =
                        run_game_with_tui(game_state, &ai_dm, &extractor, config.clone()).await
                    {
                        UI::print_error(&format!("TUI error: {}", e));
                    }
                    // Clean transition back to main menu
                    println!(); // Add spacing
                    UI::wait_for_enter(); // Give user control
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

    // Clean up server manager (stops servers)
    if let Some(mut manager) = server_manager {
        UI::print_info("Shutting down AI servers...");
        manager.stop_servers();
    }

    tracing::info!("Game exited normally");
}

/// Initialize tracing subscriber for logging
fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    // Default to info level, but allow override via RUST_LOG env var
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("fallout_dnd=info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false) // Don't show target module in logs
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}
