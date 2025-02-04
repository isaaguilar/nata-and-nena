use bevy::audio::PlaybackMode;
use bevy::prelude::*;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rapier2d::prelude::*;
mod camera;
mod camera_tracking;
mod game;
mod gameover;
mod menu;
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

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn music(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioBundle {
        source: asset_server.load("water.mp3"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..default()
        },
        ..default()
    });
}

fn main() {
    App::new()
        .add_plugins((setup::WindowSetup, camera::CameraPlugin))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .init_state::<AppState>()
        .add_systems(Startup, music)
        .add_plugins((
            splash::SplashPlugin,
            menu::MenuPlugin,
            gameover::GameOverPlugin,
            game::PlatformPlugin,
        ))
        .run();
}
