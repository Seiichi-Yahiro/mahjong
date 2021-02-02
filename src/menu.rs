use crate::ui::UiAssetData;
use crate::{GameState, StateStagePlugin};
use bevy::app::AppExit;
use bevy::prelude::*;

pub struct MenuStateStagePlugin;

impl StateStagePlugin<GameState> for MenuStateStagePlugin {
    fn build(&self, state_stage: &mut StateStage<GameState>) {
        let state = GameState::Menu;

        state_stage
            .set_enter_stage(
                state,
                SystemStage::parallel().with_system(create_ui_system.system()),
            )
            .set_update_stage(
                state,
                SystemStage::parallel().with_system(button_system.system()),
            )
            .set_exit_stage(
                state,
                SystemStage::parallel().with_system(clean_up_system.system()),
            );
    }
}

struct MenuEntity;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Button {
    Play,
    Editor,
    Exit,
}

fn create_ui_system(commands: &mut Commands, ui_asset_data: Res<UiAssetData>) {
    let button_width = 150.0;
    let button_height = 50.0;

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: ui_asset_data.get_transparent(),
            ..Default::default()
        })
        .with(MenuEntity)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(button_width), Val::Px(4.0 * button_height)),
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: ui_asset_data.get_transparent(),
                    ..Default::default()
                })
                .with_children(|group| {
                    let button_bundle = ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(button_width), Val::Px(button_height)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        material: ui_asset_data.get_button_mat(),
                        ..Default::default()
                    };

                    let text_bundle = |text: &str| TextBundle {
                        text: Text {
                            value: text.to_string(),
                            font: ui_asset_data.get_font(),
                            style: TextStyle {
                                font_size: 40.0,
                                color: Color::BLACK,
                                alignment: Default::default(),
                            },
                        },
                        ..Default::default()
                    };

                    group
                        .spawn(button_bundle.clone())
                        .with(Button::Play)
                        .with_children(|button| {
                            button.spawn(text_bundle("Play"));
                        });

                    group
                        .spawn(button_bundle.clone())
                        .with(Button::Editor)
                        .with_children(|button| {
                            button.spawn(text_bundle("Editor"));
                        });

                    group
                        .spawn(button_bundle)
                        .with(Button::Exit)
                        .with_children(|button| {
                            button.spawn(text_bundle("Exit"));
                        });
                });
        });
}

fn button_system(
    ui_asset_data: Res<UiAssetData>,
    mut exit_events: ResMut<Events<AppExit>>,
    mut state: ResMut<State<GameState>>,
    mut query: Query<(&Interaction, &mut Handle<ColorMaterial>, &Button), Mutated<Interaction>>,
) {
    for (&interaction, mut material, &button) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                *material = ui_asset_data.get_button_hovered_mat();
            }
            Interaction::None => {
                *material = ui_asset_data.get_button_mat();
            }
            Interaction::Clicked => match button {
                Button::Play => {}
                Button::Editor => {
                    state.set_next(GameState::Editor).unwrap();
                }
                Button::Exit => {
                    exit_events.send(AppExit);
                }
            },
        }
    }
}

fn clean_up_system(commands: &mut Commands, query: Query<Entity, With<MenuEntity>>) {
    for entity in query.iter() {
        commands.despawn_recursive(entity);
    }
}
