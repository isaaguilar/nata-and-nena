use bevy::prelude::*;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rapier2d::prelude::*;
mod camera;
mod camera_tracking;
mod game;
mod setup;
mod splash;

const PLAYER_MOVEMENT_SPEED: f32 = 250.;
pub const MAXIMUM_DOWNWARD_VELOCITY: f32 = 400.0;
pub const CHARACTER_DOWNWARD_VELOCITY_PER_FRAME: f32 = -600.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum AppState {
    #[default]
    Splash,
    Pause,
    Menu,
    Game,
    GameOver,
}

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }
        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn main() {
    App::new()
        .add_plugins((setup::WindowSetup, camera::CameraPlugin))
        .add_systems(PreUpdate, (close_on_esc))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .init_state::<AppState>()
        .add_plugins((splash::SplashPlugin, game::PlatformPlugin))
        .run();
}
