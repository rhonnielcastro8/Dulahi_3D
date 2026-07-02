// =============================================================
//  DULAHI 3D – Game Select Screen (Single Player)
//  File: src/ui/game_select.rs
//
//  Single-player path: pick a Larong Lahi game, a team format
//  (1v1 / 2v2 / 3v3 — filtered to what that game supports), and
//  a difficulty. AI fills every slot except the player's own,
//  on BOTH teams, mirroring the multiplayer lobby's team layout.
// =============================================================

use bevy::prelude::*;
use crate::ui::{
    GameState, LahiGame, SelectedGame, SelectedFormat, MatchFormat,
    Difficulty, colors, despawn_screen,
};

// ── Public plugin ─────────────────────────────────────────────

pub struct GameSelectPlugin;

impl Plugin for GameSelectPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameSelectState>()
            .add_systems(OnEnter(GameState::GameSelect),  spawn_game_select)
            .add_systems(
                Update,
                (handle_game_cards, handle_format_buttons, handle_diff_buttons,
                 handle_nav_buttons, repaint_game_cards, repaint_format_buttons,
                 repaint_diff_buttons, repaint_start_button)
                    .run_if(in_state(GameState::GameSelect)),
            )
            .add_systems(OnExit(GameState::GameSelect), despawn_screen::<GameSelectRoot>);
    }
}

// ── Local state ─────────────────────────────────────────────────

#[derive(Resource, Default)]
struct GameSelectState {
    chosen_game:   Option<usize>,        // index into LahiGame::all()
    chosen_format: Option<MatchFormat>,
    chosen_diff:   usize,                // 0=Easy 1=Normal 2=Hard
}

const GAME_META: [(&str, &str); 5] = [
    ("◉", "Keep the sipa/takyan airborne"),
    ("◎", "Knock down the lata, guard it"),
    ("⊞", "Cross the grid as a team"),
    ("◫", "Hop the grid without a slip"),
    ("⇑", "Jump the tinik in time"),
];

// ── Markers ───────────────────────────────────────────────────

#[derive(Component)]
pub struct GameSelectRoot;

#[derive(Component)]
struct GameCard(usize);

#[derive(Component)]
struct FormatButton(MatchFormat);

#[derive(Component)]
struct DiffButton(usize);

#[derive(Component)]
enum NavButton { Start, Back }

#[derive(Component)]
struct StartButton;
#[derive(Component)]
struct StartButtonLabel;

/// Rebuilt every time the chosen game changes.
#[derive(Component)]
struct OptionsPanel;

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_game_select(mut commands: Commands, mut local: ResMut<GameSelectState>) {
    *local = GameSelectState::default();

    commands
        .spawn((
            GameSelectRoot,
            Node {
                width:           Val::Percent(100.0),
                height:          Val::Percent(100.0),
                flex_direction:  FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.07, 0.10, 0.97)),
            ZIndex(20),
        ))
        .with_children(|root| {

            // ═══════════════════════════════════════════
            //  LEFT – game cards
            // ═══════════════════════════════════════════
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    width:          Val::Px(320.0),
                    padding:        UiRect::all(Val::Px(24.0)),
                    row_gap:        Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
            ))
            .with_children(|left| {
                left.spawn((
                    Text::new("SELECT GAME"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(colors::GOLD),
                ));

                for (i, game) in LahiGame::all().iter().enumerate() {
                    let (icon, desc) = GAME_META[i];
                    left.spawn((
                        GameCard(i),
                        Button,
                        Node {
                            flex_direction:  FlexDirection::Row,
                            align_items:     AlignItems::Center,
                            column_gap:      Val::Px(12.0),
                            height:          Val::Px(64.0),
                            padding:         UiRect::horizontal(Val::Px(14.0)),
                            border:          UiRect::all(Val::Px(1.5)),
                            border_radius:   BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(colors::BLUE_DARK),
                        BorderColor::all(Color::srgba(0.40, 0.40, 0.94, 0.25)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(icon),
                            TextFont { font_size: 26.0, ..default() },
                            TextColor(colors::BLUE_LIGHT),
                        ));
                        card.spawn((
                            Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), ..default() },
                        ))
                        .with_children(|col| {
                            col.spawn((
                                Text::new(game.display_name()),
                                TextFont { font_size: 15.0, ..default() },
                                TextColor(colors::WHITE),
                            ));
                            col.spawn((
                                Text::new(desc),
                                TextFont { font_size: 10.0, ..default() },
                                TextColor(colors::TEXT_MUTED),
                            ));
                        });
                    });
                }
            });

            // ═══════════════════════════════════════════
            //  RIGHT – format, difficulty, team preview
            // ═══════════════════════════════════════════
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow:      1.0,
                    padding:        UiRect::all(Val::Px(28.0)),
                    row_gap:        Val::Px(18.0),
                    ..default()
                },
            ))
            .with_children(|right| {
                right.spawn((
                    OptionsPanel,
                    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(18.0), flex_grow: 1.0, ..default() },
                ))
                .with_children(|panel| {
                    spawn_placeholder(panel);
                });

                // Nav row
                right.spawn((
                    Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() },
                ))
                .with_children(|nav| {
                    nav.spawn((
                        NavButton::Back,
                        Button,
                        Node {
                            width: Val::Px(130.0), height: Val::Px(44.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("◀  BACK"), TextFont { font_size: 14.0, ..default() }, TextColor(colors::TEXT_MUTED)));
                    });

                    nav.spawn((
                        NavButton::Start,
                        StartButton,
                        Button,
                        Node {
                            width: Val::Px(180.0), height: Val::Px(44.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.30, 0.10, 0.40)),
                        BorderColor::all(Color::srgba(0.40, 0.94, 0.40, 0.15)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            StartButtonLabel,
                            Text::new("START  ▶"),
                            TextFont { font_size: 15.0, ..default() },
                            TextColor(Color::srgba(0.40, 0.94, 0.40, 0.30)),
                        ));
                    });
                });
            });
        });
}

