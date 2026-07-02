// =============================================================
//  DULAHI 3D – Main Menu Screen
//  File: src/ui/main_menu.rs
//
//  Title / navigation hub. Routes to Avatar, Shop, Options,
//  ModeSelect, and Quit. Decorative low-poly night scene (moon,
//  twinkling stars, ground plane) gives the screen visual depth.
// =============================================================

use bevy::prelude::*;
use crate::ui::{GameState, colors, despawn_screen};

// ── Public plugin ─────────────────────────────────────────────

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(
                Update,
                (handle_menu_buttons, animate_title, animate_stars)
                    .run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), despawn_screen::<MainMenuRoot>);
    }
}

// ── Markers ───────────────────────────────────────────────────

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
struct MainMenuTitle;

#[derive(Component)]
struct StarNode { phase: f32 }

#[derive(Component, Clone, Copy)]
enum MenuButton { Play, Avatar, Shop, Options, Quit }

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_main_menu(mut commands: Commands) {
    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(colors::BG_DARK),
            ZIndex(10),
        ))
        .with_children(|root| {

            // ══════════════════════════════════════════════════
            //  LEFT PANEL – decorative night scene + title
            // ══════════════════════════════════════════════════
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                    flex_grow: 1.0, padding: UiRect::all(Val::Px(32.0)),
                    ..default()
                },
            ))
            .with_children(|left| {

                // Stars (twinkling)
                let star_data: &[(f32, f32, f32, f32)] = &[
                    (10.0,  8.0, 3.0, 0.0), (24.0, 14.0, 2.0, 0.7), (37.0,  6.0, 4.0, 1.3),
                    (54.0, 18.0, 2.5, 2.1), (69.0, 10.0, 3.0, 0.4), ( 8.0, 28.0, 2.0, 1.8),
                    (47.0, 32.0, 2.5, 0.9), (80.0, 26.0, 2.0, 2.5), (20.0, 52.0, 3.0, 1.1),
                    (64.0, 58.0, 2.0, 1.6), (10.0, 72.0, 2.5, 0.3), (85.0, 66.0, 3.0, 2.0),
                    (40.0, 78.0, 2.0, 0.6), (58.0, 42.0, 2.0, 1.4), (30.0, 22.0, 2.5, 0.2),
                ];
                for &(l, t, sz, phase) in star_data {
                    left.spawn((
                        StarNode { phase },
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(l), top: Val::Percent(t),
                            width: Val::Px(sz), height: Val::Px(sz),
                            border_radius: BorderRadius::all(Val::Percent(50.0)),
                            ..default()
                        },
                        BackgroundColor(Color::WHITE),
                    ));
                }

                // Ground plane
                left.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0), left: Val::Px(0.0), right: Val::Px(0.0),
                        height: Val::Percent(26.0),
                        ..default()
                    },
                    BackgroundColor(colors::GREEN_DARK),
                ));
                // Second ground layer for depth
                left.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0), left: Val::Px(0.0), right: Val::Px(0.0),
                        height: Val::Percent(14.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.08, 0.02, 0.90)),
                ));

                // Moon
                left.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Percent(16.0), top: Val::Percent(10.0),
                        width: Val::Px(60.0), height: Val::Px(60.0),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.96, 0.90, 0.75)),
                ));
                // Moon crater shading (small offset circle)
                left.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Percent(14.5), top: Val::Percent(9.0),
                        width: Val::Px(46.0), height: Val::Px(46.0),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(colors::BG_DARK),
                ));

                // Title block sits above ground/stars via normal flow (not absolute)
                left.spawn((
                    Node { flex_grow: 1.0, ..default() },
                ));
                left.spawn((
                    MainMenuTitle,
                    Text::new("DULAHI 3D"),
                    TextFont { font_size: 62.0, ..default() },
                    TextColor(colors::GOLD),
                    Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
                ));
                left.spawn((
                    Text::new("LARONG LAHI · DIGITAL EDITION"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(colors::TEXT_MUTED),
                    Node { margin: UiRect::bottom(Val::Px(28.0)), ..default() },
                ));
                left.spawn((
                    Node { width: Val::Px(180.0), height: Val::Px(1.0), ..default() },
                    BackgroundColor(Color::srgba(0.91, 0.79, 0.48, 0.35)),
                ));
                left.spawn((
                    Node { flex_grow: 1.0, ..default() },
                ));
            });

            // ══════════════════════════════════════════════════
            //  RIGHT PANEL – menu buttons
            // ══════════════════════════════════════════════════
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column, align_items: AlignItems::Stretch,
                    justify_content: JustifyContent::Center, width: Val::Px(280.0),
                    padding: UiRect { top: Val::Px(32.0), bottom: Val::Px(32.0), left: Val::Px(24.0), right: Val::Px(40.0) },
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ))
            .with_children(|right| {
                let buttons: [(MenuButton, &str, Color, Color, &str); 5] = [
                    (MenuButton::Play,    "PLAY",    colors::GREEN_MID,   colors::GREEN_LIGHT,   "▶"),
                    (MenuButton::Avatar,  "AVATAR",  colors::PURPLE_DARK, colors::PURPLE_LIGHT,  "◈"),
                    (MenuButton::Shop,    "SHOP",    colors::AMBER_DARK,  colors::AMBER_LIGHT,   "⛁"),
                    (MenuButton::Options, "OPTIONS", colors::BLUE_DARK,   colors::BLUE_LIGHT,    "⚙"),
                    (MenuButton::Quit,    "QUIT",    colors::RED_DARK,    colors::RED_LIGHT,     "✕"),
                ];

                for (action, label, bg, fg, icon) in buttons {
                    right.spawn((
                        action, Button,
                        Node {
                            width: Val::Percent(100.0), height: Val::Px(50.0),
                            justify_content: JustifyContent::FlexStart, align_items: AlignItems::Center,
                            padding: UiRect { left: Val::Px(20.0), right: Val::Px(20.0), top: Val::Px(0.0), bottom: Val::Px(0.0) },
                            border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderColor::all(Color::srgba(fg.to_srgba().red, fg.to_srgba().green, fg.to_srgba().blue, 0.35)),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new(format!("{}  ", icon)), TextFont { font_size: 17.0, ..default() }, TextColor(fg)));
                        b.spawn((Text::new(label), TextFont { font_size: 17.0, ..default() }, TextColor(fg)));
                    });
                }

                right.spawn((
                    Text::new("v0.1.0"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                    Node { margin: UiRect::top(Val::Px(10.0)), ..default() },
                ));
            });
        });
}

