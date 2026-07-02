// =============================================================
//  DULAHI 3D – UI Module Root
//  File: src/ui/mod.rs
//
//  Registers every UI sub-plugin and owns the shared GameState
//  enum plus all shared progression/team resources that multiple
//  screens read and write.
// =============================================================

pub mod splash;
pub mod main_menu;
pub mod avatar;
pub mod shop;
pub mod options;
pub mod mode_select;
pub mod game_select;
pub mod lobby;
pub mod loading;
pub mod results;

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

// ── Shared constants ─────────────────────────────────────────

/// Target logical resolution (landscape 20:9).
/// All UI is authored at this size and scaled to fit any device.
// ── Game-wide state machine ───────────────────────────────────

/// Every screen in the game maps to one variant.
/// Bevy's `State` plugin drives transitions; each UI module
/// only spawns/despawns entities when its state is active.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    /// First thing shown when the binary starts.
    #[default]
    Splash,
    /// Title / navigation hub.
    MainMenu,
    /// View/equip owned skins, see rank and stats.
    Avatar,
    /// Spend Puntos to unlock new skins.
    Shop,
    /// Audio, graphics, language settings.
    Options,
    /// Single-player vs multiplayer choice.
    ModeSelect,
    /// Choose game, team format, and difficulty (single-player path).
    GameSelect,
    /// LAN lobby – host / join a room, assign teams, AI-fill.
    Lobby,
    /// Pre-game loading with game-specific trivia.
    Loading,
    /// Scores, rewards, rematch prompt.
    Results,
    /// Actual gameplay (handed off to game-specific plugins).
    InGame,
}

// ── Larong Lahi games ──────────────────────────────────────────

/// Which game the player has selected.
/// Set during GameSelect / Lobby; read by Loading and InGame.
#[derive(Resource, Default, Clone, Debug)]
pub struct SelectedGame(pub LahiGame);

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LahiGame {
    #[default]
    None,
    Sipa,
    TumbangPreso,
    Patintero,
    Piko,
    LuksongTinik,
}

/// A team-vs-team match size. Every Larong Lahi game in Dulahi 3D
/// is played as Team A vs Team B — in single-player, AI fills every
/// slot except the human player's; in multiplayer, the host fills
/// empty slots with AI as needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchFormat {
    OneVOne,
    TwoVTwo,
    ThreeVThree,
}

impl MatchFormat {
    pub fn team_size(&self) -> usize {
        match self {
            MatchFormat::OneVOne     => 1,
            MatchFormat::TwoVTwo     => 2,
            MatchFormat::ThreeVThree => 3,
        }
    }

    pub fn total_players(&self) -> usize {
        self.team_size() * 2
    }

    pub fn label(&self) -> &'static str {
        match self {
            MatchFormat::OneVOne     => "1v1",
            MatchFormat::TwoVTwo     => "2v2",
            MatchFormat::ThreeVThree => "3v3",
        }
    }

    /// All three formats, used when filtering by a game's player range.
    pub fn all() -> &'static [MatchFormat] {
        &[MatchFormat::OneVOne, MatchFormat::TwoVTwo, MatchFormat::ThreeVThree]
    }
}

/// Which side of a team-based match a slot belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamSide {
    A,
    B,
}

impl TeamSide {
    pub fn label(&self) -> &'static str {
        match self {
            TeamSide::A => "Team A",
            TeamSide::B => "Team B",
        }
    }

    pub fn other(&self) -> TeamSide {
        match self {
            TeamSide::A => TeamSide::B,
            TeamSide::B => TeamSide::A,
        }
    }
}

/// Chosen match size for the current session (single-player or
/// multiplayer). Set on GameSelect/Lobby, read by Loading/InGame.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SelectedFormat(pub MatchFormat);
impl Default for SelectedFormat {
    fn default() -> Self { SelectedFormat(MatchFormat::OneVOne) }
}

