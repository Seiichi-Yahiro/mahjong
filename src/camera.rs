use crate::clamped_value::ClampedValue;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;
use bevy_mod_picking::PickSource;

pub struct Camera3d;

pub fn create_camera_system(commands: &mut Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.2, 0.001))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::unit_y()),
            perspective_projection: PerspectiveProjection {
                near: 0.01,
                far: 10.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Camera3d)
        .with(PickSource::default());
}

pub struct CameraState {
    mouse_motion_event_reader: EventReader<MouseMotion>,
    mouse_wheel_event_reader: EventReader<MouseWheel>,
    pitch: ClampedValue<f32>,
    yaw: ClampedValue<f32>,
    distance: ClampedValue<f32>,
    rotation_speed: f32,
    scroll_speed: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            mouse_motion_event_reader: Default::default(),
            mouse_wheel_event_reader: Default::default(),
            pitch: ClampedValue::new(0.0, 0.0, 90.0),
            yaw: ClampedValue::new(0.0, -45.0, 45.0),
            distance: ClampedValue::new(1.2, 0.5, 1.2),
            rotation_speed: 3.0,
            scroll_speed: 1.0,
        }
    }
}

pub fn camera_movement_system(
    time: Res<Time>,
    mut camera_state: Local<CameraState>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mouse_wheel_events: Res<Events<MouseWheel>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    use bevy::math::*;

    if mouse_button_input.pressed(MouseButton::Right) {
        for MouseMotion { delta } in camera_state
            .mouse_motion_event_reader
            .iter(&mouse_motion_events)
        {
            let speed = time.delta_seconds() * camera_state.rotation_speed;
            camera_state.pitch -= delta.y * speed;
            camera_state.yaw -= delta.x * speed;
        }
    }

    for MouseWheel { y, .. } in camera_state
        .mouse_wheel_event_reader
        .iter(&mouse_wheel_events)
    {
        let speed = time.delta_seconds() * camera_state.scroll_speed;
        camera_state.distance -= y * speed;
    }

    for mut transform in camera_query.iter_mut() {
        *transform =
            Transform::from_rotation(Quat::from_rotation_ypr(
                camera_state.yaw.value().to_radians(),
                camera_state.pitch.value().to_radians(),
                0.0,
            )) * Transform::from_translation(Vec3::new(0.0, camera_state.distance.value(), 0.001))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::unit_y());
    }
}
