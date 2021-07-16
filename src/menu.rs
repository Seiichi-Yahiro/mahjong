use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub struct MenuStatePlugin;

impl Plugin for MenuStatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        let state = GameState::Menu;

        app.add_system_set(SystemSet::on_update(state).with_system(ui_system.system()));
    }
}

fn ui_system(
    egui_ctx: ResMut<EguiContext>,
    mut game_state: ResMut<State<GameState>>,
    mut exit_events: EventWriter<AppExit>,
) {
    let ctx = egui_ctx.ctx();

    egui::Window::new("")
        .title_bar(false)
        .fixed_rect(egui::Rect::from_center_size(
            ctx.available_rect().center(),
            egui::Vec2::new(250.0, 100.0),
        ))
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                if ui.button("Play").clicked() {
                    game_state.set(GameState::Play).unwrap();
                }

                if ui.button("Editor").clicked() {
                    game_state.set(GameState::Editor).unwrap();
                }
            });

            ui.with_layout(
                egui::Layout::bottom_up(egui::Align::Center).with_cross_justify(true),
                |ui| {
                    if ui.button("Exit").clicked() {
                        exit_events.send(AppExit);
                    }
                },
            );
        });
}
