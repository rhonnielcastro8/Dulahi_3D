// =============================================================
//  DULAHI 3D – Results Screen
//  File: src/ui/results.rs
//
//  Shown after a match ends. Displays Team A vs Team B, the
//  winning side, and applies Puntos/rank rewards. Since InGame
//  isn't implemented yet, the winner is decided by a simple
//  placeholder rule that alternates across sessions — swap
//  `decide_winner` for real match data once gameplay exists.
// =============================================================

use bevy::prelude::*;
use crate::ui::{
    GameState, SelectedGame, SelectedFormat, PlayMode, TeamSide,
    Currency, RankPoints, PlayerStats, colors, despawn_screen,
};

// ── Public plugin ─────────────────────────────────────────────

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Results), (apply_rewards, spawn_results).chain())
            .add_systems(
                Update,
                (handle_results_buttons, animate_winner_pulse)
                    .run_if(in_state(GameState::Results)),
            )
            .add_systems(OnExit(GameState::Results), despawn_screen::<ResultsRoot>);
    }
}

// ── Placeholder outcome resource ────────────────────────────────

/// Set by `apply_rewards` right before the screen spawns, so the
/// UI and the reward math agree on who won.
#[derive(Resource, Default)]
struct LastMatchOutcome {
    you_won: bool,
}

// ── Markers ───────────────────────────────────────────────────

#[derive(Component)]
pub struct ResultsRoot;

#[derive(Component)]
enum ResultsButton { PlayAgain, ChangeGame, MainMenu }

#[derive(Component)]
struct WinnerLabel;

// ── Reward application ──────────────────────────────────────────

/// Decides a placeholder winner and grants Puntos/rank/stat rewards.
/// Alternates win/loss across plays using total games played so far,
/// so repeated testing shows both outcomes rather than always winning.
fn apply_rewards(
    mut commands: Commands,
    mut currency: ResMut<Currency>,
    mut rank:     ResMut<RankPoints>,
    mut stats:    ResMut<PlayerStats>,
) {
    let games_played = stats.wins + stats.losses;
    let you_won = games_played % 2 == 0;

    if you_won {
        currency.0 += 60;
        rank.0     += 25;
        stats.wins += 1;
    } else {
        currency.0 += 20;
        rank.0     += 10;
        stats.losses += 1;
    }

    commands.insert_resource(LastMatchOutcome { you_won });
}

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_results(
    mut commands:  Commands,
    selected_game: Res<SelectedGame>,
    format:        Res<SelectedFormat>,
    outcome:       Res<LastMatchOutcome>,
) {
    let game_name  = selected_game.0.display_name();
    let team_size  = format.0.team_size();
    let your_side  = TeamSide::A;
    let winner_side = if outcome.you_won { your_side } else { your_side.other() };

    let team_a_names: Vec<String> = std::iter::once("You".to_string())
        .chain((1..team_size).map(|_| "AI Teammate".to_string()))
        .collect();
    let team_b_names: Vec<String> = (0..team_size).map(|_| "AI Opponent".to_string()).collect();

    let (reward_puntos, reward_rank) = if outcome.you_won { (60, 25) } else { (20, 10) };

    commands
        .spawn((
            ResultsRoot,
            Node {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
                justify_content: JustifyContent::Center, padding: UiRect::all(Val::Px(28.0)),
                row_gap: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.06, 0.09, 0.98)),
            ZIndex(40),
        ))
        .with_children(|root| {

            root.spawn((
                Node { position_type: PositionType::Absolute, top: Val::Px(0.0), left: Val::Px(0.0), right: Val::Px(0.0), height: Val::Px(4.0), ..default() },
                BackgroundColor(colors::GOLD),
            ));

            // Game + format chip
            root.spawn((
                Node {
                    padding: UiRect { left: Val::Px(18.0), right: Val::Px(18.0), top: Val::Px(5.0), bottom: Val::Px(5.0) },
                    border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(20.0)),
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.91, 0.79, 0.48, 0.10)),
                BorderColor::all(Color::srgba(0.91, 0.79, 0.48, 0.30)),
            )).with_children(|chip| {
                chip.spawn((
                    Text::new(format!("{} · {}", game_name.to_uppercase(), format.0.label())),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(colors::GOLD),
                ));
            });

            root.spawn((
                Text::new("MATCH OVER"),
                TextFont { font_size: 42.0, ..default() },
                TextColor(colors::WHITE),
                Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
            ));

            // Winner banner
            let (banner_text, banner_col) = if outcome.you_won {
                ("🏆  YOUR TEAM WINS!", colors::GOLD)
            } else {
                ("ENEMY TEAM WINS", colors::TEXT_MUTED)
            };
            root.spawn((
                Node {
                    padding: UiRect { left: Val::Px(32.0), right: Val::Px(32.0), top: Val::Px(10.0), bottom: Val::Px(10.0) },
                    border: UiRect::all(Val::Px(1.5)), border_radius: BorderRadius::all(Val::Px(10.0)),
                    margin: UiRect { top: Val::Px(6.0), bottom: Val::Px(24.0), ..default() },
                    ..default()
                },
                BackgroundColor(Color::srgba(banner_col.to_srgba().red, banner_col.to_srgba().green, banner_col.to_srgba().blue, 0.12)),
                BorderColor::all(Color::srgba(banner_col.to_srgba().red, banner_col.to_srgba().green, banner_col.to_srgba().blue, 0.50)),
            )).with_children(|banner| {
                banner.spawn((
                    WinnerLabel,
                    Text::new(banner_text),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(banner_col),
                ));
            });

            // Team columns
            root.spawn((
                Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(20.0), margin: UiRect::bottom(Val::Px(24.0)), ..default() },
            )).with_children(|teams| {
                spawn_result_team(teams, TeamSide::A, &team_a_names, winner_side == TeamSide::A);
                spawn_result_team(teams, TeamSide::B, &team_b_names, winner_side == TeamSide::B);
            });

            // Rewards
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row, column_gap: Val::Px(24.0),
                    padding: UiRect::all(Val::Px(14.0)), border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)), margin: UiRect::bottom(Val::Px(28.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.40)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.10)),
            )).with_children(|rw| {
                rw.spawn((
                    Text::new(format!("+{} Puntos", reward_puntos)),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(colors::GOLD),
                ));
                rw.spawn((
                    Text::new(format!("+{} Rank Points", reward_rank)),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(colors::PURPLE_LIGHT),
                ));
            });

            // Action buttons
            root.spawn((
                Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() },
            )).with_children(|btns| {
                for (action, label, bg, fg, border) in [
                    (ResultsButton::PlayAgain,  "▶  PLAY AGAIN",  colors::GREEN_MID, colors::GREEN_LIGHT, Color::srgba(0.40, 0.94, 0.40, 0.45)),
                    (ResultsButton::ChangeGame, "⊞  CHANGE GAME", colors::BLUE_DARK, colors::BLUE_LIGHT, Color::srgba(0.40, 0.40, 0.94, 0.45)),
                    (ResultsButton::MainMenu,   "⌂  MAIN MENU",   Color::srgba(0.08, 0.08, 0.12, 1.0), colors::TEXT_MUTED, Color::srgba(1.0, 1.0, 1.0, 0.15)),
                ] {
                    btns.spawn((
                        action, Button,
                        Node { width: Val::Px(170.0), height: Val::Px(46.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() },
                        BackgroundColor(bg),
                        BorderColor::all(border),
                    )).with_children(|b| {
                        b.spawn((Text::new(label), TextFont { font_size: 14.0, ..default() }, TextColor(fg)));
                    });
                }
            });

            root.spawn((
                Node { position_type: PositionType::Absolute, bottom: Val::Px(0.0), left: Val::Px(0.0), right: Val::Px(0.0), height: Val::Px(4.0), ..default() },
                BackgroundColor(colors::GOLD),
            ));
        });
}

