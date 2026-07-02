// =============================================================
//  DULAHI 3D – Lobby Screen (LAN Multiplayer)
//  File: src/ui/lobby.rs
//
//  Multiplayer path: host or join a room, pick a game and team
//  format (1v1 / 2v2 / 3v3 — filtered to what the game supports),
//  then fill both teams. Empty slots can be filled with AI by the
//  host at any time. Start requires every slot on both teams to
//  be occupied (Human or AI) — exactly matching the chosen format.
//
//  Note: actual networking (renet/bevy_renet) is wired up in the
//  network plugin. This file manages ONLY the UI layer; a real
//  join currently has no way to occupy a slot except AI-fill.
// =============================================================

use bevy::prelude::*;
use crate::ui::{
    GameState, LahiGame, SelectedGame, SelectedFormat, MatchFormat,
    Difficulty, TeamSide, colors, despawn_screen,
};

// ── Public plugin ─────────────────────────────────────────────

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LobbyState>()
            .add_systems(OnEnter(GameState::Lobby),  spawn_lobby)
            .add_systems(
                Update,
                (handle_room_buttons, handle_game_picker, handle_format_buttons,
                 handle_slot_buttons, handle_nav_buttons, tick_dots,
                 repaint_game_label, repaint_format_buttons, repaint_start_button)
                    .run_if(in_state(GameState::Lobby)),
            )
            .add_systems(OnExit(GameState::Lobby), despawn_screen::<LobbyRoot>);
    }
}

// ── Slot model ───────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotKind {
    Empty,
    You,
    Ai(Difficulty),
}

#[derive(Resource)]
pub struct LobbyState {
    is_host:    bool,
    game_idx:   usize,
    format:     MatchFormat,
    team_a:     Vec<SlotKind>,
    team_b:     Vec<SlotKind>,
    dot_timer:  f32,
    dot_frame:  usize,
}

impl Default for LobbyState {
    fn default() -> Self {
        let game = LahiGame::all()[0];
        let format = game.available_formats().first().copied().unwrap_or(MatchFormat::OneVOne);
        let mut state = LobbyState {
            is_host: true,
            game_idx: 0,
            format,
            team_a: vec![],
            team_b: vec![],
            dot_timer: 0.0,
            dot_frame: 0,
        };
        state.resize_teams();
        state
    }
}

impl LobbyState {
    fn resize_teams(&mut self) {
        let size = self.format.team_size();
        self.team_a = vec![SlotKind::Empty; size];
        self.team_a[0] = SlotKind::You;
        self.team_b = vec![SlotKind::Empty; size];
    }

    fn ready_to_start(&self) -> bool {
        self.team_a.iter().all(|s| *s != SlotKind::Empty)
            && self.team_b.iter().all(|s| *s != SlotKind::Empty)
    }
}

// ── Markers ───────────────────────────────────────────────────

#[derive(Component)]
pub struct LobbyRoot;

#[derive(Component, Clone, Copy)]
enum RoomButton { Host, Join }

#[derive(Component, Clone, Copy)]
enum GamePickerButton { Prev, Next }

#[derive(Component)]
struct GameNameLabel;

#[derive(Component)]
struct FormatButton(MatchFormat);

#[derive(Component)]
struct TeamsPanel;

#[derive(Component)]
struct StatusLabel;

#[derive(Component, Clone, Copy)]
enum SlotAction { AddAi, CycleAi, RemoveAi }

#[derive(Component, Clone, Copy)]
struct SlotButton { side: TeamSide, index: usize, action: SlotAction }

