use bevy::prelude::*;
mod camera;
mod camera_tracking;
mod game;
mod setup;
mod splash;

const PLAYER_MOVEMENT_SPEED: f32 = 250.;

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
        .init_state::<AppState>()
        .add_plugins((splash::SplashPlugin, game::PlatformPlugin))
        .run();
}
