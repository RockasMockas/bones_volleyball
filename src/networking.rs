use crate::gameplay::GameplayPlugin;
use crate::input::GameNetworkInputConfig;
use crate::menu::menu::MenuData;
use crate::GameMeta;
use bones_framework::networking::online::{self, SearchState};
use bones_framework::networking::GgrsSessionRunner;
use bones_framework::networking::GgrsSessionRunnerInfo;
use bones_framework::prelude::*;

/// The target frames per second for the game
const FPS: f32 = 60.0;
/// The maximum number of frames the game can predict ahead
const MAX_PREDICTION_WINDOW: Option<usize> = Some(10);
/// The maximum number of players allowed in a game
const MAX_PLAYERS: u32 = 2;

/// Represents the current status of the network game
#[derive(HasSchema, Default, PartialEq, Eq, Clone, Copy)]
pub enum NetworkGameStatus {
    #[default]
    Idle,
    Searching,
    WaitingForPlayers,
    MatchFound,
}

impl NetworkGameStatus {
    /// Returns true if the status is MatchFound
    pub fn is_match_found(&self) -> bool {
        matches!(self, NetworkGameStatus::MatchFound)
    }

    /// Returns true if the status is Idle
    pub fn is_idle(&self) -> bool {
        matches!(self, NetworkGameStatus::Idle)
    }

    /// Returns true if the status is Searching
    pub fn is_searching(&self) -> bool {
        matches!(self, NetworkGameStatus::Searching)
    }

    /// Returns true if the status is WaitingForPlayers
    pub fn is_waiting_for_players(&self) -> bool {
        matches!(self, NetworkGameStatus::WaitingForPlayers)
    }
}

/// Represents the current state of the network game
#[derive(HasSchema, Clone)]
#[repr(C)]
pub struct NetworkGameState {
    /// The current status of the network game
    pub status: NetworkGameStatus,
}

impl Default for NetworkGameState {
    /// Creates a new NetworkGameState with default values
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkGameState {
    /// Creates a new NetworkGameState
    pub fn new() -> Self {
        Self {
            status: NetworkGameStatus::Idle,
        }
    }

    /// Resets the network game state to idle
    pub fn reset(&mut self) {
        self.status = NetworkGameStatus::Idle;
    }
}

/// Handles the matchmaking/connection logic tied to the online menu state by matching on NetworkGameStatus
pub fn handle_online_menu_matchmaking(
    mut network_state: ResMut<NetworkGameState>,
    sessions: ResMut<Sessions>,
    mut session_options: ResMut<SessionOptions>,
    menu_data: Res<MenuData>,
    meta: Root<GameMeta>,
) {
    match network_state.status {
        NetworkGameStatus::Searching => {
            // Start searching for a match
            println!("Started searching for match!");
            let server = meta.matchmaking_server.parse().expect("invalid server id");
            online::start_search_for_game(server, MAX_PLAYERS);
            network_state.status = NetworkGameStatus::WaitingForPlayers;
        }
        NetworkGameStatus::WaitingForPlayers => {
            // Check if a match has been found
            let mut search_state = SearchState::Searching;
            if let Some(online_socket) = online::update_search_for_game(&mut search_state) {
                network_state.status = NetworkGameStatus::MatchFound;

                // Create a new session runner for the game
                let session_runner = Box::new(GgrsSessionRunner::<GameNetworkInputConfig>::new(
                    FPS,
                    GgrsSessionRunnerInfo::new(
                        online_socket.ggrs_socket(),
                        MAX_PREDICTION_WINDOW,
                        Some(menu_data.input_delay_frames), // Use the custom input delay
                    ),
                ));

                // Reset the network state and prepare to start the game
                network_state.reset();
                session_options.delete = true;

                // Start the gameplay session
                GameplayPlugin::start_gameplay_session(
                    sessions,
                    session_runner,
                    online_socket.player_idx(),
                );
            }
        }
        NetworkGameStatus::MatchFound => {
            // Logic primarily happens in waiting for players
        }
        NetworkGameStatus::Idle => {
            // Reset the network state
            network_state.reset();
        }
    }
}