#[derive(Component)]
enum NavButton { Start, Back }
#[derive(Component)]
struct StartButton;
#[derive(Component)]
struct StartButtonLabel;

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_lobby(mut commands: Commands, mut lobby: ResMut<LobbyState>) {
    *lobby = LobbyState::default();
    let game = LahiGame::all()[lobby.game_idx];

    commands
        .spawn((
            LobbyRoot,
            Node {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.07, 0.10, 0.97)),
            ZIndex(20),
        ))
        .with_children(|root| {

            // ── Header row ──
            root.spawn((
                Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() },
            ))
            .with_children(|hdr| {
                hdr.spawn((
                    Text::new("MULTIPLAYER LOBBY"),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(colors::GOLD),
                ));

                hdr.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() })
                    .with_children(|btns| {
                        for (action, label) in [(RoomButton::Host, "HOST ROOM"), (RoomButton::Join, "JOIN ROOM")] {
                            btns.spawn((
                                action, Button,
                                Node {
                                    width: Val::Px(130.0), height: Val::Px(38.0),
                                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                            ))
                            .with_children(|b| {
                                b.spawn((Text::new(label), TextFont { font_size: 12.0, ..default() }, TextColor(colors::WHITE)));
                            });
                        }
                    });
            });

            // ── Status ──
            root.spawn((
                StatusLabel,
                Text::new("Hosting — waiting for players. Fill empty slots with AI if needed."),
                TextFont { font_size: 12.0, ..default() },
                TextColor(colors::TEXT_MUTED),
            ));

            // ── Game + format picker row ──
            root.spawn((
                Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(24.0), align_items: AlignItems::Center, ..default() },
            ))
            .with_children(|row| {
                // Game picker
                row.spawn((
                    Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(10.0), ..default() },
                ))
                .with_children(|picker| {
                    picker.spawn((
                        GamePickerButton::Prev, Button,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                    )).with_children(|b| { b.spawn((Text::new("◀"), TextFont { font_size: 15.0, ..default() }, TextColor(colors::WHITE))); });

                    picker.spawn((
                        GameNameLabel,
                        Text::new(game.display_name()),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(colors::WHITE),
                        Node { width: Val::Px(140.0), ..default() },
                    ));

                    picker.spawn((
                        GamePickerButton::Next, Button,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                    )).with_children(|b| { b.spawn((Text::new("▶"), TextFont { font_size: 15.0, ..default() }, TextColor(colors::WHITE))); });
                });

                // Format row
                row.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(8.0), ..default() })
                    .with_children(|fmt_row| {
                        for f in game.available_formats() {
                            let selected = f == lobby.format;
                            fmt_row.spawn((
                                FormatButton(f), Button,
                                Node { width: Val::Px(70.0), height: Val::Px(32.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
                                BackgroundColor(if selected { colors::GREEN_MID } else { Color::srgba(0.10, 0.10, 0.10, 0.80) }),
                                BorderColor::all(if selected { Color::srgba(0.40, 0.94, 0.40, 0.60) } else { Color::srgba(1.0, 1.0, 1.0, 0.15) }),
                            )).with_children(|b| {
                                b.spawn((Text::new(f.label()), TextFont { font_size: 13.0, ..default() }, TextColor(if selected { colors::GREEN_LIGHT } else { colors::TEXT_MUTED })));
                            });
                        }
                    });
            });

            // ── Teams panel (rebuilt on any team/format/game change) ──
            root.spawn((
                TeamsPanel,
                Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(20.0), flex_grow: 1.0, ..default() },
            ))
            .with_children(|panel| {
                spawn_team_column(panel, TeamSide::A, &lobby.team_a);
                spawn_team_column(panel, TeamSide::B, &lobby.team_b);
            });

            // ── Nav row ──
            root.spawn((
                Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() },
            ))
            .with_children(|nav| {
                nav.spawn((
                    NavButton::Back, Button,
                    Node { width: Val::Px(130.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.20)),
                )).with_children(|b| { b.spawn((Text::new("◀  BACK"), TextFont { font_size: 14.0, ..default() }, TextColor(colors::TEXT_MUTED))); });

                nav.spawn((
                    NavButton::Start, StartButton, Button,
                    Node { width: Val::Px(180.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() },
                    BackgroundColor(Color::srgba(0.10, 0.30, 0.10, 0.40)),
                    BorderColor::all(Color::srgba(0.40, 0.94, 0.40, 0.15)),
                )).with_children(|b| {
                    b.spawn((StartButtonLabel, Text::new("START MATCH  ▶"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgba(0.40, 0.94, 0.40, 0.30))));
                });
            });
        });
}

