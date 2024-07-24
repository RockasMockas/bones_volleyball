use super::{
    ball_movement, ball_net_collision, ball_player_collision, create_circle_path, gameplay_ui::*,
    player_movement, update_ball_visibility, Ball, Floor, LocalPlayer, Net, Player,
};
use crate::{
    input::{MatchInputs, PlayerControlMapping, PlayerInputCollector},
    menu::*,
    GameMeta, SessionNames,
};
use bones_framework::prelude::*;

/// The score required to win the match
pub const TARGET_SCORE: u32 = 15;
/// The Y-coordinate of the ground level
pub const GROUND_LEVEL: f32 = -244.0;
/// The gravity constant for the game
pub const GRAVITY: f32 = 0.175 * 1.5 * 1.5;
/// The movement speed of the players
pub const MOVE_SPEED: f32 = 4.35;
/// The initial velocity of a player's jump
pub const JUMP_VELOCITY: f32 = 7.5 * 1.5;
/// The left boundary of the playfield
pub const LEFT_BOUNDARY: f32 = -530.0;
/// The right boundary of the playfield
pub const RIGHT_BOUNDARY: f32 = 510.0;
/// The center boundary of the playfield
pub const CENTER_BOUNDARY: f32 = 0.0;
/// The width of the net
pub const NET_WIDTH: f32 = 10.0;
/// The height of the net
pub const NET_HEIGHT: f32 = 62.00;
/// The width of a player sprite
pub const PLAYER_WIDTH: f32 = 90.0;
/// The height of a player sprite
pub const PLAYER_HEIGHT: f32 = 14.0;
/// The radius of the ball
pub const BALL_RADIUS: f32 = 10.0;
/// The bounce factor for the ball when hitting surfaces
pub const BALL_BOUNCE_FACTOR: f32 = 0.8;
/// The bounce factor for the ball when hitting players
pub const PLAYER_BOUNCE_FACTOR: f32 = 1.2;
/// The maximum speed of the ball
pub const MAX_BALL_SPEED: f32 = 11.25 * 1.5;

/// Metadata for gameplay
#[derive(HasSchema, Default, Clone, Debug)]
#[repr(C)]
#[type_data(metadata_asset("gameplay"))]
pub struct GameplayMeta {
    pub player_sprite: Handle<Image>,
    pub floor_sprite: Handle<Image>,
    pub net_sprite: Handle<Image>,
}

/// Represents the current state of the match
#[derive(HasSchema, Clone, Debug, Default)]
pub struct MatchState {
    player_scores: [u32; 2],
    target_score: u32,
}

impl MatchState {
    /// Creates a new MatchState with the given target score
    pub fn new(target_score: u32) -> Self {
        Self {
            player_scores: [0, 0],
            target_score,
        }
    }

    /// Gets the score of the specified player
    pub fn get_player_score(&self, player_idx: usize) -> u32 {
        self.player_scores[player_idx]
    }

    /// Increments the score of the specified player
    pub fn increment_player_score(&mut self, player_idx: usize) {
        self.player_scores[player_idx] += 1;
    }

    /// Checks if there's a winner and returns their index if so
    pub fn check_for_match_winner(&self) -> Option<usize> {
        self.player_scores
            .iter()
            .position(|&score| score >= self.target_score)
    }

    /// Checks if the match is finished
    pub fn is_finished(&self) -> bool {
        self.check_for_match_winner().is_some()
    }
}

/// Plugin for managing the gameplay session
pub struct GameplayPlugin {
    pub session_runner: Box<dyn SessionRunner>,
}

impl GameplayPlugin {
    /// Starts gameplay by initializing both a gameplay and gameplay_ui session
    pub fn start_gameplay_session(
        mut sessions: ResMut<Sessions>,
        session_runner: Box<dyn SessionRunner>,
        local_player_idx: u32,
    ) {
        // First setup the gameplay ui session
        initialize_gameplay_ui_session(&mut sessions);

        // Setup gameplay session with resources that require inputs
        let gameplay_session = sessions.create(SessionNames::GAMEPLAY);
        gameplay_session
            .world
            .insert_resource(MatchState::new(TARGET_SCORE));
        gameplay_session.world.insert_resource(LocalPlayer {
            idx: local_player_idx,
        });

        // Install the gameplay plugin
        let gameplay_plugin = GameplayPlugin { session_runner };
        gameplay_session.install_plugin(gameplay_plugin);
    }
}

