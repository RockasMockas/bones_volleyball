use bones_framework::networking::debug::NetworkDebugMenuState;
use bones_framework::prelude::*;
use std::time::Duration;

/// Resource for the networking debug menu state
#[derive(HasSchema, Clone, Debug)]
pub struct VisualizedNetworkingDebugMenuState {
    pub detailed_menu_open: bool,
    pub detailed_menu_last_toggle: Instant,
}

impl Default for VisualizedNetworkingDebugMenuState {
    fn default() -> Self {
        Self {
            detailed_menu_open: false,
            detailed_menu_last_toggle: Instant::now(),
        }
    }
}

/// Activates displaying the networking debug overlays with debounce on keybind
pub fn activate_networking_debug_overlays(
    mut visualized_debug_menu_state: ResMut<VisualizedNetworkingDebugMenuState>,
    keyboard_input: Res<KeyboardInputs>,
    ctx: ResMut<EguiCtx>,
) {
    const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);
    let current_time = Instant::now();

    for input in &keyboard_input.key_events {
        if let Set(KeyCode::F1) = input.key_code {
            if current_time.duration_since(visualized_debug_menu_state.detailed_menu_last_toggle)
                >= DEBOUNCE_DURATION
            {
                // Toggle the detailed menu state
                visualized_debug_menu_state.detailed_menu_open = !visualized_debug_menu_state.detailed_menu_open;
                visualized_debug_menu_state.detailed_menu_last_toggle = current_time;

                // Set the egui context state for the detailed menu
                ctx.set_state(NetworkDebugMenuState {
                    open: visualized_debug_menu_state.detailed_menu_open,
                });
            }
            break;
        }
    }
}
