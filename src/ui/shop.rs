// =============================================================
//  DULAHI 3D – Shop Screen
//  File: src/ui/shop.rs
//
//  Spend Puntos to unlock character and game-object skins.
//  Reached from the main menu (or from Avatar's "Shop" button).
//  Equipping happens on the Avatar screen, not here.
// =============================================================

use bevy::prelude::*;
use crate::ui::{
    GameState, LahiGame, Inventory, Currency, SHOP_CATALOG, SkinCategory,
    colors, despawn_screen,
};

// ── Public plugin ─────────────────────────────────────────────

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ShopTabState>()
            .add_systems(OnEnter(GameState::Shop), spawn_shop)
            .add_systems(
                Update,
                (handle_nav_buttons, handle_tab_buttons, handle_buy_buttons,
                 repaint_tabs, repaint_buy_buttons, repaint_currency)
                    .run_if(in_state(GameState::Shop)),
            )
            .add_systems(OnExit(GameState::Shop), despawn_screen::<ShopRoot>);
    }
}

// ── Local state ───────────────────────────────────────────────

/// Tab 0 = Character skins. Tabs 1..=5 = one per LahiGame::all().
#[derive(Resource, Default)]
struct ShopTabState {
    selected_tab: usize,
}

fn category_for_tab(idx: usize) -> SkinCategory {
    if idx == 0 {
        SkinCategory::Character
    } else {
        SkinCategory::GameObject(LahiGame::all()[idx - 1])
    }
}

fn tab_label(idx: usize) -> &'static str {
    if idx == 0 {
        "Character"
    } else {
        LahiGame::all()[idx - 1].display_name()
    }
}

// ── Markers ───────────────────────────────────────────────────

#[derive(Component)]
pub struct ShopRoot;

#[derive(Component)]
enum NavButton { Back, GoAvatar }

#[derive(Component)]
struct TabButton(usize);

#[derive(Component)]
struct ItemGrid;

#[derive(Component)]
struct CurrencyLabel;

#[derive(Component, Clone, Copy)]
struct BuyButton { id: &'static str, price: u32 }

// ── Spawn ─────────────────────────────────────────────────────

fn spawn_shop(
    mut commands: Commands,
    inventory:    Res<Inventory>,
    currency:     Res<Currency>,
    mut tab:      ResMut<ShopTabState>,
) {
    tab.selected_tab = 0;

    commands
        .spawn((
            ShopRoot,
            Node {
                width:          Val::Percent(100.0),
                height:         Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding:        UiRect::all(Val::Px(28.0)),
                row_gap:        Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.07, 0.10, 0.97)),
            ZIndex(20),
        ))
        .with_children(|root| {

            // ── Header row: title + currency ──
            root.spawn((
                Node {
                    flex_direction:  FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items:     AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|hdr| {
                hdr.spawn((
                    Text::new("SHOP"),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(colors::PURPLE_LIGHT),
                ));

                hdr.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items:    AlignItems::Center,
                        column_gap:     Val::Px(10.0),
                        padding:        UiRect { left: Val::Px(16.0), right: Val::Px(16.0), top: Val::Px(8.0), bottom: Val::Px(8.0) },
                        border:         UiRect::all(Val::Px(1.0)),
                        border_radius:  BorderRadius::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.91, 0.79, 0.48, 0.10)),
                    BorderColor::all(Color::srgba(0.91, 0.79, 0.48, 0.35)),
                ))
                .with_children(|chip| {
                    chip.spawn((
                        Node { width: Val::Px(14.0), height: Val::Px(14.0), border_radius: BorderRadius::all(Val::Percent(50.0)), ..default() },
                        BackgroundColor(colors::GOLD),
                    ));
                    chip.spawn((
                        CurrencyLabel,
                        Text::new(format!("{} Puntos", currency.0)),
                        TextFont { font_size: 15.0, ..default() },
                        TextColor(colors::GOLD),
                    ));
                });
            });

