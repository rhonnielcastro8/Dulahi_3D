// =============================================================
//  DULAHI 3D – Avatar Screen
//  File: src/ui/avatar.rs
//
//  View/equip owned cosmetics and check rank progress.
//  Reached from the main menu. Only OWNED items appear here —
//  browsing/buying locked items happens in shop.rs.
// =============================================================

use bevy::prelude::*;
use crate::ui::{
    GameState, LahiGame, Inventory, Equipped, RankPoints, RankTier,
    Currency, PlayerStats, SHOP_CATALOG, SkinCategory, colors, despawn_screen,
};

// ── Public plugin ─────────────────────────────────────────────

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AvatarTabState>()
            .add_systems(OnEnter(GameState::Avatar), spawn_avatar)
            .add_systems(
                Update,
                (handle_nav_buttons, handle_tab_buttons, handle_skin_buttons,
                 repaint_tabs, repaint_skin_buttons)
                    .run_if(in_state(GameState::Avatar)),
            )
            .add_systems(OnExit(GameState::Avatar), despawn_screen::<AvatarRoot>);
    }
}

// ── Local state ───────────────────────────────────────────────

/// Which game's object-skin tab is currently showing.
/// Index into `LahiGame::all()`.
#[derive(Resource, Default)]
struct AvatarTabState {
    selected_game: usize,
}

// ── Markers ───────────────────────────────────────────────────

#[derive(Component)]
pub struct AvatarRoot;

#[derive(Component)]
enum NavButton { Back, GoShop }

#[derive(Component)]
struct TabButton(usize);

/// Grid of owned object-skin buttons is rebuilt each time the tab
/// changes, so it needs its own marker to despawn/respawn cleanly.
#[derive(Component)]
struct ObjectSkinGrid;

