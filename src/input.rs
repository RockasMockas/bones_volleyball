use bones_framework::input::{InputCollector, PlayerControls};
use bones_framework::networking::input::{NetworkInputConfig, NetworkPlayerControl};
use bones_framework::networking::proto::DenseMoveDirection;
use bones_framework::prelude::*;
use bytemuck::{Pod, Zeroable};
use std::array;

/// Maximum number of players supported
const MAX_PLAYERS: u32 = 2;

/// Represents the source of player control input, to keep things simple we join keyboard/gamepads together.
#[derive(Debug, Clone, Copy, Default, HasSchema, Hash, Eq, PartialEq)]
pub enum ControlSource {
    #[default]
    KeyboardAndGamepads,
}

/// Represents the current state of a player's controls
#[derive(HasSchema, Default, Clone, Copy, Debug)]
#[repr(C)]
pub struct PlayerControl {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub just_moved: bool,
    pub moving: bool,
    pub esc_start_pressed: bool,
    pub esc_start_just_pressed: bool,
    pub jump_pressed: bool,
    pub jump_just_pressed: bool,
    pub enter_pressed: bool,
    pub enter_just_pressed: bool,
}

/// A compact representation of player control
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
#[repr(transparent)]
pub struct DensePlayerControl(u32);

impl DensePlayerControl {
    /// Creates a new DensePlayerControl from the given inputs
    pub fn new(
        move_direction: Vec2,
        jump_pressed: bool,
        esc_start_pressed: bool,
        enter_pressed: bool,
    ) -> Self {
        let move_direction_u16: u16 = DenseMoveDirection(move_direction).into();
        let mut value = u32::from(move_direction_u16);
        if jump_pressed {
            value |= 1 << 16;
        }
        if esc_start_pressed {
            value |= 1 << 17;
        }
        if enter_pressed {
            value |= 1 << 18;
        }
        Self(value)
    }

    /// Returns the movement direction
    pub fn move_direction(&self) -> Vec2 {
        DenseMoveDirection::from((self.0 & 0xFFFF) as u16).0
    }

    /// Returns true if the jump button is pressed
    pub fn jump_pressed(&self) -> bool {
        (self.0 & (1 << 16)) != 0
    }

    /// Returns true if the escape/start button is pressed
    pub fn esc_start_pressed(&self) -> bool {
        (self.0 & (1 << 17)) != 0
    }

    /// Returns true if the enter button is pressed
    pub fn enter_pressed(&self) -> bool {
        (self.0 & (1 << 18)) != 0
    }
}

impl NetworkPlayerControl<DensePlayerControl> for PlayerControl {
    /// Converts PlayerControl to DensePlayerControl
    fn get_dense_input(&self) -> DensePlayerControl {
        let move_direction = Vec2::new(self.right - self.left, self.up - self.down);
        DensePlayerControl::new(
            move_direction,
            self.jump_pressed,
            self.esc_start_pressed,
            self.enter_pressed,
        )
    }

    /// Updates PlayerControl from DensePlayerControl
    fn update_from_dense(&mut self, new_control: &DensePlayerControl) {
        let move_direction = new_control.move_direction();
        self.left = (-move_direction.x).max(0.0);
        self.right = move_direction.x.max(0.0);
        self.up = move_direction.y.max(0.0);
        self.down = (-move_direction.y).max(0.0);

        self.left_pressed = self.left > 0.0;
        self.right_pressed = self.right > 0.0;
        self.up_pressed = self.up > 0.0;
        self.down_pressed = self.down > 0.0;

        let was_moving = self.moving;
        self.moving = move_direction.length_squared() > f32::MIN_POSITIVE;
        self.just_moved = !was_moving && self.moving;

        let was_jumping = self.jump_pressed;
        self.jump_pressed = new_control.jump_pressed();
        self.jump_just_pressed = !was_jumping && self.jump_pressed;

        let was_esc_start = self.esc_start_pressed;
        self.esc_start_pressed = new_control.esc_start_pressed();
        self.esc_start_just_pressed = !was_esc_start && self.esc_start_pressed;

        let was_enter = self.enter_pressed;
        self.enter_pressed = new_control.enter_pressed();
        self.enter_just_pressed = !was_enter && self.enter_pressed;
    }
}

