use crate::input::{ControlSource, PlayerControlMapping, PlayerInputCollector};
use crate::{
    networking::{handle_online_menu_matchmaking, NetworkGameState, NetworkGameStatus},
    GameMeta,
};
use bones_framework::prelude::*;
use egui::{Color32, RichText};
use std::time::Duration;

/// Represents the current state of the menu
#[derive(HasSchema, Clone, Default)]
pub enum MenuState {
    #[default]
    MainMenu,
    OnlinePlayConfig,
}

/// Holds data related to the menu state and configuration
#[derive(HasSchema, Clone)]
#[repr(C)]
pub struct MenuData {
    pub state: MenuState,
    pub selected_option: usize,
    pub scroll_timer: Duration,
    pub input_delay_frames: usize,
}

impl Default for MenuData {
    /// Creates a new MenuData instance with default values
    fn default() -> Self {
        Self {
            state: MenuState::MainMenu,
            selected_option: 0,
            scroll_timer: Duration::ZERO,
            input_delay_frames: 2,
        }
    }
}

/// Installs the menu plugin and its associated systems
pub fn menu_plugin(session: &mut Session) {
    session.install_plugin(DefaultSessionPlugin);
    session.world.init_resource::<PlayerInputCollector>();
    session.world.init_resource::<PlayerControlMapping>();
    session.world.init_resource::<MenuData>();
    session.world.init_resource::<NetworkGameState>();

    session
        .add_system_to_stage(Update, handle_menu_input)
        .add_system_to_stage(Update, menu_selection_system)
        .add_system_to_stage(Update, menu_draw_system)
        .add_system_to_stage(Update, handle_online_menu_matchmaking)
        .add_startup_system(menu_startup);
}

/// Handles menu selection and navigation
fn menu_selection_system(
    mut menu_data: ResMut<MenuData>,
    mut network_state: ResMut<NetworkGameState>,
    input_collector: Res<PlayerInputCollector>,
    time: Res<Time>,
) {
    let player_control = input_collector.get_control(0, ControlSource::KeyboardAndGamepads);

    // Handle menu navigation with delay
    menu_data.scroll_timer = menu_data.scroll_timer.saturating_sub(time.delta());
    if menu_data.scroll_timer.is_zero() {
        match menu_data.state {
            MenuState::MainMenu => {
                // Handle main menu navigation
                if player_control.up_pressed {
                    menu_data.selected_option = menu_data.selected_option.saturating_sub(1);
                    menu_data.scroll_timer = Duration::from_millis(200);
                } else if player_control.down_pressed {
                    menu_data.selected_option = (menu_data.selected_option + 1).min(1);
                    menu_data.scroll_timer = Duration::from_millis(200);
                }
            }
            MenuState::OnlinePlayConfig => {
                // Handle input delay adjustment
                if player_control.left_pressed {
                    menu_data.input_delay_frames =
                        menu_data.input_delay_frames.saturating_sub(1).max(1);
                    menu_data.scroll_timer = Duration::from_millis(200);
                } else if player_control.right_pressed {
                    menu_data.input_delay_frames = (menu_data.input_delay_frames + 1).min(60);
                    menu_data.scroll_timer = Duration::from_millis(200);
                }
            }
        }
    }

    // Handle menu selection if we're not searching for an online match
    if network_state.status.is_idle() {
        if player_control.jump_just_pressed || player_control.enter_just_pressed {
            match menu_data.state {
                MenuState::MainMenu => match menu_data.selected_option {
                    0 => {
                        menu_data.state = MenuState::OnlinePlayConfig;
                        menu_data.selected_option = 0;
                    }
                    1 => {
                        println!("Exiting game...");
                        std::process::exit(0);
                    }
                    _ => {}
                },
                MenuState::OnlinePlayConfig => {
                    // Trigger the match making logic
                    network_state.status = NetworkGameStatus::Searching;
                }
            }
        } else if player_control.esc_start_pressed {
            // Return to main menu from online config submenu
            if matches!(menu_data.state, MenuState::OnlinePlayConfig) {
                menu_data.state = MenuState::MainMenu;
                menu_data.selected_option = 0;
            }
        }
    }
    // If searching for an online match, allow exiting matchmaking
    else {
        if player_control.esc_start_pressed {
            network_state.status = NetworkGameStatus::Idle;
            menu_data.state = MenuState::MainMenu;
        }
    }
}

/// Draws the menu UI
fn menu_draw_system(
    meta: Root<GameMeta>,
    ctx: Res<EguiCtx>,
    menu_data: Res<MenuData>,
    network_state: Res<NetworkGameState>,
) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(meta.title_font.rich("Bones Volleyball"));
            ui.add_space(30.0);

            match menu_data.state {
                MenuState::MainMenu => {
                    let options = ["Online Play", "Exit"];
                    for (i, option) in options.iter().enumerate() {
                        let text = if i == menu_data.selected_option {
                            format!("> {} <", option)
                        } else {
                            option.to_string()
                        };
                        ui.label(menu_small_text(text));
                    }
                }
                MenuState::OnlinePlayConfig => {
                    ui.label(menu_small_text(format!(
                        "Input Delay Frames: {}",
                        menu_data.input_delay_frames
                    )));
                }
            }

            ui.add_space(30.0);
            match network_state.status {
                NetworkGameStatus::Searching => {
                    ui.label(menu_small_text("Waiting for an opponent.."));
                }
                NetworkGameStatus::WaitingForPlayers => {
                    ui.label(menu_small_text("Waiting for an opponent..."));
                }
                NetworkGameStatus::MatchFound => {
                    ui.label(menu_small_text("Match Starting..."));
                }
                NetworkGameStatus::Idle => {}
            }

            ui.add_space(ui.available_height() - 30.0);

            if matches!(menu_data.state, MenuState::OnlinePlayConfig) {
                ui.label(menu_tiny_text("Press Enter to start matchmaking..."));
            }
        });
    });
}

/// Initializes the menu settings
fn menu_startup(
    mut egui_settings: ResMutInit<EguiSettings>,
    mut clear_color: ResMutInit<ClearColor>,
    mut menu_data: ResMutInit<MenuData>,
) {
    **clear_color = Color::BLACK;
    egui_settings.scale = 2.0;
    menu_data.scroll_timer = Duration::ZERO;
    menu_data.input_delay_frames = 2;
}

/// Handles the menu input by interacting with the input collector directly
pub fn handle_menu_input(
    mut input_collector: ResMut<PlayerInputCollector>,
    control_mapping: Res<PlayerControlMapping>,
    keyboard: Res<KeyboardInputs>,
    gamepad: Res<GamepadInputs>,
) {
    input_collector.apply_inputs(&control_mapping, &keyboard, &gamepad);
    input_collector.update_just_pressed();
    input_collector.advance_frame();
}

/// Creates a RichText instance for small menu text
fn menu_small_text(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .size(28.0)
        .color(Color32::WHITE)
        .strong()
}

/// Creates a RichText instance for tiny menu text
fn menu_tiny_text(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .size(18.0)
        .color(Color32::WHITE)
        .strong()
}
