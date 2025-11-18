# 🎨 Enhanced TUI Character Sheet

## Overview

A beautiful, visually hierarchical character sheet for your Fallout D&D game using `ratatui`.

## Features Implemented

### ✅ **Visual Hierarchy**
- **Double-bordered main frame** with centered "CHARACTER" title
- **Four distinct sections** with clear separation
- **Beautiful box-drawing characters** for sub-sections
- **Color-coded elements** for quick visual parsing

### ✅ **Section 1: Character Header**
```
  VAULT DWELLER                                        Level 3
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```
- Character name in **CYAN BOLD UPPERCASE**
- Level displayed on the right
- Horizontal separator line

### ✅ **Section 2: Vitals with Progress Bars**
```
  ❤ Health  45/50  ████████████████░░
  ⚡ Action  8/8    ████████████████████
  ⭐ XP     2100/3000  ██████████████░░░░░
  💰 Caps   347
```
- **Color-coded icons and labels:**
  - ❤ Health (RED)
  - ⚡ Action Points (YELLOW)
  - ⭐ Experience (MAGENTA)
  - 💰 Caps (GREEN)
- **Dynamic progress bars** that adjust based on current/max values
- **20-character wide bars** for consistent visual appeal

### ✅ **Section 3: S.P.E.C.I.A.L. Stats**
```
  ┌─ S.P.E.C.I.A.L. ───────────────────────────────────┐
  │ Strength      [████████░░]  8   Melee DMG +8       │
  │ Perception    [██████░░░░]  6   Accuracy +6        │
  │ Endurance     [████░░░░░░]  4   HP +8              │
  │ Charisma      [███░░░░░░░]  3   Barter -10%        │
  │ Intelligence  [███████░░░]  7   Skill Pts +14      │
  │ Agility       [██████░░░░]  6   AC 16              │
  │ Luck          [█████░░░░░]  5   Critical +5%       │
  └────────────────────────────────────────────────────┘
```
- **10-segment stat bars** showing visual progression
- **Numeric value** displayed after each bar
- **Practical benefits** shown for each stat:
  - Strength → Melee Damage
  - Perception → Accuracy bonus
  - Endurance → HP bonus
  - Charisma → Barter modifier (can be negative!)
  - Intelligence → Skill points per level
  - Agility → Armor Class
  - Luck → Critical hit chance
- **Box-drawing borders** for elegant containment

### ✅ **Section 4: Top Skills**
```
  ┌─ TOP SKILLS ──────────────────────────────────────┐
  │ Small Guns     ████████████████░░░░  52  Good     │
  │ Science        ████████████████████░  63  Excellent │
  │ Lockpick       ██████████░░░░░░░░░░  31  Fair     │
  │ Speech         ███████░░░░░░░░░░░░░  25  Poor     │
  └────────────────────────────────────────────────────┘
```
- **Automatically selects top 4 skills** by value
- **20-character progress bars** (0-100 scale)
- **Numeric skill value** displayed
- **Skill rating** with color coding:
  - **Master** (96-100) - MAGENTA
  - **Excellent** (81-95) - GREEN
  - **Good** (61-80) - LIGHT GREEN
  - **Fair** (41-60) - YELLOW
  - **Poor** (21-40) - LIGHT RED
  - **Novice** (0-20) - RED

## How to Access

In-game, use any of these commands:
- `stats`
- `sheet`
- `3` (from the numbered menu)

Press **ESC** to return to the main game view.

## Color Scheme

| Element | Color | Purpose |
|---------|-------|---------|
| Main Border | GREEN (Double) | Frame containment |
| Character Name | CYAN BOLD | High visibility |
| Section Titles | CYAN/YELLOW BOLD | Section identification |
| Health | RED | Danger indicator |
| Action Points | YELLOW | Resource indicator |
| Experience | MAGENTA | Progress tracking |
| Caps | GREEN | Wealth indicator |
| SPECIAL Bars | GREEN | Stat visualization |
| Skill Bars | CYAN | Skill visualization |
| Benefits/Ratings | DARK GRAY | Supporting info |
| Borders/Lines | DARK GRAY | Visual structure |

