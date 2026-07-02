// =============================================================
//  DULAHI 3D – Mode Select Screen
//  File: src/ui/mode_select.rs
//
//  Player chooses Single Player (vs AI) or Multiplayer (LAN).
//  Single Player → GameSelect
//  Multiplayer   → Lobby
// =============================================================

use bevy::prelude::*;
use crate::ui::{GameState, PlayMode, colors, despawn_screen};

// ── Public plugin ─────────────────────────────────────────────

pub struct ModeSelectPlugin;

impl Plugin for ModeSelectPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::ModeSelect),  spawn_mode_select)
            .add_systems(
                Update,
                handle_mode_buttons.run_if(in_state(GameState::ModeSelect)),
            )
            .add_systems(OnExit(GameState::ModeSelect), despawn_screen::<ModeSelectRoot>);
    }
}

// ── Markers ───────────────────────────────────────────────────

#[derive(Component)]
pub struct ModeSelectRoot;

#[derive(Component, Clone, Copy)]
enum ModeButton {
    SinglePlayer,
    Multiplayer,
    Back,
}

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_mode_select(mut commands: Commands) {
    commands
        .spawn((
            ModeSelectRoot,
            Node {
                width:           Val::Percent(100.0),
                height:          Val::Percent(100.0),
                flex_direction:  FlexDirection::Column,
                align_items:     AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.07, 0.10, 0.97)),
            ZIndex(20),
        ))
        .with_children(|root| {

            // ── Title ──
            root.spawn((
                Text::new("SELECT MODE"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(colors::GOLD),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            root.spawn((
                Text::new("Choose how you want to play"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(colors::TEXT_MUTED),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // ── Mode cards row ──
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap:     Val::Px(28.0),
                    margin:         UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ))
            .with_children(|row| {

                // Single Player card
                spawn_mode_card(
                    row,
                    ModeButton::SinglePlayer,
                    "SINGLE PLAYER",
                    "Play vs AI opponents.\nChoose any Larong Lahi game\nand set your difficulty.",
                    "◈",
                    colors::GREEN_MID,
                    colors::GREEN_LIGHT,
                    Color::srgba(0.40, 0.94, 0.40, 0.35),
                );

                // Multiplayer card
                spawn_mode_card(
                    row,
                    ModeButton::Multiplayer,
                    "MULTIPLAYER",
                    "Play with friends over LAN.\nHost or join a room on the\nsame Wi-Fi network.",
                    "⛃",
                    colors::BLUE_DARK,
                    colors::BLUE_LIGHT,
                    Color::srgba(0.40, 0.40, 0.94, 0.35),
                );
            });

            // ── Back button ──
            root.spawn((
                ModeButton::Back,
                Button,
                Node {
                    width:           Val::Px(160.0),
                    height:          Val::Px(42.0),
                    justify_content: JustifyContent::Center,
                    align_items:     AlignItems::Center,
                    border:          UiRect::all(Val::Px(1.0)),
                    border_radius:   BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("◀  BACK"),
                    TextFont  { font_size: 15.0, ..default() },
                    TextColor(colors::TEXT_MUTED),
                ));
            });
        });
}

#[allow(clippy::too_many_arguments)]
fn spawn_mode_card(
    parent:     &mut ChildSpawnerCommands,
    action:     ModeButton,
    title:      &str,
    description: &str,
    icon:       &str,
    bg:         Color,
    fg:         Color,
    border_col: Color,
) {
    parent
        .spawn((
            action,
            Button,
            Node {
                flex_direction:  FlexDirection::Column,
                align_items:     AlignItems::Center,
                justify_content: JustifyContent::Center,
                width:           Val::Px(220.0),
                height:          Val::Px(260.0),
                padding:         UiRect::all(Val::Px(24.0)),
                border:          UiRect::all(Val::Px(1.5)),
                row_gap:         Val::Px(14.0),
                border_radius:   BorderRadius::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::all(border_col),
        ))
        .with_children(|card| {
            // Icon
            card.spawn((
                Text::new(icon),
                TextFont  { font_size: 48.0, ..default() },
                TextColor(fg),
            ));

            // Title
            card.spawn((
                Text::new(title),
                TextFont  { font_size: 18.0, ..default() },
                TextColor(fg),
                TextLayout::new_with_justify(Justify::Center),
            ));

            // Divider
            card.spawn((
                Node {
                    width:  Val::Percent(70.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(
                    fg.to_srgba().red,
                    fg.to_srgba().green,
                    fg.to_srgba().blue,
                    0.25,
                )),
            ));

            // Description
            card.spawn((
                Text::new(description),
                TextFont  { font_size: 13.0, ..default() },
                TextColor(Color::srgba(
                    fg.to_srgba().red,
                    fg.to_srgba().green,
                    fg.to_srgba().blue,
                    0.75,
                )),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

// ── Systems ───────────────────────────────────────────────────

fn handle_mode_buttons(
    mut interaction_q: Query<
        (&Interaction, &ModeButton, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next:  ResMut<NextState<GameState>>,
    mut mode:  ResMut<PlayMode>,
) {
    for (interaction, button, mut bg, mut border) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => match button {
                ModeButton::SinglePlayer => {
                    *mode = PlayMode::SinglePlayer;
                    next.set(GameState::GameSelect);
                }
                ModeButton::Multiplayer => {
                    *mode = PlayMode::Multiplayer;
                    next.set(GameState::Lobby);
                }
                ModeButton::Back => next.set(GameState::MainMenu),
            },
            Interaction::Hovered => {
                let b = bg.0.to_srgba();
                *bg = BackgroundColor(Color::srgba(
                    (b.red + 0.06).min(1.0),
                    (b.green + 0.06).min(1.0),
                    (b.blue + 0.06).min(1.0),
                    b.alpha,
                ));
                let c = border.top.to_srgba();
                *border = BorderColor::all(Color::srgba(c.red, c.green, c.blue, 0.85));
            }
            Interaction::None => {
                let c = border.top.to_srgba();
                *border = BorderColor::all(Color::srgba(c.red, c.green, c.blue, 0.35));
            }
        }
    }
}