use crate::solitaire::editor::tools::Tool;
use crate::solitaire::editor::PlacedTile;
use crate::StateStagePlugin;
use bevy::prelude::*;
use bevy_mod_picking::{Group, PickState, PickableMesh};

pub struct SelectToolStateStagePlugin;

impl StateStagePlugin<Tool> for SelectToolStateStagePlugin {
    fn build(&self, state_stage: &mut StateStage<Tool>) {
        let state = Tool::Select;

        state_stage
            .set_enter_stage(
                state,
                SystemStage::parallel().with_system(make_tiles_pickable_system.system()),
            )
            .set_update_stage(
                state,
                SystemStage::parallel()
                    .with_system(select_system.system())
                    .with_system(color_system.system())
                    .with_system(delete_system.system()),
            )
            .set_exit_stage(
                state,
                SystemStage::parallel().with_system(clean_up_system.system()),
            );
    }
}

struct Selected(bool);

fn make_tiles_pickable_system(commands: &mut Commands, query: Query<Entity, With<PlacedTile>>) {
    for entity in query.iter() {
        commands.insert(entity, (PickableMesh::default(), Selected(false)));
    }
}

fn select_system(
    pick_state: Res<PickState>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Selected, (With<PlacedTile>, With<PickableMesh>)>,
) {
    if mouse_button_inputs.just_pressed(MouseButton::Left) {
        if !keyboard_input.pressed(KeyCode::LShift) {
            for mut selected in query.iter_mut() {
                selected.0 = false;
            }
        }

        if let Some((entity, _)) = pick_state.top(Group::default()) {
            if let Ok(mut selected) = query.get_mut(*entity) {
                selected.0 = true;
            }
        }
    }
}

fn color_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Selected, &Handle<StandardMaterial>), (With<PlacedTile>, Changed<Selected>)>,
) {
    let default_color = Color::WHITE;
    //let hover_color = Color::rgb(0.3, 0.5, 0.8);
    let selected_color = Color::rgb(0.3, 0.8, 0.5);

    for (selected, handle) in query.iter() {
        if let Some(material) = materials.get_mut(handle) {
            material.albedo = if selected.0 {
                selected_color
            } else {
                default_color
            };
        }
    }
}

fn delete_system(
    commands: &mut Commands,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(Entity, &Selected), With<PlacedTile>>,
) {
    if keyboard_input.just_pressed(KeyCode::X) {
        for (entity, selected) in query.iter() {
            if selected.0 {
                commands.despawn(entity);
            }
        }
    }
}

fn clean_up_system(
    commands: &mut Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Handle<StandardMaterial>), (With<Selected>, With<PickableMesh>)>,
) {
    for (entity, handle) in query.iter() {
        if let Some(material) = materials.get_mut(handle) {
            material.albedo = Color::WHITE;
        }
        commands.remove::<(PickableMesh, Selected)>(entity);
    }
}
