use bones_framework::networking::debug::NetworkDebugMenuState;
use bones_framework::prelude::*;
use std::time::Duration;
// gameplay_dimensions_scale_factor removed
use crate::SessionNames;
// bones_framework::prelude::* already imported above
use egui::{Color32, RichText}; // FontId, Pos2 removed, RichText added
use std::collections::VecDeque;

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


/// Struct which holds player pings over the past second to have smoother ping drawing
#[derive(HasSchema, Clone, Default)]
pub struct PlayerPings {
    pings: VecDeque<u128>,
}

impl PlayerPings {
    pub fn new() -> Self {
        PlayerPings {
            pings: VecDeque::with_capacity(60),
        }
    }

    pub fn add_averaged_ping(&mut self, ping: u128) {
        if self.pings.len() >= 60 {
            self.pings.pop_front();
        }
        self.pings.push_back(ping);
    }

    pub fn averaged_ping_last_second(&self) -> u128 {
        if self.pings.is_empty() {
            return 0;
        }
        let sum: u128 = self.pings.iter().sum();
        sum / self.pings.len() as u128
    }
}

/// Draws the ping and input delay at the top of the screen.
pub fn draw_ping_and_frame_delay(
    sessions: Res<Sessions>,
    ctx: Res<EguiCtx>,
    mut player_pings: ResMut<PlayerPings>,
) {
    if let Some(session) = sessions.get(SessionNames::GAMEPLAY) {
        if let Some(syncing_info) = session.world.get_resource::<SyncingInfo>() {
            player_pings.add_averaged_ping(syncing_info.averaged_ping());

            let ping = player_pings.averaged_ping_last_second();
            let frame_delay = syncing_info.local_frame_delay();

            let ping_text_str = format!("Ping: {}ms", ping);
            let frame_delay_text_str = format!("Input Delay: {}", frame_delay);

            // Area for ping and frame delay, positioned at top of screen
            egui::Area::new("ping_frame_delay_overlay") // Changed ID for clarity
                .fixed_pos(egui::pos2(10.0, 10.0)) // 10px margin from top-left
                .show(&ctx, |ui| {
                    // Calculate available width for the layout within the area
                    let screen_width = ctx.screen_rect().width();
                    // The layout width should be screen_width minus twice the margin (for left and right)
                    let layout_width = screen_width - (2.0 * 10.0);
                    ui.set_max_width(layout_width);

                    ui.horizontal(|ui| {
                        let text_size = 20.0;
                        // Left text (Ping)
                        let ping_text_rich = RichText::new(ping_text_str)
                            .size(text_size)
                            .color(Color32::WHITE);
                        ui.label(ping_text_rich);

                        // Right text (Frame Delay) - aligned to the right of the layout_width
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui_right| {
                            let frame_delay_text_rich = RichText::new(frame_delay_text_str)
                                .size(text_size)
                                .color(Color32::WHITE);
                            ui_right.label(frame_delay_text_rich);
                        });
                    });
                });
        }
    }
}
