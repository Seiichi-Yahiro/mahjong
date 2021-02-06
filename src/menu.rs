use crate::{GameState, StateStagePlugin};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub struct MenuStateStagePlugin;

impl StateStagePlugin<GameState> for MenuStateStagePlugin {
    fn build(&self, state_stage: &mut StateStage<GameState>) {
        let state = GameState::Menu;

        state_stage.set_update_stage(
            state,
            SystemStage::parallel().with_system(ui_system.system()),
        );
    }
}

fn ui_system(_world: &mut World, resources: &mut Resources) {
    let mut egui_context = resources.get_mut::<EguiContext>().unwrap();
    let ctx = &mut egui_context.ctx;

    egui::Window::new("")
        .title_bar(false)
        .fixed_rect(egui::Rect::from_center_size(
            ctx.available_rect().center(),
            egui::Vec2::new(250.0, 100.0),
        ))
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                let mut state = resources.get_mut::<State<GameState>>().unwrap();

                if ui.button("Play").clicked {
                    state.set_next(GameState::Play).unwrap();
                }

                if ui.button("Editor").clicked {
                    state.set_next(GameState::Editor).unwrap();
                }
            });

            ui.with_layout(
                egui::Layout::bottom_up(egui::Align::Center).with_cross_justify(true),
                |ui| {
                    if ui.button("Exit").clicked {
                        let mut exit_events = resources.get_mut::<Events<AppExit>>().unwrap();
                        exit_events.send(AppExit);
                    }
                },
            );
        });
}