impl LahiGame {
    pub fn display_name(&self) -> &'static str {
        match self {
            LahiGame::None          => "—",
            LahiGame::Sipa          => "Sipa",
            LahiGame::TumbangPreso  => "Tumbang Preso",
            LahiGame::Patintero     => "Patintero",
            LahiGame::Piko          => "Piko",
            LahiGame::LuksongTinik  => "Luksong Tinik",
        }
    }

    /// All five playable games, in a stable display order. Used by
    /// GameSelect, the Lobby game picker, and the Shop's per-game tabs.
    pub fn all() -> &'static [LahiGame] {
        &[
            LahiGame::Sipa,
            LahiGame::TumbangPreso,
            LahiGame::Patintero,
            LahiGame::Piko,
            LahiGame::LuksongTinik,
        ]
    }

    /// Minimum players, straight from the manuscript's per-game
    /// "No. of Players" field.
    pub fn min_players(&self) -> usize {
        match self {
            LahiGame::None         => 1,
            LahiGame::Sipa         => 1,
            LahiGame::TumbangPreso => 2,
            LahiGame::Patintero    => 2,
            LahiGame::Piko         => 1,
            LahiGame::LuksongTinik => 3,
        }
    }

    /// Maximum players, straight from the manuscript's per-game
    /// "No. of Players" field.
    pub fn max_players(&self) -> usize {
        match self {
            LahiGame::None         => 1,
            LahiGame::Sipa         => 4,
            LahiGame::TumbangPreso => 6,
            LahiGame::Patintero    => 6,
            LahiGame::Piko         => 4,
            LahiGame::LuksongTinik => 6,
        }
    }

    /// Which team formats (1v1 / 2v2 / 3v3) fit within this game's
    /// documented player range. A format is offered only if its
    /// total player count falls within [min_players, max_players].
    pub fn available_formats(&self) -> Vec<MatchFormat> {
        MatchFormat::all()
            .iter()
            .copied()
            .filter(|f| {
                let total = f.total_players();
                total >= self.min_players() && total <= self.max_players()
            })
            .collect()
    }

    /// Returns one trivia fact shown on the pre-game loading screen.
    pub fn trivia(&self) -> &'static [&'static str] {
        match self {
            LahiGame::None => &["Dulahi 3D — Larong Lahi, digital edition."],
            LahiGame::Sipa => &[
                "Sipa means 'kick' in Filipino. The goal is to keep the sipa/takyan airborne using your feet.",
                "A sipa/takyan is traditionally made from strips of lead washer and feathers.",
                "Sipa was once considered the national sport of the Philippines before arnis took the title.",
                "Players earn one point per successful kick without letting the sipa touch the ground.",
            ],
            LahiGame::TumbangPreso => &[
                "Tumbang Preso means 'knock down the prisoner' — the can is the prisoner!",
                "The taya (it) must guard the can while players try to topple it with their tsinelas.",
                "If the taya tags a player before they retrieve their slipper, that player becomes the new taya.",
                "Tumbang Preso teaches accuracy, strategy, and quick reflexes.",
            ],
            LahiGame::Patintero => &[
                "Patintero is a team territory game played on a grid drawn in chalk or dirt.",
                "The defending team must block all horizontal and vertical lines without letting runners cross.",
                "A team scores when all runners successfully cross the entire grid and return.",
                "Patintero is sometimes called 'tubigan' in some regions of the Philippines.",
            ],
            LahiGame::Piko => &[
                "Piko is the Filipino version of hopscotch, played on numbered squares.",
                "Players toss a pamato (small stone or piece of tile) to mark their progress.",
                "You must hop through all squares on one foot without stepping on lines or your pamato square.",
                "Piko develops balance, coordination, and counting skills in young players.",
            ],
            LahiGame::LuksongTinik => &[
                "Luksong Tinik means 'jump over the thorns' — players jump over crossed hands and feet.",
                "Two players form the 'tinik' by pressing their hands and feet together, raising the height each round.",
                "A player who touches the tinik becomes the new tinik-holder.",
                "The game tests timing, agility, and jumping height — all while having fun with friends.",
            ],
        }
    }
}

/// Which mode was selected.
#[derive(Resource, Default, Clone, Debug, PartialEq, Eq)]
pub enum PlayMode {
    #[default]
    SinglePlayer,
    Multiplayer,
}

/// Difficulty for AI-controlled players (teammates and opponents),
/// applied per-session in single-player or per-bot in the lobby.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    pub fn label(&self) -> &'static str {
        match self {
            Difficulty::Easy   => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard   => "Hard",
        }
    }

    /// Cycles Easy -> Normal -> Hard -> Easy. Used by the lobby's
    /// per-AI-slot difficulty control.
    pub fn cycle(&self) -> Difficulty {
        match self {
            Difficulty::Easy   => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard   => Difficulty::Easy,
        }
    }
}

// ── Progression: currency, rank, inventory ────────────────────

/// In-game currency ("Puntos"), earned from match results and
/// spent in the Shop. Starts with enough for one premium skin so
/// the Shop doesn't feel empty on a fresh install.
#[derive(Resource, Clone, Debug)]
pub struct Currency(pub u32);
impl Default for Currency {
    fn default() -> Self { Currency(250) }
}

