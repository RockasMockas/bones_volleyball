#![allow(clippy::too_many_arguments)]
pub mod gameplay;
pub mod input;
pub mod menu;
pub mod networking;

pub use input::*;
pub use menu::*;
pub use networking::*;

use bones_bevy_renderer::{bevy::diagnostic::LogDiagnosticsPlugin, BonesBevyRenderer};
use bones_framework::prelude::*;

/// Metadata for the game, including sprite handles and fonts
#[derive(HasSchema, Default, Clone, Debug)]
#[repr(C)]
#[type_data(metadata_asset("game"))]
pub struct GameMeta {
    pub matchmaking_server: String,
    pub player_sprite: Handle<Image>,
    pub floor_sprite: Handle<Image>,
    pub net_sprite: Handle<Image>,
    pub title_font: FontMeta,
    pub fonts: SVec<Handle<Font>>,
}

/// Provides constants for session names
#[derive(Clone, Debug)]
pub struct SessionNames;

impl SessionNames {
    pub const MAIN_MENU: &'static str = "main_menu";
    pub const GAMEPLAY: &'static str = "gameplay";
    pub const GAMEPLAY_UI: &'static str = "gameplay_ui";
}

/// Entry point of the application
fn main() {
    let game = create_game();
    let mut renderer = BonesBevyRenderer::new(game);
    renderer.app_namespace = ("org".into(), "example".into(), "bones.volleyball".into());
    renderer
        .app()
        .add_plugins(LogDiagnosticsPlugin::default())
        .run();
}

/// Creates and configures the game instance
pub fn create_game() -> Game {
    let mut game = Game::new();

    // Install default plugin and initialize resources
    game.install_plugin(DefaultGamePlugin)
        .init_shared_resource::<AssetServer>()
        .register_default_assets();

    // Register the schema of all Metas
    GameMeta::register_schema();

    // Create the main menu session and install the menu plugin
    game.sessions
        .create(SessionNames::MAIN_MENU)
        .install_plugin(menu_plugin);

    game
}