## Technical Implementation

### Files Modified
- `src/tui/ui.rs` - Complete rewrite of `render_detailed_stats()` function
- Added helper functions:
  - `render_sheet_header()` - Character name and level
  - `render_sheet_vitals()` - HP, AP, XP, Caps with bars
  - `render_sheet_special()` - SPECIAL stats with benefits
  - `render_sheet_skills()` - Top 4 skills with ratings
  - `make_special_line()` - SPECIAL stat line builder
  - `make_skill_line()` - Skill line with rating
  - `make_stat_bar()` - 10-segment bar for stats
  - `make_progress_bar()` - Variable-width progress bar

### Responsive Design
- **Automatic skill sorting** - Always shows your best skills
- **Dynamic progress bars** - Adjust to current values
- **Benefit calculations** - Real-time stat benefit display
- **Rating system** - Skill proficiency levels

## Example Output

```
╔══════════════════════════ CHARACTER ══════════════════════════╗
║                                                                ║
║  VAULT DWELLER                                    Level 3      ║
║  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ ║
║                                                                ║
║  ❤ Health  45/50  ████████████████░░                          ║
║  ⚡ Action  8/8    ████████████████████                        ║
║  ⭐ XP     2100/3000  ██████████████░░░░░                      ║
║  💰 Caps   347                                                 ║
║                                                                ║
║  ┌─ S.P.E.C.I.A.L. ───────────────────────────────────────┐   ║
║  │ Strength      [████████░░]  8   Melee DMG +8          │   ║
║  │ Perception    [██████░░░░]  6   Accuracy +6           │   ║
║  │ Endurance     [████░░░░░░]  4   HP +8                 │   ║
║  │ Charisma      [███░░░░░░░]  3   Barter -10%           │   ║
║  │ Intelligence  [███████░░░]  7   Skill Pts +14         │   ║
║  │ Agility       [██████░░░░]  6   AC 16                 │   ║
║  │ Luck          [█████░░░░░]  5   Critical +5%          │   ║
║  └────────────────────────────────────────────────────────┘   ║
║                                                                ║
║  ┌─ TOP SKILLS ──────────────────────────────────────────┐   ║
║  │ Small Guns     ████████████████░░░░  52  Good         │   ║
║  │ Science        ████████████████████░  63  Excellent   │   ║
║  │ Lockpick       ██████████░░░░░░░░░░  31  Fair         │   ║
║  │ Speech         ███████░░░░░░░░░░░░░  25  Poor         │   ║
║  └────────────────────────────────────────────────────────┘   ║
╚════════════════════════════════════════════════════════════════╝
```

## Future Enhancements (Optional)

### Possible Additions:
1. **Perks/Traits Section** - Display special character abilities
2. **Conditions/Status Effects** - Show active buffs/debuffs
3. **Equipment Section** - Display equipped weapon and armor
4. **Reputation Tracker** - Show faction standings
5. **Interactive Tooltips** - Hover for detailed stat descriptions
6. **Scrollable Full Skills List** - View all 20 skills with PageUp/PageDown
7. **Animated Bars** - Smooth transitions when stats change
8. **Color Themes** - Different color schemes (Classic Fallout green, Vault-Tec blue, etc.)

## Performance

- **Minimal overhead** - Only renders when view changes
- **Efficient sorting** - Top skills calculated once per render
- **No external assets** - Pure Unicode box drawing
- **Fast rendering** - Uses ratatui's optimized terminal backend

## Accessibility

- **High contrast** - Color choices work on most terminal themes
- **Clear hierarchy** - Visual structure aids navigation
- **Readable fonts** - Works with all monospace terminals
- **No complex Unicode** - Basic box-drawing characters only

---

**Built with:** Rust + Ratatui + Love ❤️

**Last Updated:** November 2024
