// =============================================================
//  DULAHI 3D – Options Screen
//  File: src/ui/options.rs
//
//  Settings panel: audio volume, music toggle, SFX toggle.
//  Reached from the main menu. Back button returns to MainMenu.
// =============================================================

use bevy::prelude::*;
use crate::ui::{GameState, colors, despawn_screen};

// ── Public plugin ─────────────────────────────────────────────

pub struct OptionsPlugin;

impl Plugin for OptionsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AudioSettings>()
            .add_systems(OnEnter(GameState::Options),  spawn_options)
            .add_systems(
                Update,
                (handle_nav_and_toggles, handle_volume_buttons, repaint_volume)
                    .run_if(in_state(GameState::Options)),
            )
            .add_systems(OnExit(GameState::Options), despawn_screen::<OptionsRoot>);
    }
}

// ── Resources ─────────────────────────────────────────────────

#[derive(Resource)]
pub struct AudioSettings {
    pub master_volume: f32,   // 0.0 – 1.0
    pub music_enabled: bool,
    pub sfx_enabled:   bool,
}
impl Default for AudioSettings {
    fn default() -> Self {
        Self { master_volume: 0.8, music_enabled: true, sfx_enabled: true }
    }
}

// ── Markers ───────────────────────────────────────────────────

#[derive(Component)]
pub struct OptionsRoot;

#[derive(Component, Clone, Copy)]
enum NavButton { Back, ToggleMusic, ToggleSfx }

#[derive(Component, Clone, Copy)]
enum VolumeButton { Down, Up }

#[derive(Component)]
struct VolumeTrackFill;

#[derive(Component)]
struct VolumeLabel;

#[derive(Component)]
struct MusicToggle;

#[derive(Component)]
struct SfxToggle;

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_options(mut commands: Commands, audio: Res<AudioSettings>) {
    commands
        .spawn((
            OptionsRoot,
            Node {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.07, 0.10, 0.96)),
            ZIndex(20),
        ))
        .with_children(|root| {

            root.spawn((
                Text::new("OPTIONS"),
                TextFont { font_size: 38.0, ..default() },
                TextColor(colors::GOLD),
                Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
            ));

            root.spawn((
                Node { width: Val::Px(320.0), height: Val::Px(1.0), margin: UiRect::bottom(Val::Px(36.0)), ..default() },
                BackgroundColor(Color::srgba(0.91, 0.79, 0.48, 0.30)),
            ));

            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column, align_items: AlignItems::Stretch,
                    width: Val::Px(420.0), padding: UiRect::all(Val::Px(28.0)),
                    border: UiRect::all(Val::Px(1.0)), row_gap: Val::Px(22.0),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.10)),
            ))
            .with_children(|card| {

                // ── Master Volume ──
                card.spawn((
                    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(10.0), ..default() },
                ))
                .with_children(|row| {
                    row.spawn((
                        VolumeLabel,
                        Text::new(format!("Master Volume  {:>3}%", (audio.master_volume * 100.0) as u32)),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(colors::WHITE),
                    ));

                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(10.0),
                            ..default()
                        },
                    ))
                    .with_children(|slider_row| {
                        // Track
                        slider_row.spawn((
                            Node {
                                width: Val::Percent(100.0), height: Val::Px(8.0),
                                border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(4.0)),
                                flex_grow: 1.0,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.10)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                        ))
                        .with_children(|track| {
                            track.spawn((
                                VolumeTrackFill,
                                Node { width: Val::Percent(audio.master_volume * 100.0), height: Val::Percent(100.0), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() },
                                BackgroundColor(colors::GOLD),
                            ));
                        });
                    });

                    // +/- nudge buttons — now correctly wired
                    row.spawn((
                        Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() },
                    ))
                    .with_children(|nudge| {
                        for (label, action) in [("−", VolumeButton::Down), ("+", VolumeButton::Up)] {
                            nudge.spawn((
                                action, Button,
                                Node {
                                    width: Val::Px(38.0), height: Val::Px(30.0),
                                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.07)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new(label), TextFont { font_size: 18.0, ..default() }, TextColor(colors::WHITE)));
                            });
                        }
                    });
                });

                card.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(1.0), ..default() },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                ));

                spawn_toggle_row(card, "Music", audio.music_enabled, NavButton::ToggleMusic, MusicToggle);
                spawn_toggle_row(card, "Sound Effects", audio.sfx_enabled, NavButton::ToggleSfx, SfxToggle);
            });

            root.spawn((
                NavButton::Back, Button,
                Node {
                    width: Val::Px(180.0), height: Val::Px(46.0), margin: UiRect::top(Val::Px(32.0)),
                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.07)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.30)),
            ))
            .with_children(|b| {
                b.spawn((Text::new("◀  BACK"), TextFont { font_size: 16.0, ..default() }, TextColor(colors::TEXT_MUTED)));
            });
        });
}

