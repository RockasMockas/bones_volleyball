use bones_framework::networking::debug::{NetworkDebug, NetworkDebugMenuState};
use bones_framework::prelude::*;
use egui::{Color32, Frame, RichText, Stroke, Vec2};
use std::time::Duration;

/// Resource for the networking debug menu state
#[derive(HasSchema, Clone, Debug)]
pub struct NetworkingDebugMenuState {
    pub detailed_menu_open: bool,
    pub detailed_menu_last_toggle: Instant,
    pub simple_menu_open: bool,
    pub simple_menu_last_toggle: Instant,
}

impl Default for NetworkingDebugMenuState {
    fn default() -> Self {
        Self {
            detailed_menu_open: false,
            detailed_menu_last_toggle: Instant::now(),
            simple_menu_open: true,
            simple_menu_last_toggle: Instant::now(),
        }
    }
}

/// System displaying a simplified network debug overlay
pub fn simple_network_debug_overlay(
    diagnostics: Res<NetworkDebug>,
    debug_menu_state: Res<NetworkingDebugMenuState>,
    egui_ctx: ResMut<EguiCtx>,
) {
    if debug_menu_state.simple_menu_open {
        egui::Area::new("simple_network_debug")
            .fixed_pos((10.0, 10.0))
            .show(&egui_ctx, |ui| {
                Frame::none()
                    .fill(Color32::from_black_alpha(0))
                    .stroke(Stroke::new(1.0, Color32::BLACK))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if let Some((_, stats)) = diagnostics.network_stats.first() {
                                add_text_with_shadow(ui, &format!("Ping: {} ms", stats.ping));
                                // add_text_with_shadow(
                                //     ui,
                                //     &format!("Sending: {:.2} kbps", stats.kbps_sent),
                                // );
                            } else {
                                add_text_with_shadow(ui, "No network stats available");
                            }
                        });
                    });
            });
    }
}

/// Helper function to add text with a shadow effect
fn add_text_with_shadow(ui: &mut egui::Ui, text: &str) {
    let shadow_color = Color32::from_black_alpha(180);
    let text_color = Color32::WHITE;
    let shadow_offset = 1.0;

    // Draw shadow
    for offset in [
        Vec2::new(-shadow_offset, -shadow_offset),
        Vec2::new(shadow_offset, -shadow_offset),
        Vec2::new(-shadow_offset, shadow_offset),
        Vec2::new(shadow_offset, shadow_offset),
    ] {
        ui.label(
            RichText::new(text)
                .color(shadow_color)
                .text_style(egui::TextStyle::Monospace),
        );
        ui.allocate_ui_at_rect(ui.min_rect().translate(offset), |ui| {
            ui.label(
                RichText::new(text)
                    .color(shadow_color)
                    .text_style(egui::TextStyle::Monospace),
            );
        });
    }

    // Draw main text
    ui.label(
        RichText::new(text)
            .color(text_color)
            .text_style(egui::TextStyle::Monospace),
    );
}

/// Activates displaying the networking debug overlays with debounce
pub fn activate_networking_debug_overlays(
    mut debug_menu_state: ResMut<NetworkingDebugMenuState>,
    keyboard_input: Res<KeyboardInputs>,
    ctx: ResMut<EguiCtx>,
) {
    const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);
    let current_time = Instant::now();

    for input in &keyboard_input.key_events {
        match input.key_code {
            Set(KeyCode::F2) => {
                if current_time.duration_since(debug_menu_state.simple_menu_last_toggle)
                    >= DEBOUNCE_DURATION
                {
                    // Toggle the simple menu state
                    debug_menu_state.simple_menu_open = !debug_menu_state.simple_menu_open;
                    debug_menu_state.simple_menu_last_toggle = current_time;
                }
                break;
            }
            Set(KeyCode::F1) => {
                if current_time.duration_since(debug_menu_state.detailed_menu_last_toggle)
                    >= DEBOUNCE_DURATION
                {
                    // Toggle the detailed menu state
                    debug_menu_state.detailed_menu_open = !debug_menu_state.detailed_menu_open;
                    debug_menu_state.detailed_menu_last_toggle = current_time;

                    // Set the egui context state for the detailed menu
                    ctx.set_state(NetworkDebugMenuState {
                        open: debug_menu_state.detailed_menu_open,
                    });
                }
                break;
            }
            _ => {}
        }
    }
}
