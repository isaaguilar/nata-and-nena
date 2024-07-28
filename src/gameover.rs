use crate::despawn_screen;
use crate::game::TotalScore;
use crate::AppState;
use bevy::{prelude::*, render::view::RenderLayers};

#[derive(Component)]
pub struct OnGameOverScreen;

#[derive(Resource, Deref, DerefMut)]
struct GameOverTimer(Timer);

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameOver), gameover_setup)
            .add_systems(Update, countdown.run_if(in_state(AppState::GameOver)))
            .add_systems(
                OnExit(AppState::GameOver),
                despawn_screen::<OnGameOverScreen>,
            );
    }
}

#[derive(Component)]
struct GameOverCamera;

fn gameover_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut bg: ResMut<ClearColor>,
    total_score: Res<TotalScore>,
) {
    bg.0 = Color::BLACK;

    commands.spawn((
        OnGameOverScreen,
        GameOverCamera,
        Camera2dBundle {
            camera_2d: Camera2d { ..default() },
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        RenderLayers::from_layers(&[2, 3]),
    ));

    commands.spawn((
        RenderLayers::layer(2),
        OnGameOverScreen,
        TextBundle::from_sections(vec![
            TextSection {
                value: String::from("Final Score:\n\n"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 52.0,
                    color: Color::WHITE,
                    ..default()
                },
            },
            TextSection {
                value: total_score.to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 52.0,
                    color: Color::WHITE,
                    ..default()
                },
            },
        ])
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            top: Val::Percent(20.0),
            left: Val::Percent(20.0),

            ..default()
        }),
    ));

    commands.spawn((
        RenderLayers::layer(2),
        OnGameOverScreen,
        TextBundle::from_sections(vec![
            TextSection {
                value: String::from(
                    "Credits\n\nPlay testers:\n\n- Dominic Aguilar\n\n- Frankie Aguilar\n\n- Julien Aguilar\n\n",
                ),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            },
            TextSection {
                value: String::from(
                    "Lead Engineer, Music Producer & everything else\n\n- Isa Aguilar\n\n",
                ),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            },
            TextSection {
                value: String::from("Special thanks to ♥♥ Alma Aguilar (Wifey!) ♥♥"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            },
        ])
        .with_text_justify(JustifyText::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            top: Val::Percent(40.0),
            left: Val::Percent(2.0),

            ..default()
        }),
    ));

    commands.insert_resource(GameOverTimer(Timer::from_seconds(1.0, TimerMode::Once)));
}

fn countdown(
    mut app_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<GameOverTimer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Res<Gamepads>,
    button_inputs: Res<ButtonInput<GamepadButton>>,
) {
    if timer.tick(time.delta()).finished() {
        let gamepad = match gamepads.iter().next() {
            Some(gp) => gp,
            None => Gamepad::new(0),
        };
        let jump_key_just_pressed = button_inputs
            .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
            || keyboard_input.just_pressed(KeyCode::Space);

        if jump_key_just_pressed {
            app_state.set(AppState::Menu);
        }
    }
}