/// Defines the key mappings for player controls
#[derive(HasSchema, Clone, Debug)]
pub struct PlayerControlMapping {
    pub left: Vec<KeyCode>,
    pub right: Vec<KeyCode>,
    pub up: Vec<KeyCode>,
    pub down: Vec<KeyCode>,
    pub jump: Vec<KeyCode>,
    pub esc_start: Vec<KeyCode>,
    pub enter: Vec<KeyCode>,
}

impl Default for PlayerControlMapping {
    fn default() -> Self {
        Self {
            left: vec![KeyCode::Left, KeyCode::A],
            right: vec![KeyCode::Right, KeyCode::D],
            up: vec![KeyCode::Up, KeyCode::W],
            down: vec![KeyCode::Down, KeyCode::S],
            jump: vec![KeyCode::Space, KeyCode::Z, KeyCode::L],
            esc_start: vec![KeyCode::Escape],
            enter: vec![KeyCode::Return],
        }
    }
}

/// Collects and manages player input
#[derive(HasSchema, Clone)]
pub struct PlayerInputCollector {
    current_controls: PlayerControl,
    last_controls: PlayerControl,
}

impl PlayerInputCollector {
    /// Returns the current player controls
    pub fn get_current_controls(&self) -> &PlayerControl {
        &self.current_controls
    }
}

impl Default for PlayerInputCollector {
    fn default() -> Self {
        Self {
            current_controls: default(),
            last_controls: default(),
        }
    }
}

