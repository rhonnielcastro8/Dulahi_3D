// =============================================================
//  DULAHI 3D – Pre-Game Loading Screen
//  File: src/ui/loading.rs
//
//  Shown BEFORE each game session (NOT the app splash screen).
//  Displays a trivia fact specific to the selected Larong Lahi
//  game, an animated loading bar, and the game name.
//
//  Flow: GameSelect / Lobby → Loading → InGame
// =============================================================

use bevy::prelude::*;
use crate::ui::{GameState, SelectedGame, colors, despawn_screen};

// ── Public plugin ─────────────────────────────────────────────

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(LoadingProgress::default())
            .add_systems(OnEnter(GameState::Loading),  spawn_loading)
            .add_systems(
                Update,
                (tick_loading, animate_loading_bar, cycle_trivia)
                    .run_if(in_state(GameState::Loading)),
            )
            .add_systems(OnExit(GameState::Loading), despawn_screen::<LoadingRoot>);
    }
}

// ── Resources ─────────────────────────────────────────────────

/// Simulated loading progress (0.0 → 1.0).
/// In production this would be driven by actual asset loading handles.
#[derive(Resource)]
struct LoadingProgress {
    value:         f32,   // 0.0 – 1.0
    elapsed:       f32,
    trivia_index:  usize,
    trivia_timer:  f32,
    /// Set to true once value reaches 1.0 and we are ready to transition.
    ready:         bool,
}
impl Default for LoadingProgress {
    fn default() -> Self {
        Self {
            value:        0.0,
            elapsed:      0.0,
            trivia_index: 0,
            trivia_timer: 0.0,
            ready:        false,
        }
    }
}

/// Minimum time to show the loading screen (even if assets load instantly).
const MIN_DURATION: f32 = 3.5;
/// How often (seconds) the trivia fact cycles.
const TRIVIA_CYCLE: f32 = 3.0;

// ── Markers ───────────────────────────────────────────────────

#[derive(Component)]
pub struct LoadingRoot;

#[derive(Component)]
struct LoadingBarFill;

#[derive(Component)]
struct LoadingPercentLabel;

#[derive(Component)]
struct TriviaLabel;

