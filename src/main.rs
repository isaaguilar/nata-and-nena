use bevy::prelude::*;
mod camera;
mod setup;
mod splash;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    Splash,
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

fn main() {
    App::new()
        .add_plugins((setup::WindowSetup, camera::CameraPlugin))
        .add_systems(PreUpdate, (close_on_esc))
        .init_state::<GameState>()
        .add_plugins(splash::splash_plugin)
        .run();
}