impl<'a> InputCollector<'a, PlayerControlMapping, ControlSource, PlayerControl>
    for PlayerInputCollector
{
    /// Updates the "just pressed" states
    fn update_just_pressed(&mut self) {
        let last = self.last_controls;
        let current = &mut self.current_controls;

        current.esc_start_just_pressed = current.esc_start_pressed && !last.esc_start_pressed;
        current.moving =
            current.left > 0.01 || current.right > 0.01 || current.up > 0.01 || current.down > 0.01;
        current.jump_just_pressed = current.jump_pressed && !last.jump_pressed;
        current.just_moved = current.moving && !last.moving;
        current.enter_just_pressed = current.enter_pressed && !last.enter_pressed;
    }

    /// Advances to the next frame, updating last controls
    fn advance_frame(&mut self) {
        self.last_controls = self.current_controls.clone();
    }

    /// Applies inputs from keyboard and gamepad
    fn apply_inputs(
        &mut self,
        mapping: &PlayerControlMapping,
        keyboard: &KeyboardInputs,
        gamepad: &GamepadInputs,
    ) {
        // Keyboard input
        let current_control = &mut self.current_controls;

        // Update pressed state based on key events
        for event in &keyboard.key_events {
            match event.key_code {
                Set(key) if mapping.left.contains(&key) => {
                    current_control.left_pressed = event.button_state.pressed();
                }
                Set(key) if mapping.right.contains(&key) => {
                    current_control.right_pressed = event.button_state.pressed();
                }
                Set(key) if mapping.up.contains(&key) => {
                    current_control.up_pressed = event.button_state.pressed();
                }
                Set(key) if mapping.down.contains(&key) => {
                    current_control.down_pressed = event.button_state.pressed();
                }
                Set(key) if mapping.jump.contains(&key) => {
                    current_control.jump_pressed = event.button_state.pressed();
                }
                Set(key) if mapping.esc_start.contains(&key) => {
                    current_control.esc_start_pressed = event.button_state.pressed();
                }
                Set(key) if mapping.enter.contains(&key) => {
                    current_control.enter_pressed = event.button_state.pressed();
                }
                _ => {}
            }
        }

        // Set movement values based on pressed state
        current_control.left = if current_control.left_pressed {
            1.0
        } else {
            0.0
        };
        current_control.right = if current_control.right_pressed {
            1.0
        } else {
            0.0
        };
        current_control.up = if current_control.up_pressed { 1.0 } else { 0.0 };
        current_control.down = if current_control.down_pressed {
            1.0
        } else {
            0.0
        };

        // Now apply gamepad input
        for event in &gamepad.gamepad_events {
            match event {
                GamepadEvent::Axis(axis_event) => {
                    if axis_event.axis == GamepadAxis::LeftStickX {
                        if axis_event.value < -0.2 {
                            current_control.left = 1.0;
                            current_control.right = 0.0;
                            current_control.left_pressed = true;
                            current_control.right_pressed = false;
                        } else if axis_event.value > 0.2 {
                            current_control.right = 1.0;
                            current_control.left = 0.0;
                            current_control.right_pressed = true;
                            current_control.left_pressed = false;
                        } else {
                            current_control.left = 0.0;
                            current_control.right = 0.0;
                            current_control.left_pressed = false;
                            current_control.right_pressed = false;
                        }
                    } else if axis_event.axis == GamepadAxis::LeftStickY {
                        if axis_event.value < -0.2 {
                            current_control.down = 1.0;
                            current_control.up = 0.0;
                            current_control.down_pressed = true;
                            current_control.up_pressed = false;
                        } else if axis_event.value > 0.2 {
                            current_control.up = 1.0;
                            current_control.down = 0.0;
                            current_control.up_pressed = true;
                            current_control.down_pressed = false;
                        } else {
                            current_control.up = 0.0;
                            current_control.down = 0.0;
                            current_control.up_pressed = false;
                            current_control.down_pressed = false;
                        }
                    }
                }
                GamepadEvent::Button(button_event) => match button_event.button {
                    GamepadButton::DPadLeft => {
                        current_control.left = if button_event.value > 0.2 { 1.0 } else { 0.0 };
                        current_control.left_pressed = button_event.value > 0.2;
                    }
                    GamepadButton::DPadRight => {
                        current_control.right = if button_event.value > 0.2 { 1.0 } else { 0.0 };
                        current_control.right_pressed = button_event.value > 0.2;
                    }
                    GamepadButton::DPadUp => {
                        current_control.up = if button_event.value > 0.2 { 1.0 } else { 0.0 };
                        current_control.up_pressed = button_event.value > 0.2;
                    }
                    GamepadButton::DPadDown => {
                        current_control.down = if button_event.value > 0.2 { 1.0 } else { 0.0 };
                        current_control.down_pressed = button_event.value > 0.2;
                    }
                    GamepadButton::South => {
                        current_control.jump_pressed = button_event.value > 0.5;
                    }
                    GamepadButton::Start => {
                        current_control.esc_start_pressed = button_event.value > 0.5;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    /// Gets the current control state
    fn get_control(&self, _player_idx: usize, _control_source: ControlSource) -> &PlayerControl {
        &self.current_controls
    }
}

/// Manages inputs for all players in a match
#[derive(Clone, Debug, HasSchema)]
pub struct MatchInputs {
    pub players: [PlayerControl; MAX_PLAYERS as usize],
}

impl Default for MatchInputs {
    fn default() -> Self {
        Self {
            players: array::from_fn(|_| default()),
        }
    }
}

impl PlayerControls<'_, PlayerControl> for MatchInputs {
    type ControlSource = ControlSource;
    type ControlMapping = PlayerControlMapping;
    type InputCollector = PlayerInputCollector;

    /// Updates controls for all players
    fn update_controls(&mut self, collector: &mut PlayerInputCollector) {
        (0..MAX_PLAYERS as usize).for_each(|i| {
            self.players[i] = collector
                .get_control(i, ControlSource::KeyboardAndGamepads)
                .clone();
        });
    }

    /// Gets the control source for a player
    fn get_control_source(&self, _player_idx: usize) -> Option<ControlSource> {
        Some(ControlSource::KeyboardAndGamepads)
    }

    /// Gets the control state for a player
    fn get_control(&self, player_idx: usize) -> &PlayerControl {
        &self.players[player_idx]
    }

    /// Gets mutable control state for a player
    fn get_control_mut(&mut self, player_idx: usize) -> &mut PlayerControl {
        &mut self.players[player_idx]
    }
}

/// Configuration for network input handling, used with the ggrs session runner
pub struct GameNetworkInputConfig;

impl<'a> NetworkInputConfig<'a> for GameNetworkInputConfig {
    type Dense = DensePlayerControl;
    type Control = PlayerControl;
    type PlayerControls = MatchInputs;
    type InputCollector = PlayerInputCollector;
}