#[derive(Component)]
struct GameNameLabel;

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_loading(
    mut commands:  Commands,
    mut progress:  ResMut<LoadingProgress>,
    selected_game: Res<SelectedGame>,
) {
    // Reset progress for a fresh loading session
    *progress = LoadingProgress::default();

    let game_name   = selected_game.0.display_name();
    let trivia_list = selected_game.0.trivia();
    let first_trivia = trivia_list.first().copied().unwrap_or("Loading…");

    commands
        .spawn((
            LoadingRoot,
            Node {
                width:           Val::Percent(100.0),
                height:          Val::Percent(100.0),
                flex_direction:  FlexDirection::Column,
                align_items:     AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding:         UiRect::all(Val::Px(40.0)),
                row_gap:         Val::Px(0.0),
                ..default()
            },
            BackgroundColor(colors::BG_DARK),
            ZIndex(50),
        ))
        .with_children(|root| {

            // ── Top gold strip ──
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top:    Val::Px(0.0),
                    left:   Val::Px(0.0),
                    right:  Val::Px(0.0),
                    height: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(colors::GOLD),
            ));

            // ── Game name chip ──
            root.spawn((
                Node {
                    padding:       UiRect { left: Val::Px(20.0), right: Val::Px(20.0), top: Val::Px(6.0), bottom: Val::Px(6.0) },
                    border:        UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(20.0)),
                    margin:        UiRect::bottom(Val::Px(14.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.91, 0.79, 0.48, 0.10)),
                BorderColor::all(Color::srgba(0.91, 0.79, 0.48, 0.35)),
            ))
            .with_children(|chip| {
                chip.spawn((
                    GameNameLabel,
                    Text::new(game_name.to_uppercase()),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(colors::GOLD),
                    TextLayout::new_with_justify(Justify::Center),
                ));
            });

            // ── Main "LOADING" heading ──
            root.spawn((
                Text::new("LOADING"),
                TextFont { font_size: 52.0, ..default() },
                TextColor(colors::WHITE),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // ── Tagline below heading ──
            root.spawn((
                Text::new("Preparing your game…"),
                TextFont { font_size: 15.0, ..default() },
                TextColor(colors::TEXT_MUTED),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(48.0)),
                    ..default()
                },
            ));

            // ── Trivia card ──
            root.spawn((
                Node {
                    width:         Val::Px(580.0),
                    padding:       UiRect::all(Val::Px(24.0)),
                    border:        UiRect::all(Val::Px(1.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap:       Val::Px(10.0),
                    margin:        UiRect::bottom(Val::Px(48.0)),
                    border_radius: BorderRadius::all(Val::Px(14.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                BorderColor::all(Color::srgba(0.91, 0.79, 0.48, 0.20)),
            ))
            .with_children(|card| {
                // "DID YOU KNOW?" label
                card.spawn((
                    Text::new("DID YOU KNOW?"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(colors::GOLD),
                    TextLayout::new_with_justify(Justify::Left),
                ));

                // Divider
                card.spawn((
                    Node {
                        width:  Val::Percent(100.0),
                        height: Val::Px(1.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.91, 0.79, 0.48, 0.18)),
                ));

                // Trivia text (updated each cycle)
                card.spawn((
                    TriviaLabel,
                    Text::new(first_trivia),
                    TextFont { font_size: 15.0, ..default() },
                    TextColor(Color::srgb(0.90, 0.90, 0.90)),
                    TextLayout::new_with_justify(Justify::Left),
                ));
            });

            // ── Loading bar container ──
            root.spawn((
                Node {
                    width:  Val::Px(560.0),
                    height: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.07)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    LoadingBarFill,
                    Node {
                        width:  Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(colors::GOLD),
                ));
            });

            // ── Percentage label ──
            root.spawn((
                LoadingPercentLabel,
                Text::new("0%"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(colors::TEXT_MUTED),
                TextLayout::new_with_justify(Justify::Center),
            ));

            // ── Bottom gold strip ──
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left:   Val::Px(0.0),
                    right:  Val::Px(0.0),
                    height: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(colors::GOLD),
            ));
        });
}

// ── Systems ───────────────────────────────────────────────────

/// Advances the simulated loading progress and triggers transition.
fn tick_loading(
    time:          Res<Time>,
    mut progress:  ResMut<LoadingProgress>,
    mut next:      ResMut<NextState<GameState>>,
) {
    if progress.ready { return; }

    progress.elapsed += time.delta_secs();

    // Ease-in-out curve so bar doesn't stall at the end
    let target = (progress.elapsed / MIN_DURATION).clamp(0.0, 1.0);
    // Smooth step
    let t = target * target * (3.0 - 2.0 * target);
    progress.value = t;

    if progress.elapsed >= MIN_DURATION {
        progress.value = 1.0;
        progress.ready = true;
        next.set(GameState::InGame);
    }
}

/// Keeps the bar fill width and percentage label in sync with progress.
fn animate_loading_bar(
    progress:     Res<LoadingProgress>,
    mut bar_q:    Query<&mut Node, With<LoadingBarFill>>,
    mut label_q:  Query<&mut Text, With<LoadingPercentLabel>>,
) {
    let pct = (progress.value * 100.0) as u32;
    for mut node in &mut bar_q {
        node.width = Val::Percent(progress.value * 100.0);
    }
    for mut text in &mut label_q {
        *text = Text::new(format!("{}%", pct));
    }
}

/// Rotates through the game-specific trivia facts on a timer.
fn cycle_trivia(
    time:          Res<Time>,
    mut progress:  ResMut<LoadingProgress>,
    selected_game: Res<SelectedGame>,
    mut text_q:    Query<&mut Text, With<TriviaLabel>>,
) {
    progress.trivia_timer += time.delta_secs();
    if progress.trivia_timer < TRIVIA_CYCLE { return; }
    progress.trivia_timer = 0.0;

    let facts = selected_game.0.trivia();
    if facts.is_empty() { return; }
    progress.trivia_index = (progress.trivia_index + 1) % facts.len();

    let fact = facts[progress.trivia_index];
    for mut text in &mut text_q {
        *text = Text::new(fact);
    }
}