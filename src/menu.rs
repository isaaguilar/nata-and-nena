use crate::despawn_screen;
use crate::AppState;
use bevy::scene::ron::de;
use bevy::{prelude::*, render::view::RenderLayers};

#[derive(Component)]
pub struct MenuScreen;

#[derive(Component)]
pub struct MenuScrollControl {
    rate_limit_selection: Timer,
    selection_index: usize,
}

#[derive(Resource, Deref, DerefMut)]
pub struct TotalSeconds(pub f32);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), menu_setup)
            .insert_resource(TotalSeconds(180.0))
            .add_systems(
                Update,
                menu_selection_system.run_if(in_state(AppState::Menu)),
            )
            .add_systems(OnExit(AppState::Menu), despawn_screen::<MenuScreen>);
    }
}

#[derive(Component)]
struct MenuCamera;

fn menu_setup(mut commands: Commands, asset_server: Res<AssetServer>, mut bg: ResMut<ClearColor>) {
    bg.0 = Color::srgb(0.7, 0.7, 0.7);

    commands.spawn((
        MenuScreen,
        MenuCamera,
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
        MenuScreen,
        ImageBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(1.5),
                left: Val::Percent(27.5),
                height: Val::Px(100.0),
                // width: Val::Px(40.0),
                ..default()
            },
            image: UiImage {
                texture: asset_server.load("gamemenutitle.png"),
                ..default()
            },
            ..default()
        },
    ));

    commands.spawn((
        MenuScreen,
        MenuScrollControl {
            rate_limit_selection: Timer::from_seconds(0.2, TimerMode::Once),
            selection_index: 1,
        },
        RenderLayers::layer(2),
        TextBundle::from_sections(vec![
            TextSection {
                value: String::from("Game length:\n\n"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 32.0,
                    color: Color::BLACK,
                    ..default()
                },
            },
            TextSection {
                value: String::from("▶  180 seconds (3 minutes)\n\n"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 32.0,
                    color: Color::BLACK,
                    ..default()
                },
            },
            TextSection {
                value: String::from("  240 seconds (4 minutes)\n\n"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 32.0,
                    color: Color::BLACK,
                    ..default()
                },
            },
            TextSection {
                value: String::from("  300 seconds (5 minutes)"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 32.0,
                    color: Color::BLACK,
                    ..default()
                },
            },
        ])
        .with_text_justify(JustifyText::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            // align_items: AlignItems::Center,
            top: Val::Percent(20.0),
            left: Val::Percent(20.0),

            ..default()
        }),
    ));

    commands.spawn((
        MenuScreen,
        RenderLayers::layer(2),
        SpriteBundle {
            texture: asset_server.load("synopsys.png"),
            transform: Transform {
                translation: Vec3::new(0.0, -120.0, 10.0),
                ..default()
            },
            ..default()
        },
    ));

    commands.spawn((
        MenuScreen,
        RenderLayers::layer(2),
        SpriteBundle {
            texture: asset_server.load("gamecontroller.png"),
            transform: Transform {
                translation: Vec3::new(400.0, -270.0, 10.0),
                ..default()
            },
            ..default()
        },
    ));

    commands.spawn((
        MenuScreen,
        RenderLayers::layer(2),
        SpriteBundle {
            texture: asset_server.load("keyboard.png"),
            transform: Transform {
                translation: Vec3::new(-400.0, -270.0, 10.0),
                ..default()
            },
            ..default()
        },
    ));

    // corner indicator of selected player
}

fn menu_selection_system(
    time: Res<Time>,
    mut app_state: ResMut<NextState<AppState>>,
    mut total_seconds: ResMut<TotalSeconds>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Res<Gamepads>,
    button_inputs: Res<ButtonInput<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut time_selection_query: Query<(&mut Text, &mut MenuScrollControl)>,
) {
    let gamepad = match gamepads.iter().next() {
        Some(gp) => gp,
        None => Gamepad::new(0),
    };
    let left_stick_y = match axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY)) {
        Some(y) => y,
        None => 0.0,
    };

    let up_key_pressed = left_stick_y > 0.10 || keyboard_input.pressed(KeyCode::ArrowUp);
    let down_key_pressed = left_stick_y < -0.10 || keyboard_input.pressed(KeyCode::ArrowDown);
    let jump_key_just_pressed = button_inputs
        .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
        || keyboard_input.just_pressed(KeyCode::Space);

    let (mut text_sections, mut menu_select_control) = time_selection_query.single_mut();

    menu_select_control.rate_limit_selection.tick(time.delta());

    if menu_select_control.rate_limit_selection.finished()
        || menu_select_control.rate_limit_selection.just_finished()
    {
        if up_key_pressed {
            menu_select_control.selection_index -= 1;
            menu_select_control.rate_limit_selection.reset();
        } else if down_key_pressed {
            menu_select_control.selection_index += 1;
            menu_select_control.rate_limit_selection.reset();
        }

        if menu_select_control.selection_index < 1 {
            menu_select_control.selection_index = 1
        }
        if menu_select_control.selection_index > 3 {
            menu_select_control.selection_index = 3
        }

        for (usize, mut text_section) in text_sections.sections.iter_mut().enumerate() {
            if usize == 0 {
                continue;
            }
            if usize != menu_select_control.selection_index {
                text_section.value = text_section.value.replace("▶", "");
            } else {
                if !text_section.value.contains("▶") {
                    text_section.value = format!("▶{}", text_section.value);
                }
            }
        }
    }

    if jump_key_just_pressed {
        if menu_select_control.selection_index == 1 {
            total_seconds.0 = 180.0;
        } else if menu_select_control.selection_index == 2 {
            total_seconds.0 = 240.0;
        } else {
            total_seconds.0 = 300.0;
        }

        app_state.set(AppState::Game);
    }
}
