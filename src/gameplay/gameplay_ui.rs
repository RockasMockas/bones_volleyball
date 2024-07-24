use super::{activate_networking_debug_overlays, MatchState, NetworkingDebugMenuState};
use crate::SessionNames;
use bones_framework::networking::debug::network_debug_window;
use bones_framework::prelude::*;
use egui::{Color32, RichText};

/// Initializes the gameplay_ui session
pub fn initialize_gameplay_ui_session(sessions: &mut ResMut<Sessions>) {
    let gameplay_ui_session = sessions.create(SessionNames::GAMEPLAY_UI);
    gameplay_ui_session
        .world
        .init_resource::<NetworkingDebugMenuState>();

    gameplay_ui_session
        .add_system_to_stage(CoreStage::First, network_debug_window)
        .add_system_to_stage(Update, draw_winning_text)
        .add_system_to_stage(Update, draw_score_system)
        .add_system_to_stage(Update, activate_networking_debug_overlays);
}

pub fn draw_winning_text(sessions: Res<Sessions>, ctx: Res<EguiCtx>) {
    if let Some(session) = sessions.get(SessionNames::GAMEPLAY) {
        let match_state = session
            .world
            .get_resource::<MatchState>()
            .expect("MatchState resource not found");
        if let Some(winner_idx) = match_state.check_for_match_winner() {
            let winner_text = if winner_idx == 0 {
                "Left Player Wins!"
            } else {
                "Right Player Wins!"
            };

            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(&ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        let text = RichText::new(winner_text)
                            .color(Color32::YELLOW)
                            .size(60.0)
                            .strong();
                        ui.label(text);
                    });
                });
        }
    }
}

pub fn draw_score_system(sessions: Res<Sessions>, ctx: Res<EguiCtx>) {
    if let Some(session) = sessions.get(SessionNames::GAMEPLAY) {
        let match_state = session
            .world
            .get_resource::<MatchState>()
            .expect("MatchState resource not found");
        egui::TopBottomPanel::top("score_panel")
            .frame(egui::Frame::none())
            .show(&ctx, |ui| {
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    let score_text = format!(
                        "{} - {}",
                        match_state.get_player_score(0),
                        match_state.get_player_score(1)
                    );
                    let text = RichText::new(score_text)
                        .size(72.0)
                        .color(Color32::WHITE)
                        .strong();
                    ui.label(text);
                });
                ui.add_space(550.0);
            });
    }
}