fn spawn_placeholder(panel: &mut ChildSpawnerCommands) {
    panel.spawn((
        Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, flex_grow: 1.0, ..default() },
    ))
    .with_children(|p| {
        p.spawn((
            Text::new("Choose a game on the left to configure your match"),
            TextFont { font_size: 15.0, ..default() },
            TextColor(colors::TEXT_MUTED),
        ));
    });
}

/// Rebuilds the right panel's contents for the given game/format/difficulty.
fn spawn_options_content(
    panel:  &mut ChildSpawnerCommands,
    game:   LahiGame,
    format: MatchFormat,
    diff_idx: usize,
) {
    panel.spawn((
        Text::new(format!("{} — MATCH SETUP", game.display_name().to_uppercase())),
        TextFont { font_size: 20.0, ..default() },
        TextColor(colors::GOLD),
    ));

    // Format row
    panel.spawn((
        Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(8.0), ..default() },
    ))
    .with_children(|s| {
        s.spawn((Text::new("Team Format"), TextFont { font_size: 13.0, ..default() }, TextColor(colors::TEXT_MUTED)));
        s.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() })
            .with_children(|row| {
                for f in game.available_formats() {
                    let selected = f == format;
                    row.spawn((
                        FormatButton(f),
                        Button,
                        Node {
                            width: Val::Px(90.0), height: Val::Px(38.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(if selected { colors::GREEN_MID } else { Color::srgba(0.10, 0.10, 0.10, 0.80) }),
                        BorderColor::all(if selected { Color::srgba(0.40, 0.94, 0.40, 0.60) } else { Color::srgba(1.0, 1.0, 1.0, 0.15) }),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(f.label()),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(if selected { colors::GREEN_LIGHT } else { colors::TEXT_MUTED }),
                        ));
                    });
                }
            });
    });

    // Difficulty row
    panel.spawn((
        Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(8.0), ..default() },
    ))
    .with_children(|s| {
        s.spawn((Text::new("AI Difficulty"), TextFont { font_size: 13.0, ..default() }, TextColor(colors::TEXT_MUTED)));
        s.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() })
            .with_children(|row| {
                for (i, label) in ["EASY", "NORMAL", "HARD"].iter().enumerate() {
                    let selected = i == diff_idx;
                    row.spawn((
                        DiffButton(i),
                        Button,
                        Node {
                            width: Val::Px(90.0), height: Val::Px(38.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(if selected { colors::AMBER_DARK } else { Color::srgba(0.10, 0.10, 0.10, 0.80) }),
                        BorderColor::all(if selected { Color::srgba(0.96, 0.78, 0.40, 0.60) } else { Color::srgba(1.0, 1.0, 1.0, 0.15) }),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(*label),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(if selected { colors::AMBER_LIGHT } else { colors::TEXT_MUTED }),
                        ));
                    });
                }
            });
    });

    // Team preview
    let team_size = format.team_size();
    let diff_label = ["Easy", "Normal", "Hard"][diff_idx];
    panel.spawn((
        Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(20.0), flex_grow: 1.0, ..default() },
    ))
    .with_children(|teams| {
        spawn_team_box(teams, "YOUR TEAM", colors::GREEN_MID, colors::GREEN_LIGHT, {
            let mut v = vec!["You".to_string()];
            for _ in 1..team_size { v.push(format!("AI Teammate ({})", diff_label)); }
            v
        });
        spawn_team_box(teams, "ENEMY TEAM", colors::RED_DARK, colors::RED_LIGHT, {
            (0..team_size).map(|_| format!("AI Opponent ({})", diff_label)).collect()
        });
    });
}