#[derive(Component, Clone, Copy)]
struct CharacterSkinButton { id: &'static str }

#[derive(Component, Clone, Copy)]
struct ObjectSkinButton { id: &'static str, game: LahiGame }

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_avatar(
    mut commands: Commands,
    inventory:    Res<Inventory>,
    equipped:     Res<Equipped>,
    rank:         Res<RankPoints>,
    currency:     Res<Currency>,
    stats:        Res<PlayerStats>,
    mut tab:      ResMut<AvatarTabState>,
) {
    tab.selected_game = 0;
    let tier = RankTier::from_points(rank.0);

    commands
        .spawn((
            AvatarRoot,
            Node {
                width:          Val::Percent(100.0),
                height:         Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.07, 0.10, 0.97)),
            ZIndex(20),
        ))
        .with_children(|root| {

            // ═══════════════════════════════════════════
            //  LEFT PANEL – rank, currency, stats, character skin
            // ═══════════════════════════════════════════
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    width:          Val::Px(300.0),
                    padding:        UiRect::all(Val::Px(28.0)),
                    row_gap:        Val::Px(18.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ))
            .with_children(|left| {

                left.spawn((
                    Text::new("AVATAR"),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(colors::PURPLE_LIGHT),
                ));

                // Rank card
                left.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding:        UiRect::all(Val::Px(16.0)),
                        border:         UiRect::all(Val::Px(1.0)),
                        border_radius:  BorderRadius::all(Val::Px(10.0)),
                        row_gap:        Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.91, 0.79, 0.48, 0.08)),
                    BorderColor::all(Color::srgba(0.91, 0.79, 0.48, 0.30)),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new(tier.display_name().to_uppercase()),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(colors::GOLD),
                    ));
                    let progress_text = match tier.next_threshold() {
                        Some(next) => format!("{} / {} points", rank.0, next),
                        None       => format!("{} points — max rank", rank.0),
                    };
                    card.spawn((
                        Text::new(progress_text),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(colors::TEXT_MUTED),
                    ));
                });

                // Currency card
                left.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items:    AlignItems::Center,
                        column_gap:     Val::Px(10.0),
                        padding:        UiRect::all(Val::Px(14.0)),
                        border:         UiRect::all(Val::Px(1.0)),
                        border_radius:  BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.12)),
                ))
                .with_children(|card| {
                    card.spawn((
                        Node { width: Val::Px(14.0), height: Val::Px(14.0), border_radius: BorderRadius::all(Val::Percent(50.0)), ..default() },
                        BackgroundColor(colors::GOLD),
                    ));
                    card.spawn((
                        Text::new(format!("{} Puntos", currency.0)),
                        TextFont { font_size: 15.0, ..default() },
                        TextColor(colors::WHITE),
                    ));
                });

                // Stats
                left.spawn((
                    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), ..default() },
                ))
                .with_children(|s| {
                    s.spawn((
                        Text::new(format!("Wins: {}   Losses: {}", stats.wins, stats.losses)),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(colors::TEXT_MUTED),
                    ));
                    s.spawn((
                        Text::new(format!("Win rate: {:.0}%", stats.win_rate())),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(colors::TEXT_MUTED),
                    ));
                });

                // Divider
                left.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(1.0), ..default() },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                ));

                left.spawn((
                    Text::new("CHARACTER SKIN"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(colors::TEXT_MUTED),
                ));

                // Owned character skins
                for item in SHOP_CATALOG.iter().filter(|i| {
                    matches!(i.category, SkinCategory::Character) && inventory.0.contains(i.id)
                }) {
                    let is_equipped = equipped.character == item.id;
                    left.spawn((
                        CharacterSkinButton { id: item.id },
                        Button,
                        Node {
                            flex_direction:  FlexDirection::Row,
                            align_items:     AlignItems::Center,
                            column_gap:      Val::Px(10.0),
                            height:          Val::Px(44.0),
                            padding:         UiRect::horizontal(Val::Px(12.0)),
                            border:          UiRect::all(Val::Px(1.0)),
                            border_radius:   BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(if is_equipped { colors::PURPLE_DARK } else { Color::srgba(0.08, 0.08, 0.10, 0.80) }),
                        BorderColor::all(if is_equipped { Color::srgba(0.78, 0.67, 0.93, 0.60) } else { Color::srgba(1.0, 1.0, 1.0, 0.10) }),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(item.icon),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(if is_equipped { colors::PURPLE_LIGHT } else { colors::TEXT_MUTED }),
                        ));
                        b.spawn((
                            Text::new(if is_equipped { format!("{} ✓", item.name) } else { item.name.to_string() }),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(if is_equipped { colors::WHITE } else { colors::TEXT_MUTED }),
                        ));
                    });
                }
            });

            // ═══════════════════════════════════════════
            //  RIGHT PANEL – game-object skin tabs + grid
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
                    Text::new("GAME OBJECT SKINS"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(colors::PURPLE_LIGHT),
                ));

                // Tabs row
                right.spawn((
                    Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() },
                ))
                .with_children(|tabs| {
                    for (i, game) in LahiGame::all().iter().enumerate() {
                        let is_selected = i == 0;
                        tabs.spawn((
                            TabButton(i),
                            Button,
                            Node {
                                padding:         UiRect { left: Val::Px(16.0), right: Val::Px(16.0), top: Val::Px(8.0), bottom: Val::Px(8.0) },
                                border:          UiRect::all(Val::Px(1.0)),
                                border_radius:   BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(if is_selected { colors::PURPLE_DARK } else { Color::srgba(0.08, 0.08, 0.10, 0.80) }),
                            BorderColor::all(if is_selected { Color::srgba(0.78, 0.67, 0.93, 0.50) } else { Color::srgba(1.0, 1.0, 1.0, 0.10) }),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(game.display_name()),
                                TextFont { font_size: 13.0, ..default() },
                                TextColor(if is_selected { colors::PURPLE_LIGHT } else { colors::TEXT_MUTED }),
                            ));
                        });
                    }
                });

                // Object skin grid (rebuilt on tab change)
                right.spawn((
                    ObjectSkinGrid,
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap:      FlexWrap::Wrap,
                        column_gap:     Val::Px(14.0),
                        row_gap:        Val::Px(14.0),
                        ..default()
                    },
                ))
                .with_children(|grid| {
                    spawn_object_skin_cards(grid, LahiGame::all()[0], &inventory, &equipped);
                });

                // Spacer
                right.spawn(Node { flex_grow: 1.0, ..default() });

                // Nav buttons
                right.spawn((
                    Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() },
                ))
                .with_children(|nav| {
                    nav.spawn((
                        NavButton::GoShop,
                        Button,
                        Node {
                            width: Val::Px(160.0), height: Val::Px(44.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.91, 0.79, 0.48, 0.12)),
                        BorderColor::all(Color::srgba(0.91, 0.79, 0.48, 0.40)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("⛁  SHOP"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(colors::GOLD),
                        ));
                    });

                    nav.spawn((
                        NavButton::Back,
                        Button,
                        Node {
                            width: Val::Px(140.0), height: Val::Px(44.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("◀  BACK"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(colors::TEXT_MUTED),
                        ));
                    });
                });
            });
        });
}

