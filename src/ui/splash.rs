// =============================================================
//  DULAHI 3D – Splash / App Loading Screen
//  File: src/ui/splash.rs
//
//  Shown ONCE when the application is first launched.
//  Displays the game logo, a loading bar that fills over time,
//  then transitions automatically to MainMenu.
//
//  This screen is NOT the per-game loading screen (see loading.rs).
// =============================================================

use bevy::prelude::*;
use crate::ui::{GameState, colors, despawn_screen};

// ── Public plugin ─────────────────────────────────────────────

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SplashTimer::default())
            .add_systems(OnEnter(GameState::Splash), spawn_splash)
            .add_systems(Update, tick_splash.run_if(in_state(GameState::Splash)))
            .add_systems(Update, animate_logo.run_if(in_state(GameState::Splash)))
            .add_systems(Update, animate_bar.run_if(in_state(GameState::Splash)))
            .add_systems(OnExit(GameState::Splash), despawn_screen::<SplashRoot>);
    }
}

// ── Resources ─────────────────────────────────────────────────

/// How long (seconds) the splash screen is shown before auto-advancing.
const SPLASH_DURATION: f32 = 3.2;

#[derive(Resource)]
struct SplashTimer {
    elapsed: f32,
}
impl Default for SplashTimer {
    fn default() -> Self { Self { elapsed: 0.0 } }
}

// ── Marker components ─────────────────────────────────────────

/// Root of every splash entity — used for bulk cleanup on exit.
#[derive(Component)]
pub struct SplashRoot;

/// Animated logo text that fades in.
#[derive(Component)]
struct LogoText;

/// Loading bar fill node.
#[derive(Component)]
struct BarFill;

/// Tagline that slides up after logo fades in.
#[derive(Component)]
struct TaglineText;

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_splash(mut commands: Commands) {
    // ── Full-screen dark background ──
    commands
        .spawn((
            SplashRoot,
            Node {
                width:           Val::Percent(100.0),
                height:          Val::Percent(100.0),
                flex_direction:  FlexDirection::Column,
                align_items:     AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(colors::BG_DARK),
            ZIndex(100),
        ))
        .with_children(|root| {

            // ── Decorative top strip (Filipino pattern) ──
            root.spawn((
                Node {
                    width:  Val::Percent(100.0),
                    height: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(colors::GOLD),
            ));

            // ── Spacer ──
            root.spawn(Node {
                height: Val::Px(60.0),
                ..default()
            });

            // ── DULAHI 3D logo ──
            root.spawn((
                LogoText,
                Text::new("DULAHI 3D"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(colors::GOLD),
                TextLayout::new_with_justify(Justify::Center),
                // start invisible; fades in via animate_logo
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            // ── Tagline ──
            root.spawn((
                TaglineText,
                Text::new("LARONG LAHI · DIGITAL EDITION"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),   // hidden initially
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // ── Loading bar container ──
            root.spawn((
                Node {
                    width:  Val::Px(480.0),
                    height: Val::Px(6.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.10)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.25)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    BarFill,
                    Node {
                        width:  Val::Percent(0.0),   // animated in tick_splash
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(colors::GOLD),
                ));
            });

            // ── Loading label ──
            root.spawn((
                Text::new("Loading…"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(colors::TEXT_MUTED),
                Node {
                    margin: UiRect::top(Val::Px(14.0)),
                    ..default()
                },
            ));

            // ── Bottom spacer ──
            root.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // ── Bottom pattern strip ──
            root.spawn((
                Node {
                    width:  Val::Percent(100.0),
                    height: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(colors::GOLD),
            ));
        });
}

// ── Systems ───────────────────────────────────────────────────

/// Advances the timer and transitions to MainMenu when done.
fn tick_splash(
    time:          Res<Time>,
    mut timer:     ResMut<SplashTimer>,
    current_state:  Res<State<GameState>>,
    mut next:       ResMut<NextState<GameState>>,
) {
    timer.elapsed += time.delta_secs();
    if timer.elapsed >= SPLASH_DURATION && *current_state.get() == GameState::Splash {
        next.set(GameState::MainMenu);
    }
}

/// Fades the logo in during the first second; fades tagline in after.
fn animate_logo(
    timer: Res<SplashTimer>,
    mut logo_q:    Query<&mut TextColor, (With<LogoText>,    Without<TaglineText>)>,
    mut tagline_q: Query<&mut TextColor, (With<TaglineText>, Without<LogoText>)>,
) {
    // Logo: fade in 0-0.8 s
    let logo_alpha = (timer.elapsed / 0.8).clamp(0.0, 1.0);
    for mut tc in &mut logo_q {
        let c = colors::GOLD;
        *tc = TextColor(Color::srgba(c.to_srgba().red, c.to_srgba().green, c.to_srgba().blue, logo_alpha));
    }

    // Tagline: fade in 1.0-1.8 s
    let tag_alpha = ((timer.elapsed - 1.0) / 0.8).clamp(0.0, 1.0);
    for mut tc in &mut tagline_q {
        *tc = TextColor(Color::srgba(0.60, 0.60, 0.60, tag_alpha));
    }
}

/// Grows the loading bar from 0 → 100 % proportionally to the timer.
fn animate_bar(
    timer:   Res<SplashTimer>,
    mut bar: Query<&mut Node, With<BarFill>>,
) {
    let progress = (timer.elapsed / SPLASH_DURATION).clamp(0.0, 1.0) * 100.0;
    for mut node in &mut bar {
        node.width = Val::Percent(progress);
    }
}