            // ── Tabs row ──
            root.spawn((
                Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(8.0), flex_wrap: FlexWrap::Wrap, ..default() },
            ))
            .with_children(|tabs| {
                for i in 0..=5 {
                    let is_selected = i == 0;
                    tabs.spawn((
                        TabButton(i),
                        Button,
                        Node {
                            padding:       UiRect { left: Val::Px(14.0), right: Val::Px(14.0), top: Val::Px(8.0), bottom: Val::Px(8.0) },
                            border:        UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(if is_selected { colors::PURPLE_DARK } else { Color::srgba(0.08, 0.08, 0.10, 0.80) }),
                        BorderColor::all(if is_selected { Color::srgba(0.78, 0.67, 0.93, 0.50) } else { Color::srgba(1.0, 1.0, 1.0, 0.10) }),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(tab_label(i)),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(if is_selected { colors::PURPLE_LIGHT } else { colors::TEXT_MUTED }),
                        ));
                    });
                }
            });

            // ── Item grid ──
            root.spawn((
                ItemGrid,
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap:      FlexWrap::Wrap,
                    column_gap:     Val::Px(14.0),
                    row_gap:        Val::Px(14.0),
                    ..default()
                },
            ))
            .with_children(|grid| {
                spawn_item_cards(grid, category_for_tab(0), &inventory, currency.0);
            });

            // Spacer
            root.spawn(Node { flex_grow: 1.0, ..default() });

            // ── Nav buttons ──
            root.spawn((
                Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() },
            ))
            .with_children(|nav| {
                nav.spawn((
                    NavButton::GoAvatar,
                    Button,
                    Node {
                        width: Val::Px(160.0), height: Val::Px(44.0),
                        justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.78, 0.67, 0.93, 0.12)),
                    BorderColor::all(Color::srgba(0.78, 0.67, 0.93, 0.40)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("◈  EQUIP"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors::PURPLE_LIGHT),
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
}

/// Spawns one card per catalog item in the given category —
/// owned AND locked items both appear, so the player can see
/// what's still available to unlock.
fn spawn_item_cards(
    grid:      &mut ChildSpawnerCommands,
    category:  SkinCategory,
    inventory: &Inventory,
    currency:  u32,
) {
    for item in SHOP_CATALOG.iter().filter(|i| i.category == category) {
        let owned      = inventory.0.contains(item.id);
        let affordable = currency >= item.price;

        grid.spawn((
            Node {
                flex_direction:  FlexDirection::Column,
                align_items:     AlignItems::Center,
                justify_content: JustifyContent::Center,
                width:           Val::Px(140.0),
                height:          Val::Px(150.0),
                border:          UiRect::all(Val::Px(1.0)),
                border_radius:   BorderRadius::all(Val::Px(10.0)),
                row_gap:         Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.10, 0.80)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.10)),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(item.icon),
                TextFont { font_size: 30.0, ..default() },
                TextColor(if owned { colors::GOLD } else { colors::TEXT_MUTED }),
            ));
            card.spawn((
                Text::new(item.name),
                TextFont { font_size: 12.0, ..default() },
                TextColor(colors::WHITE),
                TextLayout::new_with_justify(Justify::Center),
            ));

            if owned {
                card.spawn((
                    Text::new("OWNED"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(colors::GREEN_LIGHT),
                ));
            } else {
                card.spawn((
                    BuyButton { id: item.id, price: item.price },
                    Button,
                    Node {
                        width:           Val::Px(100.0),
                        height:          Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items:     AlignItems::Center,
                        border:          UiRect::all(Val::Px(1.0)),
                        border_radius:   BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(if affordable { colors::GREEN_MID } else { Color::srgba(0.15, 0.05, 0.05, 0.80) }),
                    BorderColor::all(if affordable { Color::srgba(0.40, 0.94, 0.40, 0.50) } else { Color::srgba(0.94, 0.40, 0.40, 0.30) }),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(format!("{} P", item.price)),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(if affordable { colors::GREEN_LIGHT } else { colors::RED_LIGHT }),
                    ));
                });
            }
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
            NavButton::Back     => next.set(GameState::MainMenu),
            NavButton::GoAvatar => next.set(GameState::Avatar),
        }
    }
}