/// Accumulated rank points from match performance (wins, scores,
/// daily play). Maps to a RankTier for display.
#[derive(Resource, Default, Clone, Debug)]
pub struct RankPoints(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RankTier {
    Baguhan,
    Manlalaro,
    Mandirigma,
    Bayani,
    Datu,
}

impl RankTier {
    /// Determines tier from accumulated rank points.
    pub fn from_points(points: u32) -> RankTier {
        match points {
            0..=99      => RankTier::Baguhan,
            100..=299   => RankTier::Manlalaro,
            300..=599   => RankTier::Mandirigma,
            600..=999   => RankTier::Bayani,
            _           => RankTier::Datu,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            RankTier::Baguhan    => "Baguhan",
            RankTier::Manlalaro  => "Manlalaro",
            RankTier::Mandirigma => "Mandirigma",
            RankTier::Bayani     => "Bayani",
            RankTier::Datu       => "Datu",
        }
    }

    /// Points needed to reach the *next* tier, or None if already Datu.
    pub fn next_threshold(&self) -> Option<u32> {
        match self {
            RankTier::Baguhan    => Some(100),
            RankTier::Manlalaro  => Some(300),
            RankTier::Mandirigma => Some(600),
            RankTier::Bayani     => Some(1000),
            RankTier::Datu       => None,
        }
    }
}

/// Which cosmetic slot an item belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkinCategory {
    Character,
    GameObject(LahiGame),
}

/// One purchasable/equippable cosmetic item.
#[derive(Clone, Copy, Debug)]
pub struct ShopItem {
    pub id:       &'static str,
    pub name:     &'static str,
    pub price:    u32,
    pub category: SkinCategory,
    pub icon:     &'static str,
}

/// Static catalog of every skin in the game. Each category has one
/// free default (price 0, owned from the start) plus one premium
/// unlockable. Actual 3D models are out of scope for this pass —
/// these are UI-layer placeholders (icon glyph + name) until the
/// team's Blender assets are ready to swap in.
pub const SHOP_CATALOG: &[ShopItem] = &[
    // Character skins
    ShopItem { id: "char_default",    name: "Manlilikha (Default)", price: 0,   category: SkinCategory::Character, icon: "◈" },
    ShopItem { id: "char_manggagawa", name: "Manggagawa Attire",    price: 200, category: SkinCategory::Character, icon: "◆" },
    ShopItem { id: "char_datu",       name: "Datu's Regalia",       price: 450, category: SkinCategory::Character, icon: "❖" },
    // Sipa
    ShopItem { id: "sipa_default", name: "Rattan Takyan", price: 0,   category: SkinCategory::GameObject(LahiGame::Sipa), icon: "◉" },
    ShopItem { id: "sipa_gold",    name: "Gilded Takyan",  price: 180, category: SkinCategory::GameObject(LahiGame::Sipa), icon: "✦" },
    // Tumbang Preso
    ShopItem { id: "preso_default", name: "Tin Lata",    price: 0,   category: SkinCategory::GameObject(LahiGame::TumbangPreso), icon: "◎" },
    ShopItem { id: "preso_bronze",  name: "Bronze Lata",  price: 180, category: SkinCategory::GameObject(LahiGame::TumbangPreso), icon: "⊙" },
    // Patintero
    ShopItem { id: "patintero_default", name: "Chalk Lines", price: 0,   category: SkinCategory::GameObject(LahiGame::Patintero), icon: "⊞" },
    ShopItem { id: "patintero_neon",    name: "Neon Lines",  price: 180, category: SkinCategory::GameObject(LahiGame::Patintero), icon: "▦" },
    // Piko
    ShopItem { id: "piko_default", name: "Stone Pamato", price: 0,   category: SkinCategory::GameObject(LahiGame::Piko), icon: "◫" },
    ShopItem { id: "piko_jade",    name: "Jade Pamato",   price: 180, category: SkinCategory::GameObject(LahiGame::Piko), icon: "◇" },
    // Luksong Tinik
    ShopItem { id: "tinik_default", name: "Woven Tinik",       price: 0,   category: SkinCategory::GameObject(LahiGame::LuksongTinik), icon: "⇑" },
    ShopItem { id: "tinik_thorned", name: "Thorned Vine Tinik", price: 180, category: SkinCategory::GameObject(LahiGame::LuksongTinik), icon: "⚡" },
];

