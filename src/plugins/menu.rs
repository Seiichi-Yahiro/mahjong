use crate::colors::{GUN_METAL, JADE, JADE_DARK, JADE_LIGHT};
use crate::plugins::assets::fonts::Fonts;
use crate::AppState;
use bevy::app::AppExit;
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_ui.in_schedule(OnEnter(AppState::Menu)))
            .add_system(button_interaction.in_set(OnUpdate(AppState::Menu)))
            .add_system(cleanup.in_schedule(OnExit(AppState::Menu)));
    }
}

const BORDER_COLOR: Color = GUN_METAL;
const NORMAL_BUTTON_COLOR: Color = JADE;
const HOVERED_BUTTON_COLOR: Color = JADE_LIGHT;
const CLICKED_BUTTON_COLOR: Color = JADE_DARK;
const TEXT_COLOR: Color = GUN_METAL;

#[derive(Component)]
struct MenuEntity;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component)]
enum ButtonType {
    Play,
    Editor,
    Exit,
}

fn setup_ui(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::NoWrap,
                ..default()
            },
            ..default()
        })
        .insert(MenuEntity)
        .with_children(|parent| {
            for (text, button_type) in [
                ("Play", ButtonType::Play),
                ("Editor", ButtonType::Editor),
                ("Exit", ButtonType::Exit),
            ] {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(33.0), Val::Px(100.0)),
                            margin: UiRect::vertical(Val::Px(10.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BORDER_COLOR.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                    padding: UiRect::all(Val::Px(5.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: NORMAL_BUTTON_COLOR.into(),
                                ..default()
                            })
                            .insert(button_type)
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    text,
                                    TextStyle {
                                        color: TEXT_COLOR,
                                        font_size: 42.0,
                                        font: fonts.roboto(),
                                    },
                                ));
                            });
                    });
            }
        });
}

fn button_interaction(
    mut button_query: Query<
        (&mut BackgroundColor, &Interaction, &ButtonType),
        (With<Button>, Changed<Interaction>),
    >,
    mut state: ResMut<NextState<AppState>>,
    mut exit_events: EventWriter<AppExit>,
) {
    for (mut background_color, interaction, button_type) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *background_color = CLICKED_BUTTON_COLOR.into();

                match *button_type {
                    ButtonType::Play => {
                        state.set(AppState::Play);
                    }
                    ButtonType::Editor => {
                        state.set(AppState::Editor);
                    }
                    ButtonType::Exit => exit_events.send_default(),
                }

                return;
            }
            Interaction::Hovered => {
                *background_color = HOVERED_BUTTON_COLOR.into();
            }
            Interaction::None => {
                *background_color = NORMAL_BUTTON_COLOR.into();
            }
        }
    }
}

fn cleanup(mut commands: Commands, menu_entity_query: Query<Entity, With<MenuEntity>>) {
    for entity in menu_entity_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