fn spawn_team_column(parent: &mut ChildSpawnerCommands, side: TeamSide, slots: &[SlotKind]) {
    let (bg, fg) = match side {
        TeamSide::A => (colors::GREEN_MID, colors::GREEN_LIGHT),
        TeamSide::B => (colors::RED_DARK, colors::RED_LIGHT),
    };
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
        BackgroundColor(Color::srgba(bg.to_srgba().red, bg.to_srgba().green, bg.to_srgba().blue, 0.15)),
        BorderColor::all(Color::srgba(fg.to_srgba().red, fg.to_srgba().green, fg.to_srgba().blue, 0.35)),
    ))
    .with_children(|col| {
        col.spawn((Text::new(side.label().to_uppercase()), TextFont { font_size: 14.0, ..default() }, TextColor(fg)));

        for (i, slot) in slots.iter().enumerate() {
            col.spawn((
                Node {
                    flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    height: Val::Px(42.0), padding: UiRect::horizontal(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.06, 0.08, 0.80)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
            ))
            .with_children(|row| {
                match slot {
                    SlotKind::You => {
                        row.spawn((Text::new("★ You"), TextFont { font_size: 13.0, ..default() }, TextColor(colors::GOLD)));
                        row.spawn((Text::new("Host"), TextFont { font_size: 11.0, ..default() }, TextColor(colors::TEXT_MUTED)));
                    }
                    SlotKind::Empty => {
                        row.spawn((Text::new("— Empty —"), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgba(1.0, 1.0, 1.0, 0.25))));
                        row.spawn((
                            SlotButton { side, index: i, action: SlotAction::AddAi }, Button,
                            Node { padding: UiRect { left: Val::Px(10.0), right: Val::Px(10.0), top: Val::Px(4.0), bottom: Val::Px(4.0) }, border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
                            BackgroundColor(colors::AMBER_DARK),
                            BorderColor::all(Color::srgba(0.96, 0.78, 0.40, 0.40)),
                        )).with_children(|b| { b.spawn((Text::new("+ ADD AI"), TextFont { font_size: 10.0, ..default() }, TextColor(colors::AMBER_LIGHT))); });
                    }
                    SlotKind::Ai(diff) => {
                        row.spawn((Text::new(format!("AI Bot")), TextFont { font_size: 13.0, ..default() }, TextColor(colors::WHITE)));
                        row.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(6.0), ..default() })
                            .with_children(|btns| {
                                btns.spawn((
                                    SlotButton { side, index: i, action: SlotAction::CycleAi }, Button,
                                    Node { padding: UiRect { left: Val::Px(8.0), right: Val::Px(8.0), top: Val::Px(4.0), bottom: Val::Px(4.0) }, border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
                                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                                )).with_children(|b| { b.spawn((Text::new(diff.label()), TextFont { font_size: 10.0, ..default() }, TextColor(colors::TEXT_MUTED))); });

                                btns.spawn((
                                    SlotButton { side, index: i, action: SlotAction::RemoveAi }, Button,
                                    Node { width: Val::Px(24.0), height: Val::Px(24.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
                                    BackgroundColor(Color::srgba(0.30, 0.06, 0.06, 0.80)),
                                    BorderColor::all(Color::srgba(0.94, 0.40, 0.40, 0.40)),
                                )).with_children(|b| { b.spawn((Text::new("✕"), TextFont { font_size: 11.0, ..default() }, TextColor(colors::RED_LIGHT))); });
                            });
                    }
                }
            });
        }
    });
}

fn rebuild_teams_panel(commands: &mut Commands, panel_q: &Query<Entity, With<TeamsPanel>>, lobby: &LobbyState) {
    if let Ok(panel_entity) = panel_q.single() {
        commands.entity(panel_entity).despawn_related::<Children>();
        let team_a = lobby.team_a.clone();
        let team_b = lobby.team_b.clone();
        commands.entity(panel_entity).with_children(|panel| {
            spawn_team_column(panel, TeamSide::A, &team_a);
            spawn_team_column(panel, TeamSide::B, &team_b);
        });
    }
}

// ── Systems ───────────────────────────────────────────────────

fn handle_room_buttons(
    interaction_q: Query<(&Interaction, &RoomButton), (Changed<Interaction>, With<Button>)>,
    mut lobby: ResMut<LobbyState>,
    mut status_q: Query<&mut Text, With<StatusLabel>>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction != Interaction::Pressed { continue; }
        lobby.is_host = matches!(button, RoomButton::Host);
        let msg = if lobby.is_host {
            "Hosting — waiting for players. Fill empty slots with AI if needed."
        } else {
            "Scanning for rooms on your Wi-Fi network..."
        };
        for mut t in &mut status_q { *t = Text::new(msg); }
    }
}

fn handle_game_picker(
    interaction_q: Query<(&Interaction, &GamePickerButton), (Changed<Interaction>, With<Button>)>,
    mut lobby:     ResMut<LobbyState>,
    mut commands:  Commands,
    panel_q:       Query<Entity, With<TeamsPanel>>,
) {
    let mut moved = false;
    for (interaction, button) in &interaction_q {
        if *interaction != Interaction::Pressed { continue; }
        let n = LahiGame::all().len();
        match button {
            GamePickerButton::Prev => lobby.game_idx = (lobby.game_idx + n - 1) % n,
            GamePickerButton::Next => lobby.game_idx = (lobby.game_idx + 1) % n,
        }
        moved = true;
    }
    if moved {
        let game = LahiGame::all()[lobby.game_idx];
        lobby.format = game.available_formats().first().copied().unwrap_or(MatchFormat::OneVOne);
        lobby.resize_teams();
        rebuild_teams_panel(&mut commands, &panel_q, &lobby);
    }
}