/// Item IDs the player owns. Default skins (price 0) are owned
/// from the start so every category always has something equippable.
#[derive(Resource, Clone, Debug)]
pub struct Inventory(pub HashSet<&'static str>);
impl Default for Inventory {
    fn default() -> Self {
        let mut owned = HashSet::new();
        for item in SHOP_CATALOG {
            if item.price == 0 {
                owned.insert(item.id);
            }
        }
        Inventory(owned)
    }
}

/// Currently equipped character skin + one equipped object skin
/// per game.
#[derive(Resource, Clone, Debug)]
pub struct Equipped {
    pub character: &'static str,
    pub objects:   HashMap<LahiGame, &'static str>,
}
impl Default for Equipped {
    fn default() -> Self {
        let mut objects = HashMap::new();
        objects.insert(LahiGame::Sipa,         "sipa_default");
        objects.insert(LahiGame::TumbangPreso, "preso_default");
        objects.insert(LahiGame::Patintero,    "patintero_default");
        objects.insert(LahiGame::Piko,         "piko_default");
        objects.insert(LahiGame::LuksongTinik, "tinik_default");
        Equipped { character: "char_default", objects }
    }
}

/// Lightweight running stats shown on the Avatar screen. Updated on
/// entering Results; starts at zero on a fresh install.
#[derive(Resource, Default, Clone, Debug)]
pub struct PlayerStats {
    pub wins:   u32,
    pub losses: u32,
}
impl PlayerStats {
    pub fn win_rate(&self) -> f32 {
        let total = self.wins + self.losses;
        if total == 0 { 0.0 } else { self.wins as f32 / total as f32 * 100.0 }
    }
}

// ── Master UI plugin ──────────────────────────────────────────

/// Register this plugin in `main.rs`.
/// It inserts shared resources and registers every sub-plugin.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Global state machine
            .init_state::<GameState>()
            // Shared resources
            .init_resource::<SelectedGame>()
            .insert_resource(PlayMode::default())
            .insert_resource(Difficulty::default())
            .init_resource::<SelectedFormat>()
            .init_resource::<Currency>()
            .init_resource::<RankPoints>()
            .init_resource::<Inventory>()
            .init_resource::<Equipped>()
            .init_resource::<PlayerStats>()
            // Sub-plugins (one per screen)
            .add_plugins((
                splash::SplashPlugin,
                main_menu::MainMenuPlugin,
                avatar::AvatarPlugin,
                shop::ShopPlugin,
                options::OptionsPlugin,
                mode_select::ModeSelectPlugin,
                game_select::GameSelectPlugin,
                lobby::LobbyPlugin,
                loading::LoadingPlugin,
                results::ResultsPlugin,
            ));
    }
}

// ── Helper: despawn everything tagged with a marker ──────────

/// Generic cleanup used by every sub-plugin.
/// Each module defines its own marker component (e.g. `MainMenuRoot`)
/// and calls `despawn_screen::<MainMenuRoot>` on exit.
pub fn despawn_screen<T: Component>(
    mut commands: Commands,
    query:        Query<Entity, With<T>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ── Shared color palette (low-poly Philippine night aesthetic) ─

pub mod colors {
    use bevy::prelude::Color;

    /// Deep night-sky background.
    pub const BG_DARK:     Color = Color::srgb(0.06, 0.10, 0.14);
    /// Primary gold accent (title, highlights, currency).
    pub const GOLD:        Color = Color::srgb(0.91, 0.79, 0.48);
    /// Secondary green accent (buttons, foliage, Team A).
    pub const GREEN_LIGHT: Color = Color::srgb(0.60, 0.94, 0.55);
    pub const GREEN_MID:   Color = Color::srgb(0.16, 0.42, 0.16);
    pub const GREEN_DARK:  Color = Color::srgb(0.04, 0.12, 0.04);
    /// Danger / exit red / Team B.
    pub const RED_DARK:    Color = Color::srgb(0.42, 0.10, 0.10);
    pub const RED_LIGHT:   Color = Color::srgb(0.93, 0.67, 0.67);
    /// Neutral muted text.
    pub const TEXT_MUTED:  Color = Color::srgb(0.60, 0.60, 0.60);
    /// White full.
    pub const WHITE:       Color = Color::srgb(1.0, 1.0, 1.0);
    /// Blue-gray for secondary buttons.
    pub const BLUE_DARK:   Color = Color::srgb(0.10, 0.10, 0.23);
    pub const BLUE_LIGHT:  Color = Color::srgb(0.67, 0.67, 0.93);
    /// Purple accent for Avatar/Shop screens.
    pub const PURPLE_DARK:  Color = Color::srgb(0.18, 0.10, 0.26);
    pub const PURPLE_LIGHT: Color = Color::srgb(0.78, 0.67, 0.93);
    /// Amber accent for AI-bot indicators and secondary highlights.
    pub const AMBER_DARK:  Color = Color::srgb(0.30, 0.20, 0.04);
    pub const AMBER_LIGHT: Color = Color::srgb(0.96, 0.78, 0.40);
}