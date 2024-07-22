use crate::GameState;
use bevy::{log::info, prelude::*, render::view::RenderLayers};

#[derive(Component)]
pub struct MainCamera {
    pub time_delta: f32,
}

fn camera_setup(mut commands: Commands) {
    commands
        .spawn((
            MainCamera { time_delta: 0.0 },
            Camera2dBundle {
                camera_2d: Camera2d { ..default() },
                camera: Camera {
                    order: 0,
                    ..default()
                },
                ..default()
            },
            RenderLayers::from_layers(&[0, 1]),
        ))
        .insert(Transform::from_xyz(0., 0., 0.));
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup);
    }
}