fn handle_format_buttons(
    interaction_q: Query<(&Interaction, &FormatButton), (Changed<Interaction>, With<Button>)>,
    mut lobby:     ResMut<LobbyState>,
    mut commands:  Commands,
    panel_q:       Query<Entity, With<TeamsPanel>>,
) {
    let mut new_format = None;
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed { new_format = Some(btn.0); }
    }
    if let Some(format) = new_format {
        lobby.format = format;
        lobby.resize_teams();
        rebuild_teams_panel(&mut commands, &panel_q, &lobby);
    }
}

fn handle_slot_buttons(
    interaction_q: Query<(&Interaction, &SlotButton), (Changed<Interaction>, With<Button>)>,
    mut lobby:     ResMut<LobbyState>,
    mut commands:  Commands,
    panel_q:       Query<Entity, With<TeamsPanel>>,
) {
    let mut touched = false;
    for (interaction, btn) in &interaction_q {
        if *interaction != Interaction::Pressed { continue; }
        let slots = match btn.side { TeamSide::A => &mut lobby.team_a, TeamSide::B => &mut lobby.team_b };
        if let Some(slot) = slots.get_mut(btn.index) {
            match btn.action {
                SlotAction::AddAi    => { if *slot == SlotKind::Empty { *slot = SlotKind::Ai(Difficulty::Normal); } }
                SlotAction::CycleAi  => { if let SlotKind::Ai(d) = *slot { *slot = SlotKind::Ai(d.cycle()); } }
                SlotAction::RemoveAi => { if matches!(*slot, SlotKind::Ai(_)) { *slot = SlotKind::Empty; } }
            }
            touched = true;
        }
    }
    if touched {
        rebuild_teams_panel(&mut commands, &panel_q, &lobby);
    }
}

fn handle_nav_buttons(
    interaction_q: Query<(&Interaction, &NavButton), (Changed<Interaction>, With<Button>)>,
    mut next:  ResMut<NextState<GameState>>,
    lobby:     Res<LobbyState>,
    mut sel:   ResMut<SelectedGame>,
    mut fmt:   ResMut<SelectedFormat>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction != Interaction::Pressed { continue; }
        match button {
            NavButton::Back => next.set(GameState::ModeSelect),
            NavButton::Start => {
                if lobby.ready_to_start() {
                    sel.0 = LahiGame::all()[lobby.game_idx];
                    fmt.0 = lobby.format;
                    next.set(GameState::Loading);
                }
            }
        }
    }
}

/// Cycles the "Hosting…" / "Scanning…" dot animation.
fn tick_dots(time: Res<Time>, mut lobby: ResMut<LobbyState>) {
    lobby.dot_timer += time.delta_secs();
    if lobby.dot_timer >= 0.5 {
        lobby.dot_timer = 0.0;
        lobby.dot_frame = (lobby.dot_frame + 1) % 4;
    }
}

fn repaint_game_label(
    lobby: Res<LobbyState>,
    mut q: Query<&mut Text, With<GameNameLabel>>,
) {
    if !lobby.is_changed() { return; }
    let game = LahiGame::all()[lobby.game_idx];
    for mut t in &mut q { *t = Text::new(game.display_name()); }
}

fn repaint_format_buttons(
    lobby: Res<LobbyState>,
    mut q: Query<(&FormatButton, &mut BackgroundColor, &mut BorderColor, &Children)>,
    mut text_q: Query<&mut TextColor>,
) {
    for (btn, mut bg, mut border, children) in &mut q {
        let selected = btn.0 == lobby.format;
        *bg = BackgroundColor(if selected { colors::GREEN_MID } else { Color::srgba(0.10, 0.10, 0.10, 0.80) });
        *border = BorderColor::all(if selected { Color::srgba(0.40, 0.94, 0.40, 0.60) } else { Color::srgba(1.0, 1.0, 1.0, 0.15) });
        for child in children.iter() {
            if let Ok(mut tc) = text_q.get_mut(child) {
                *tc = TextColor(if selected { colors::GREEN_LIGHT } else { colors::TEXT_MUTED });
            }
        }
    }
}

fn repaint_start_button(
    lobby: Res<LobbyState>,
    mut btn_q: Query<(&mut BackgroundColor, &mut BorderColor), With<StartButton>>,
    mut label_q: Query<&mut TextColor, With<StartButtonLabel>>,
) {
    let ready = lobby.ready_to_start();
    for (mut bg, mut border) in &mut btn_q {
        *bg = BackgroundColor(if ready { colors::GREEN_MID } else { Color::srgba(0.10, 0.30, 0.10, 0.40) });
        *border = BorderColor::all(if ready { Color::srgba(0.40, 0.94, 0.40, 0.60) } else { Color::srgba(0.40, 0.94, 0.40, 0.15) });
    }
    for mut tc in &mut label_q {
        *tc = TextColor(if ready { colors::GREEN_LIGHT } else { Color::srgba(0.40, 0.94, 0.40, 0.30) });
    }
}