fn spawn_toggle_row<M: Component>(
    parent:  &mut ChildSpawnerCommands,
    label:   &str,
    enabled: bool,
    action:  NavButton,
    marker:  M,
) {
    parent.spawn((
        Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween, ..default() },
    ))
    .with_children(|row| {
        row.spawn((Text::new(label), TextFont { font_size: 16.0, ..default() }, TextColor(colors::WHITE)));

        row.spawn((
            action, marker, Button,
            Node {
                width: Val::Px(70.0), height: Val::Px(32.0),
                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(if enabled { colors::GREEN_MID } else { Color::srgba(0.15, 0.05, 0.05, 1.0) }),
            BorderColor::all(if enabled { Color::srgba(0.40, 0.94, 0.40, 0.50) } else { Color::srgba(0.94, 0.40, 0.40, 0.50) }),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(if enabled { "ON" } else { "OFF" }),
                TextFont { font_size: 14.0, ..default() },
                TextColor(if enabled { colors::GREEN_LIGHT } else { colors::RED_LIGHT }),
            ));
        });
    });
}

// ── Systems ───────────────────────────────────────────────────

fn handle_nav_and_toggles(
    interaction_q: Query<(&Interaction, &NavButton), (Changed<Interaction>, With<Button>)>,
    mut next:  ResMut<NextState<GameState>>,
    mut audio: ResMut<AudioSettings>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction == Interaction::Pressed {
            match button {
                NavButton::Back        => next.set(GameState::MainMenu),
                NavButton::ToggleMusic => audio.music_enabled = !audio.music_enabled,
                NavButton::ToggleSfx   => audio.sfx_enabled   = !audio.sfx_enabled,
            }
        }
    }
}

/// Handles the +/− volume nudge buttons — previously both mistakenly
/// wired to Back; now correctly adjusts master_volume by ±5%.
fn handle_volume_buttons(
    interaction_q: Query<(&Interaction, &VolumeButton), (Changed<Interaction>, With<Button>)>,
    mut audio: ResMut<AudioSettings>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction != Interaction::Pressed { continue; }
        match button {
            VolumeButton::Down => audio.master_volume = (audio.master_volume - 0.05).max(0.0),
            VolumeButton::Up   => audio.master_volume = (audio.master_volume + 0.05).min(1.0),
        }
    }
}

/// Keeps the track fill width and percentage label in sync every frame.
fn repaint_volume(
    audio: Res<AudioSettings>,
    mut fill_q:  Query<&mut Node, With<VolumeTrackFill>>,
    mut label_q: Query<&mut Text, With<VolumeLabel>>,
) {
    if !audio.is_changed() { return; }
    for mut node in &mut fill_q {
        node.width = Val::Percent(audio.master_volume * 100.0);
    }
    for mut text in &mut label_q {
        *text = Text::new(format!("Master Volume  {:>3}%", (audio.master_volume * 100.0) as u32));
    }
}