/// Spawns one card per owned object skin for the given game.
fn spawn_object_skin_cards(
    grid:      &mut ChildSpawnerCommands,
    game:      LahiGame,
    inventory: &Inventory,
    equipped:  &Equipped,
) {
    let currently_equipped = equipped.objects.get(&game).copied().unwrap_or("");
    for item in SHOP_CATALOG.iter().filter(|i| i.category == SkinCategory::GameObject(game) && inventory.0.contains(i.id)) {
        let is_equipped = item.id == currently_equipped;
        grid.spawn((
            ObjectSkinButton { id: item.id, game },
            Button,
            Node {
                flex_direction:  FlexDirection::Column,
                align_items:     AlignItems::Center,
                justify_content: JustifyContent::Center,
                width:           Val::Px(130.0),
                height:          Val::Px(120.0),
                border:          UiRect::all(Val::Px(1.0)),
                border_radius:   BorderRadius::all(Val::Px(10.0)),
                row_gap:         Val::Px(8.0),
                ..default()
            },
            BackgroundColor(if is_equipped { colors::PURPLE_DARK } else { Color::srgba(0.08, 0.08, 0.10, 0.80) }),
            BorderColor::all(if is_equipped { Color::srgba(0.78, 0.67, 0.93, 0.60) } else { Color::srgba(1.0, 1.0, 1.0, 0.10) }),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(item.icon),
                TextFont { font_size: 30.0, ..default() },
                TextColor(if is_equipped { colors::PURPLE_LIGHT } else { colors::TEXT_MUTED }),
            ));
            card.spawn((
                Text::new(if is_equipped { format!("{} ✓", item.name) } else { item.name.to_string() }),
                TextFont { font_size: 12.0, ..default() },
                TextColor(if is_equipped { colors::WHITE } else { colors::TEXT_MUTED }),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
    }
}

// ── Systems ───────────────────────────────────────────────────

fn handle_nav_buttons(
    mut interaction_q: Query<(&Interaction, &NavButton), (Changed<Interaction>, With<Button>)>,
    mut next: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in &mut interaction_q {
        if *interaction != Interaction::Pressed { continue; }
        match button {
            NavButton::Back   => next.set(GameState::MainMenu),
            NavButton::GoShop => next.set(GameState::Shop),
        }
    }
}

fn handle_tab_buttons(
    mut interaction_q: Query<(&Interaction, &TabButton), (Changed<Interaction>, With<Button>)>,
    mut tab:            ResMut<AvatarTabState>,
    mut commands:        Commands,
    grid_q:              Query<Entity, With<ObjectSkinGrid>>,
    inventory:           Res<Inventory>,
    equipped:            Res<Equipped>,
) {
    let mut changed = false;
    for (interaction, button) in &mut interaction_q {
        if *interaction == Interaction::Pressed {
            tab.selected_game = button.0;
            changed = true;
        }
    }
    if changed {
        if let Ok(grid_entity) = grid_q.single() {
            commands.entity(grid_entity).despawn_related::<Children>();
            let game = LahiGame::all()[tab.selected_game];
            commands.entity(grid_entity).with_children(|grid| {
                spawn_object_skin_cards(grid, game, &inventory, &equipped);
            });
        }
    }
}

fn handle_skin_buttons(
    char_q:   Query<(&Interaction, &CharacterSkinButton), (Changed<Interaction>, With<Button>)>,
    obj_q:    Query<(&Interaction, &ObjectSkinButton), (Changed<Interaction>, With<Button>)>,
    mut equipped: ResMut<Equipped>,
) {
    for (interaction, btn) in &char_q {
        if *interaction == Interaction::Pressed {
            equipped.character = btn.id;
        }
    }
    for (interaction, btn) in &obj_q {
        if *interaction == Interaction::Pressed {
            equipped.objects.insert(btn.game, btn.id);
        }
    }
}

/// Keeps tab highlight in sync with AvatarTabState every frame.
fn repaint_tabs(
    tab: Res<AvatarTabState>,
    mut q: Query<(&TabButton, &mut BackgroundColor, &mut BorderColor, &Children)>,
    mut text_q: Query<&mut TextColor>,
) {
    for (btn, mut bg, mut border, children) in &mut q {
        let selected = btn.0 == tab.selected_game;
        *bg = BackgroundColor(if selected { colors::PURPLE_DARK } else { Color::srgba(0.08, 0.08, 0.10, 0.80) });
        *border = BorderColor::all(if selected { Color::srgba(0.78, 0.67, 0.93, 0.50) } else { Color::srgba(1.0, 1.0, 1.0, 0.10) });
        for child in children.iter() {
            if let Ok(mut tc) = text_q.get_mut(child) {
                *tc = TextColor(if selected { colors::PURPLE_LIGHT } else { colors::TEXT_MUTED });
            }
        }
    }
}

/// Keeps character-skin highlight in sync with Equipped every frame.
fn repaint_skin_buttons(
    equipped: Res<Equipped>,
    mut char_q: Query<(&CharacterSkinButton, &mut BackgroundColor, &mut BorderColor), Without<ObjectSkinButton>>,
) {
    for (btn, mut bg, mut border) in &mut char_q {
        let is_equipped = equipped.character == btn.id;
        *bg = BackgroundColor(if is_equipped { colors::PURPLE_DARK } else { Color::srgba(0.08, 0.08, 0.10, 0.80) });
        *border = BorderColor::all(if is_equipped { Color::srgba(0.78, 0.67, 0.93, 0.60) } else { Color::srgba(1.0, 1.0, 1.0, 0.10) });
    }
}