fn spawn_team_box(
    parent: &mut ChildSpawnerCommands,
    title:  &str,
    bg:     Color,
    fg:     Color,
    members: Vec<String>,
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow:      1.0,
            padding:        UiRect::all(Val::Px(16.0)),
            border:         UiRect::all(Val::Px(1.0)),
            border_radius:  BorderRadius::all(Val::Px(10.0)),
            row_gap:        Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(bg.to_srgba().red, bg.to_srgba().green, bg.to_srgba().blue, 0.18)),
        BorderColor::all(Color::srgba(fg.to_srgba().red, fg.to_srgba().green, fg.to_srgba().blue, 0.35)),
    ))
    .with_children(|box_| {
        box_.spawn((Text::new(title), TextFont { font_size: 13.0, ..default() }, TextColor(fg)));
        for m in members {
            box_.spawn((
                Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(8.0), height: Val::Px(30.0), ..default() },
            ))
            .with_children(|row| {
                row.spawn((
                    Node { width: Val::Px(8.0), height: Val::Px(8.0), border_radius: BorderRadius::all(Val::Percent(50.0)), ..default() },
                    BackgroundColor(fg),
                ));
                row.spawn((Text::new(m), TextFont { font_size: 13.0, ..default() }, TextColor(colors::WHITE)));
            });
        }
    });
}

// ── Systems ───────────────────────────────────────────────────

fn handle_game_cards(
    interaction_q: Query<(&Interaction, &GameCard), (Changed<Interaction>, With<Button>)>,
    mut local:     ResMut<GameSelectState>,
    mut commands:  Commands,
    panel_q:       Query<Entity, With<OptionsPanel>>,
) {
    let mut changed_to: Option<usize> = None;
    for (interaction, card) in &interaction_q {
        if *interaction == Interaction::Pressed {
            changed_to = Some(card.0);
        }
    }
    if let Some(idx) = changed_to {
        local.chosen_game = Some(idx);
        let game = LahiGame::all()[idx];
        let formats = game.available_formats();
        local.chosen_format = formats.first().copied();
        local.chosen_diff = 1;

        if let Ok(panel_entity) = panel_q.single() {
            commands.entity(panel_entity).despawn_related::<Children>();
            if let Some(format) = local.chosen_format {
                let diff = local.chosen_diff;
                commands.entity(panel_entity).with_children(|panel| {
                    spawn_options_content(panel, game, format, diff);
                });
            }
        }
    }
}

fn handle_format_buttons(
    interaction_q: Query<(&Interaction, &FormatButton), (Changed<Interaction>, With<Button>)>,
    mut local:     ResMut<GameSelectState>,
    mut commands:  Commands,
    panel_q:       Query<Entity, With<OptionsPanel>>,
) {
    let mut new_format = None;
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            new_format = Some(btn.0);
        }
    }
    if let (Some(format), Some(idx)) = (new_format, local.chosen_game) {
        local.chosen_format = Some(format);
        let game = LahiGame::all()[idx];
        let diff = local.chosen_diff;
        if let Ok(panel_entity) = panel_q.single() {
            commands.entity(panel_entity).despawn_related::<Children>();
            commands.entity(panel_entity).with_children(|panel| {
                spawn_options_content(panel, game, format, diff);
            });
        }
    }
}

fn handle_diff_buttons(
    interaction_q: Query<(&Interaction, &DiffButton), (Changed<Interaction>, With<Button>)>,
    mut local:     ResMut<GameSelectState>,
    mut commands:  Commands,
    panel_q:       Query<Entity, With<OptionsPanel>>,
) {
    let mut new_diff = None;
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            new_diff = Some(btn.0);
        }
    }
    if let (Some(diff), Some(idx), Some(format)) = (new_diff, local.chosen_game, local.chosen_format) {
        local.chosen_diff = diff;
        let game = LahiGame::all()[idx];
        if let Ok(panel_entity) = panel_q.single() {
            commands.entity(panel_entity).despawn_related::<Children>();
            commands.entity(panel_entity).with_children(|panel| {
                spawn_options_content(panel, game, format, diff);
            });
        }
    }
}