fn handle_tab_buttons(
    mut interaction_q: Query<(&Interaction, &TabButton), (Changed<Interaction>, With<Button>)>,
    mut tab:            ResMut<ShopTabState>,
    mut commands:        Commands,
    grid_q:              Query<Entity, With<ItemGrid>>,
    inventory:           Res<Inventory>,
    currency:            Res<Currency>,
) {
    let mut changed = false;
    for (interaction, button) in &mut interaction_q {
        if *interaction == Interaction::Pressed {
            tab.selected_tab = button.0;
            changed = true;
        }
    }
    if changed {
        if let Ok(grid_entity) = grid_q.single() {
            commands.entity(grid_entity).despawn_related::<Children>();
            let category = category_for_tab(tab.selected_tab);
            commands.entity(grid_entity).with_children(|grid| {
                spawn_item_cards(grid, category, &inventory, currency.0);
            });
        }
    }
}

fn handle_buy_buttons(
    interaction_q: Query<(&Interaction, &BuyButton), (Changed<Interaction>, With<Button>)>,
    mut currency:   ResMut<Currency>,
    mut inventory:  ResMut<Inventory>,
    mut commands:   Commands,
    tab:            Res<ShopTabState>,
    grid_q:         Query<Entity, With<ItemGrid>>,
) {
    let mut bought = false;
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed
            && !inventory.0.contains(btn.id)
            && currency.0 >= btn.price
        {
            currency.0 -= btn.price;
            inventory.0.insert(btn.id);
            bought = true;
        }
    }
    // Rebuild the grid so the purchased item flips to "OWNED".
    if bought {
        if let Ok(grid_entity) = grid_q.single() {
            commands.entity(grid_entity).despawn_related::<Children>();
            let category = category_for_tab(tab.selected_tab);
            commands.entity(grid_entity).with_children(|grid| {
                spawn_item_cards(grid, category, &inventory, currency.0);
            });
        }
    }
}

fn repaint_tabs(
    tab: Res<ShopTabState>,
    mut q: Query<(&TabButton, &mut BackgroundColor, &mut BorderColor, &Children)>,
    mut text_q: Query<&mut TextColor>,
) {
    for (btn, mut bg, mut border, children) in &mut q {
        let selected = btn.0 == tab.selected_tab;
        *bg = BackgroundColor(if selected { colors::PURPLE_DARK } else { Color::srgba(0.08, 0.08, 0.10, 0.80) });
        *border = BorderColor::all(if selected { Color::srgba(0.78, 0.67, 0.93, 0.50) } else { Color::srgba(1.0, 1.0, 1.0, 0.10) });
        for child in children.iter() {
            if let Ok(mut tc) = text_q.get_mut(child) {
                *tc = TextColor(if selected { colors::PURPLE_LIGHT } else { colors::TEXT_MUTED });
            }
        }
    }
}

/// Keeps Buy button colors in sync with affordability every frame
/// (covers the case where currency drops after a purchase elsewhere
/// on the same visible grid).
fn repaint_buy_buttons(
    currency: Res<Currency>,
    mut q: Query<(&BuyButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (btn, mut bg, mut border) in &mut q {
        let affordable = currency.0 >= btn.price;
        *bg = BackgroundColor(if affordable { colors::GREEN_MID } else { Color::srgba(0.15, 0.05, 0.05, 0.80) });
        *border = BorderColor::all(if affordable { Color::srgba(0.40, 0.94, 0.40, 0.50) } else { Color::srgba(0.94, 0.40, 0.40, 0.30) });
    }
}

fn repaint_currency(
    currency: Res<Currency>,
    mut q: Query<&mut Text, With<CurrencyLabel>>,
) {
    if !currency.is_changed() { return; }
    for mut text in &mut q {
        *text = Text::new(format!("{} Puntos", currency.0));
    }
}