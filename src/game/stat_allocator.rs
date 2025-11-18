use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use super::character::Special;

const TOTAL_POINTS: u8 = 40;
const MIN_STAT: u8 = 1;
const MAX_STAT: u8 = 10;
const STATS_COUNT: usize = 7;

#[derive(Debug, Clone, Copy)]
enum SelectedStat {
    Strength = 0,
    Perception = 1,
    Endurance = 2,
    Charisma = 3,
    Intelligence = 4,
    Agility = 5,
    Luck = 6,
}

impl SelectedStat {
    fn next(&self) -> Self {
        match self {
            SelectedStat::Strength => SelectedStat::Perception,
            SelectedStat::Perception => SelectedStat::Endurance,
            SelectedStat::Endurance => SelectedStat::Charisma,
            SelectedStat::Charisma => SelectedStat::Intelligence,
            SelectedStat::Intelligence => SelectedStat::Agility,
            SelectedStat::Agility => SelectedStat::Luck,
            SelectedStat::Luck => SelectedStat::Strength,
        }
    }

    fn prev(&self) -> Self {
        match self {
            SelectedStat::Strength => SelectedStat::Luck,
            SelectedStat::Perception => SelectedStat::Strength,
            SelectedStat::Endurance => SelectedStat::Perception,
            SelectedStat::Charisma => SelectedStat::Endurance,
            SelectedStat::Intelligence => SelectedStat::Charisma,
            SelectedStat::Agility => SelectedStat::Intelligence,
            SelectedStat::Luck => SelectedStat::Agility,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            SelectedStat::Strength => "Strength",
            SelectedStat::Perception => "Perception",
            SelectedStat::Endurance => "Endurance",
            SelectedStat::Charisma => "Charisma",
            SelectedStat::Intelligence => "Intelligence",
            SelectedStat::Agility => "Agility",
            SelectedStat::Luck => "Luck",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            SelectedStat::Strength => "Physical power, melee damage",
            SelectedStat::Perception => "Awareness, accuracy",
            SelectedStat::Endurance => "Health, radiation resistance",
            SelectedStat::Charisma => "Speech, barter",
            SelectedStat::Intelligence => "Skill points, hacking",
            SelectedStat::Agility => "Action points, speed",
            SelectedStat::Luck => "Critical chance, general fortune",
        }
    }
}

pub fn allocate_stats_interactive() -> io::Result<Special> {
    let mut stats = [MIN_STAT; STATS_COUNT];
    let mut selected = SelectedStat::Strength;
    let mut points_spent = STATS_COUNT as u8 * MIN_STAT;

    // Enter raw mode to capture arrow keys
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    // Clear screen and hide cursor
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Hide
    )?;

    let result = run_allocator_loop(&mut stdout, &mut stats, &mut selected, &mut points_spent);

    // Cleanup: show cursor and disable raw mode
    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result?;

    // Convert stats array to Special struct
    Ok(Special {
        strength: stats[SelectedStat::Strength as usize],
        perception: stats[SelectedStat::Perception as usize],
        endurance: stats[SelectedStat::Endurance as usize],
        charisma: stats[SelectedStat::Charisma as usize],
        intelligence: stats[SelectedStat::Intelligence as usize],
        agility: stats[SelectedStat::Agility as usize],
        luck: stats[SelectedStat::Luck as usize],
    })
}

fn run_allocator_loop(
    stdout: &mut io::Stdout,
    stats: &mut [u8; STATS_COUNT],
    selected: &mut SelectedStat,
    points_spent: &mut u8,
) -> io::Result<()> {
    loop {
        render_ui(stdout, stats, *selected, *points_spent)?;

        if let Event::Key(key_event) = event::read()? {
            // Only process key press events, ignore release and repeat to prevent double-triggering
            if key_event.kind == event::KeyEventKind::Press {
                match handle_key_event(key_event, stats, selected, points_spent) {
                    KeyAction::Continue => continue,
                    KeyAction::Finish => break,
                    KeyAction::Quit => {
                        return Err(io::Error::new(
                            io::ErrorKind::Interrupted,
                            "User cancelled stat allocation",
                        ))
                    }
                }
            }
        }
    }

    Ok(())
}