impl SessionPlugin for GameplayPlugin {
    /// Installs the gameplay plugin, initializing resources and systems
    fn install(self, session: &mut Session) {
        // Initialize resources that don't require inputs
        session.world.init_resource::<MatchInputs>();
        session.world.init_resource::<PlayerInputCollector>();
        session.world.init_resource::<PlayerControlMapping>();

        // Add default plugin + systems
        session.install_plugin(DefaultSessionPlugin);
        session
            .add_startup_system(gameplay_startup)
            .add_system_to_stage(Update, player_movement)
            .add_system_to_stage(Update, ball_movement)
            .add_system_to_stage(Update, ball_player_collision)
            .add_system_to_stage(Update, ball_net_collision)
            .add_system_to_stage(Update, update_ball_visibility)
            .add_system_to_stage(Update, handle_escape);

        session.runner = self.session_runner;
    }
}

/// Initializes the gameplay entities and components
fn gameplay_startup(
    mut entities: ResMut<Entities>,
    mut sprites: CompMut<Sprite>,
    mut transforms: CompMut<Transform>,
    mut cameras: CompMut<Camera>,
    mut players: CompMut<Player>,
    mut balls: CompMut<Ball>,
    mut floors: CompMut<Floor>,
    mut nets: CompMut<Net>,
    mut paths: CompMut<Path2d>,
    meta: Root<GameMeta>,
) {
    // Create and set up the camera
    let camera_ent = spawn_default_camera(&mut entities, &mut transforms, &mut cameras);
    if let Some(camera) = cameras.get_mut(camera_ent) {
        camera.size = CameraSize::FixedHeight(580.0);
    }
    if let Some(camera_transform) = transforms.get_mut(camera_ent) {
        camera_transform.translation = Vec3::new(0.0, -20.0, 1.0);
    }

    // Create the floor
    let floor_ent = entities.create();
    floors.insert(floor_ent, Floor);
    sprites.insert(
        floor_ent,
        Sprite {
            image: meta.gameplay.floor_sprite,
            ..default()
        },
    );
    transforms.insert(
        floor_ent,
        Transform::from_translation(vec3(0.0, -365.0, 0.0)),
    );

    // Create the net
    let net_ent = entities.create();
    nets.insert(net_ent, Net);
    sprites.insert(
        net_ent,
        Sprite {
            image: meta.gameplay.net_sprite,
            ..default()
        },
    );
    transforms.insert(net_ent, Transform::from_translation(vec3(0.0, -259.0, 0.0)));

    // Create Player 1 (left side)
    let player1_ent = entities.create();
    transforms.insert(
        player1_ent,
        Transform::from_translation(vec3(-290.0, GROUND_LEVEL, 0.0)),
    );
    sprites.insert(
        player1_ent,
        Sprite {
            image: meta.gameplay.player_sprite,
            ..default()
        },
    );
    players.insert(
        player1_ent,
        Player {
            idx: 0,
            ..default()
        },
    );
    if let Some(transform) = transforms.get_mut(player1_ent) {
        transform.scale = Vec3::new(2.0, 2.0, 2.0);
    }

    // Create Player 2 (right side)
    let player2_ent = entities.create();
    transforms.insert(
        player2_ent,
        Transform::from_translation(vec3(290.0, GROUND_LEVEL, 0.0)),
    );
    sprites.insert(
        player2_ent,
        Sprite {
            image: meta.gameplay.player_sprite,
            flip_x: true,
            ..default()
        },
    );
    players.insert(
        player2_ent,
        Player {
            idx: 1,
            ..default()
        },
    );
    if let Some(transform) = transforms.get_mut(player2_ent) {
        transform.scale = Vec3::new(2.0, 2.0, 2.0);
    }

    // Create the ball
    let ball_ent = entities.create();
    let mut ball_transform = Transform::from_translation(vec3(-290.0, 0.0, 0.0));
    let mut ball = Ball {
        velocity: Vec2::ZERO,
    };
    ball.reset(false, &mut ball_transform);
    transforms.insert(ball_ent, ball_transform);
    balls.insert(ball_ent, ball);
    paths.insert(
        ball_ent,
        create_circle_path(
            BALL_RADIUS,
            Color::Rgba {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
        ),
    );
}

/// Handles the escape key press to return to the main menu
fn handle_escape(
    match_inputs: Res<MatchInputs>,
    mut sessions: ResMut<Sessions>,
    mut session_options: ResMut<SessionOptions>,
) {
    for player_idx in 0..2 {
        let player_control = match_inputs.get_control(player_idx);
        if player_control.esc_start_just_pressed {
            session_options.delete = true;
            sessions
                .create(SessionNames::MAIN_MENU)
                .install_plugin(menu_plugin);
            break;
        }
    }
}