fn spawn_result_team(parent: &mut ChildSpawnerCommands, side: TeamSide, names: &[String], won: bool) {
    let (bg, fg) = match side {
        TeamSide::A => (colors::GREEN_MID, colors::GREEN_LIGHT),
        TeamSide::B => (colors::RED_DARK, colors::RED_LIGHT),
    };
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column, width: Val::Px(220.0),
            padding: UiRect::all(Val::Px(16.0)), border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)), row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(bg.to_srgba().red, bg.to_srgba().green, bg.to_srgba().blue, 0.15)),
        BorderColor::all(Color::srgba(fg.to_srgba().red, fg.to_srgba().green, fg.to_srgba().blue, 0.35)),
    )).with_children(|col| {
        col.spawn((
            Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() },
        )).with_children(|hdr| {
            hdr.spawn((Text::new(side.label().to_uppercase()), TextFont { font_size: 13.0, ..default() }, TextColor(fg)));
            if won {
                hdr.spawn((Text::new("WIN"), TextFont { font_size: 11.0, ..default() }, TextColor(colors::GOLD)));
            }
        });
        for name in names {
            col.spawn((Text::new(name.clone()), TextFont { font_size: 13.0, ..default() }, TextColor(colors::WHITE)));
        }
    });
}

// ── Systems ───────────────────────────────────────────────────

fn handle_results_buttons(
    interaction_q: Query<(&Interaction, &ResultsButton), (Changed<Interaction>, With<Button>)>,
    mut next: ResMut<NextState<GameState>>,
    mode:     Res<PlayMode>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction != Interaction::Pressed { continue; }
        match button {
            ResultsButton::PlayAgain  => next.set(GameState::Loading),
            ResultsButton::ChangeGame => {
                if *mode == PlayMode::Multiplayer { next.set(GameState::Lobby); } else { next.set(GameState::GameSelect); }
            }
            ResultsButton::MainMenu   => next.set(GameState::MainMenu),
        }
    }
}

fn animate_winner_pulse(time: Res<Time>, mut q: Query<&mut TextColor, With<WinnerLabel>>) {
    let t = time.elapsed_secs();
    let pulse = (t * 1.8).sin() * 0.06 + 0.94;
    for mut tc in &mut q {
        let c = tc.0.to_srgba();
        *tc = TextColor(Color::srgba((c.red * pulse).min(1.0), (c.green * pulse).min(1.0), (c.blue * pulse).min(1.0), c.alpha));
    }
}