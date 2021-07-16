use crate::clamped_value::ClampedValue;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;
use bevy_mod_picking::PickingCameraBundle;

pub struct Camera3d;

pub fn create_camera_system(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 1.2, 0.001)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            perspective_projection: PerspectiveProjection {
                near: 0.01,
                far: 10.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Camera3d)
        .insert_bundle(PickingCameraBundle::default());
}

pub struct CameraState {
    pitch: ClampedValue<f32>,
    yaw: ClampedValue<f32>,
    distance: ClampedValue<f32>,
    rotation_speed: f32,
    scroll_speed: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
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
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    use bevy::math::*;

    if mouse_button_input.pressed(MouseButton::Right) {
        for MouseMotion { delta } in mouse_motion_events.iter() {
            let speed = time.delta_seconds() * camera_state.rotation_speed;
            camera_state.pitch -= delta.y * speed;
            camera_state.yaw -= delta.x * speed;
        }
    }

    for MouseWheel { y, .. } in mouse_wheel_events.iter() {
        let speed = time.delta_seconds() * camera_state.scroll_speed;
        camera_state.distance -= y * speed;
    }

    for mut transform in camera_query.iter_mut() {
        *transform = Transform::from_rotation(Quat::from_rotation_ypr(
            camera_state.yaw.value().to_radians(),
            camera_state.pitch.value().to_radians(),
            0.0,
        )) * Transform::from_xyz(0.0, camera_state.distance.value(), 0.001)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    }
}