// ── Systems ───────────────────────────────────────────────────

fn handle_menu_buttons(
    mut interaction_q: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button, mut bg, mut border) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => match button {
                MenuButton::Play    => next.set(GameState::ModeSelect),
                MenuButton::Avatar  => next.set(GameState::Avatar),
                MenuButton::Shop    => next.set(GameState::Shop),
                MenuButton::Options => next.set(GameState::Options),
                MenuButton::Quit    => { exit.write(AppExit::Success); }
            },
            Interaction::Hovered => {
                let b = bg.0.to_srgba();
                *bg = BackgroundColor(Color::srgba((b.red + 0.08).min(1.0), (b.green + 0.08).min(1.0), (b.blue + 0.08).min(1.0), b.alpha));
                let c = border.top.to_srgba();
                *border = BorderColor::all(Color::srgba(c.red, c.green, c.blue, 0.75));
            }
            Interaction::None => {
                let c = border.top.to_srgba();
                *border = BorderColor::all(Color::srgba(c.red, c.green, c.blue, 0.35));
            }
        }
    }
}

fn animate_title(time: Res<Time>, mut q: Query<&mut TextColor, With<MainMenuTitle>>) {
    let pulse = (time.elapsed_secs() * 1.4).sin() * 0.07 + 0.93;
    let c = colors::GOLD.to_srgba();
    for mut tc in &mut q {
        *tc = TextColor(Color::srgba((c.red * pulse).min(1.0), (c.green * pulse).min(1.0), (c.blue * pulse).min(1.0), 1.0));
    }
}

fn animate_stars(time: Res<Time>, mut q: Query<(&StarNode, &mut BackgroundColor)>) {
    let t = time.elapsed_secs();
    for (star, mut bg) in &mut q {
        let alpha = ((t * 0.9 + star.phase).sin() * 0.4 + 0.55).clamp(0.0, 1.0);
        *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, alpha));
    }
}