fn handle_nav_buttons(
    interaction_q: Query<(&Interaction, &NavButton), (Changed<Interaction>, With<Button>)>,
    mut next:  ResMut<NextState<GameState>>,
    local:     Res<GameSelectState>,
    mut sel:   ResMut<SelectedGame>,
    mut fmt:   ResMut<SelectedFormat>,
    mut diff:  ResMut<Difficulty>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction != Interaction::Pressed { continue; }
        match button {
            NavButton::Back => next.set(GameState::ModeSelect),
            NavButton::Start => {
                if let (Some(idx), Some(format)) = (local.chosen_game, local.chosen_format) {
                    sel.0 = LahiGame::all()[idx];
                    fmt.0 = format;
                    *diff = match local.chosen_diff {
                        0 => Difficulty::Easy,
                        2 => Difficulty::Hard,
                        _ => Difficulty::Normal,
                    };
                    next.set(GameState::Loading);
                }
            }
        }
    }
}

/// Keeps game-card highlight in sync with chosen_game every frame.
fn repaint_game_cards(
    local: Res<GameSelectState>,
    mut q: Query<(&GameCard, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (card, mut bg, mut border) in &mut q {
        let selected = local.chosen_game == Some(card.0);
        *bg = BackgroundColor(if selected { Color::srgb(0.14, 0.14, 0.32) } else { colors::BLUE_DARK });
        *border = BorderColor::all(if selected { Color::srgba(0.40, 0.40, 0.94, 0.70) } else { Color::srgba(0.40, 0.40, 0.94, 0.25) });
    }
}

/// Keeps format-button highlight in sync (covers rebuilt panels).
fn repaint_format_buttons(
    local: Res<GameSelectState>,
    mut q: Query<(&FormatButton, &mut BackgroundColor, &mut BorderColor, &Children)>,
    mut text_q: Query<&mut TextColor>,
) {
    for (btn, mut bg, mut border, children) in &mut q {
        let selected = local.chosen_format == Some(btn.0);
        *bg = BackgroundColor(if selected { colors::GREEN_MID } else { Color::srgba(0.10, 0.10, 0.10, 0.80) });
        *border = BorderColor::all(if selected { Color::srgba(0.40, 0.94, 0.40, 0.60) } else { Color::srgba(1.0, 1.0, 1.0, 0.15) });
        for child in children.iter() {
            if let Ok(mut tc) = text_q.get_mut(child) {
                *tc = TextColor(if selected { colors::GREEN_LIGHT } else { colors::TEXT_MUTED });
            }
        }
    }
}

fn repaint_diff_buttons(
    local: Res<GameSelectState>,
    mut q: Query<(&DiffButton, &mut BackgroundColor, &mut BorderColor, &Children)>,
    mut text_q: Query<&mut TextColor>,
) {
    for (btn, mut bg, mut border, children) in &mut q {
        let selected = btn.0 == local.chosen_diff;
        *bg = BackgroundColor(if selected { colors::AMBER_DARK } else { Color::srgba(0.10, 0.10, 0.10, 0.80) });
        *border = BorderColor::all(if selected { Color::srgba(0.96, 0.78, 0.40, 0.60) } else { Color::srgba(1.0, 1.0, 1.0, 0.15) });
        for child in children.iter() {
            if let Ok(mut tc) = text_q.get_mut(child) {
                *tc = TextColor(if selected { colors::AMBER_LIGHT } else { colors::TEXT_MUTED });
            }
        }
    }
}

/// Keeps the Start button visually active only once a game+format
/// is chosen — fixes the bug where Start never lit up.
fn repaint_start_button(
    local: Res<GameSelectState>,
    mut btn_q: Query<(&mut BackgroundColor, &mut BorderColor), With<StartButton>>,
    mut label_q: Query<&mut TextColor, With<StartButtonLabel>>,
) {
    let ready = local.chosen_game.is_some() && local.chosen_format.is_some();
    for (mut bg, mut border) in &mut btn_q {
        *bg = BackgroundColor(if ready { colors::GREEN_MID } else { Color::srgba(0.10, 0.30, 0.10, 0.40) });
        *border = BorderColor::all(if ready { Color::srgba(0.40, 0.94, 0.40, 0.60) } else { Color::srgba(0.40, 0.94, 0.40, 0.15) });
    }
    for mut tc in &mut label_q {
        *tc = TextColor(if ready { colors::GREEN_LIGHT } else { Color::srgba(0.40, 0.94, 0.40, 0.30) });
    }
}