enum KeyAction {
    Continue,
    Finish,
    Quit,
}

fn handle_key_event(
    key_event: KeyEvent,
    stats: &mut [u8; STATS_COUNT],
    selected: &mut SelectedStat,
    points_spent: &mut u8,
) -> KeyAction {
    match key_event.code {
        KeyCode::Up => {
            *selected = selected.prev();
            KeyAction::Continue
        }
        KeyCode::Down => {
            *selected = selected.next();
            KeyAction::Continue
        }
        KeyCode::Left => {
            decrease_stat(stats, *selected, points_spent);
            KeyAction::Continue
        }
        KeyCode::Right => {
            increase_stat(stats, *selected, points_spent);
            KeyAction::Continue
        }
        KeyCode::Enter => {
            let points_remaining = TOTAL_POINTS.saturating_sub(*points_spent);
            if points_remaining == 0 {
                KeyAction::Finish
            } else {
                KeyAction::Continue
            }
        }
        KeyCode::Char('q') | KeyCode::Esc
            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            KeyAction::Quit
        }
        _ => KeyAction::Continue,
    }
}

fn increase_stat(stats: &mut [u8; STATS_COUNT], selected: SelectedStat, points_spent: &mut u8) {
    let idx = selected as usize;
    let current = stats[idx];

    if current < MAX_STAT && *points_spent < TOTAL_POINTS {
        stats[idx] += 1;
        *points_spent += 1;
    }
}

fn decrease_stat(stats: &mut [u8; STATS_COUNT], selected: SelectedStat, points_spent: &mut u8) {
    let idx = selected as usize;
    let current = stats[idx];

    if current > MIN_STAT {
        stats[idx] -= 1;
        *points_spent -= 1;
    }
}

fn render_ui(
    stdout: &mut io::Stdout,
    stats: &[u8; STATS_COUNT],
    selected: SelectedStat,
    points_spent: u8,
) -> io::Result<()> {
    let points_remaining = TOTAL_POINTS.saturating_sub(points_spent);

    queue!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::All),
    )?;

    // Title
    queue!(
        stdout,
        cursor::MoveTo(2, 1),
        SetForegroundColor(Color::Cyan),
        Print("╔════════════════════════════════════════════════════════════════╗"),
        cursor::MoveTo(2, 2),
        Print("║"),
        cursor::MoveTo(25, 2),
        SetForegroundColor(Color::Yellow),
        Print("✦ SPECIAL ALLOCATION ✦"),
        cursor::MoveTo(67, 2),
        SetForegroundColor(Color::Cyan),
        Print("║"),
        cursor::MoveTo(2, 3),
        Print("╚════════════════════════════════════════════════════════════════╝"),
    )?;

    // Points remaining
    let points_color = if points_remaining == 0 {
        Color::Green
    } else if points_remaining <= 5 {
        Color::Yellow
    } else {
        Color::White
    };

    queue!(
        stdout,
        cursor::MoveTo(2, 5),
        SetForegroundColor(Color::White),
        Print("Points Remaining: "),
        SetForegroundColor(points_color),
        Print(format!("{:2}", points_remaining)),
        SetForegroundColor(Color::DarkGrey),
        Print(format!(" / {}", TOTAL_POINTS)),
    )?;

    // Instructions
    queue!(
        stdout,
        cursor::MoveTo(2, 6),
        SetForegroundColor(Color::DarkGrey),
        Print("↑↓ Navigate  │  ←→ Adjust  │  Enter to Continue"),
    )?;

    // Render each stat
    let start_row = 8;
    for i in 0..STATS_COUNT {
        let stat_selected = SelectedStat::from_index(i);
        let is_selected = i == selected as usize;
        let stat_value = stats[i];

        render_stat_line(
            stdout,
            start_row + (i as u16 * 2),
            stat_selected,
            stat_value,
            is_selected,
            points_remaining,
        )?;
    }

    // Footer message
    let footer_row = start_row + (STATS_COUNT as u16 * 2) + 2;
    if points_remaining > 0 {
        queue!(
            stdout,
            cursor::MoveTo(2, footer_row),
            SetForegroundColor(Color::Yellow),
            Print("⚠  You must allocate all points before continuing!"),
        )?;
    } else {
        queue!(
            stdout,
            cursor::MoveTo(2, footer_row),
            SetForegroundColor(Color::Green),
            Print("✓  All points allocated! Press Enter to continue."),
        )?;
    }

    stdout.flush()?;
    Ok(())
}

fn render_stat_line(
    stdout: &mut io::Stdout,
    row: u16,
    stat: SelectedStat,
    value: u8,
    is_selected: bool,
    points_remaining: u8,
) -> io::Result<()> {
    let name = stat.name();
    let description = stat.description();

    if is_selected {
        // Render selected stat with highlight
        queue!(
            stdout,
            cursor::MoveTo(2, row),
            SetForegroundColor(Color::Black),
            SetForegroundColor(Color::Green),
            Print("►"),
            SetForegroundColor(Color::Cyan),
            Print(format!(" {:12} ", name)),
        )?;

        // Render bar
        render_stat_bar(stdout, value, true)?;

        queue!(
            stdout,
            SetForegroundColor(Color::White),
            Print(format!("  {:2}", value)),
        )?;

        // Show arrows if adjustable
        let can_decrease = value > MIN_STAT;
        let can_increase = value < MAX_STAT && points_remaining > 0;

        queue!(
            stdout,
            SetForegroundColor(if can_decrease {
                Color::Green
            } else {
                Color::DarkGrey
            }),
            Print(" ◄"),
            SetForegroundColor(if can_increase {
                Color::Green
            } else {
                Color::DarkGrey
            }),
            Print("►"),
        )?;

        // Description on next line
        queue!(
            stdout,
            cursor::MoveTo(4, row + 1),
            SetForegroundColor(Color::DarkGrey),
            Print(format!("└─ {}", description)),
        )?;
    } else {
        // Render unselected stat
        queue!(
            stdout,
            cursor::MoveTo(2, row),
            SetForegroundColor(Color::DarkGrey),
            Print("  "),
            SetForegroundColor(Color::White),
            Print(format!("{:12} ", name)),
        )?;

        render_stat_bar(stdout, value, false)?;

        queue!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  {:2}", value)),
        )?;
    }

    Ok(())
}

fn render_stat_bar(stdout: &mut io::Stdout, value: u8, highlighted: bool) -> io::Result<()> {
    let filled = value as usize;
    let empty = MAX_STAT as usize - filled;

    queue!(stdout, Print(" ["))?;

    for i in 0..filled {
        let color = if highlighted {
            match i {
                0..=3 => Color::Red,
                4..=6 => Color::Yellow,
                _ => Color::Green,
            }
        } else {
            Color::DarkGrey
        };

        queue!(stdout, SetForegroundColor(color), Print("█"))?;
    }

    for _ in 0..empty {
        queue!(
            stdout,
            SetForegroundColor(if highlighted {
                Color::DarkGrey
            } else {
                Color::DarkGrey
            }),
            Print("░")
        )?;
    }

    queue!(
        stdout,
        SetForegroundColor(if highlighted {
            Color::White
        } else {
            Color::DarkGrey
        }),
        Print("]")
    )?;

    Ok(())
}

impl SelectedStat {
    fn from_index(i: usize) -> Self {
        match i {
            0 => SelectedStat::Strength,
            1 => SelectedStat::Perception,
            2 => SelectedStat::Endurance,
            3 => SelectedStat::Charisma,
            4 => SelectedStat::Intelligence,
            5 => SelectedStat::Agility,
            6 => SelectedStat::Luck,
            _ => SelectedStat::Strength,
        }
    }
}
