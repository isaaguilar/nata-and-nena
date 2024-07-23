use crate::camera_tracking;
use crate::despawn_screen;
use crate::AppState;
use crate::PLAYER_MOVEMENT_SPEED;
use bevy::{prelude::*, render::texture::ImageLoader};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Resource)]
enum GamePhase {
    #[default]
    Open,
    Dialog,
    Init,
    Begining,
    Middle,
    End,
}

#[derive(Component)]
pub struct Game;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct ActivePlayer;
#[derive(Component)]
pub struct Cloud;

#[derive(Component)]
pub struct DialogBox;

#[derive(Component)]
pub struct Dialog {
    pub image: Handle<Image>,
    pub dialog: Text,
}

// TODO add a timer
// TODO add a point resource

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("person.png");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(40, 50),
        1,
        1,
        Some(UVec2::new(0, 0)),
        Some(UVec2::new(0, 0)),
    );

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn((
        Player,
        ActivePlayer,
        Game,
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 400.0, 10.0),
                ..default()
            },
            sprite: Sprite {
                flip_x: false,
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("cloud1.png");

    commands.spawn((
        Game,
        Cloud,
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 1200.0, 0.0),
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("raincloud.png");

    commands.spawn((
        Game,
        Cloud,
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(-200.0, 800.0, 0.0),
                ..default()
            },
            ..default()
        },
    ));

    commands.spawn((
        Game,
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(-200.0, 11200.0, 0.0),
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("gradient.png");

    commands.spawn((
        Game,
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 6000.0 - 750., -10.0),
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("chalchiuhtlicue-bust.png");

    let text_style = TextStyle {
        font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
        font_size: 18.0,
        ..default()
    };
    commands.spawn((
        Game,
        Dialog {
            image: texture_handle.clone(),
            dialog: Text {
                sections: vec![TextSection {
                    value: String::from("Hello, Frankie."),
                    style: text_style.clone(),
                }],
                ..default()
            },
        },
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, -100.0, -9.0),
                ..default()
            },
            ..default()
        },
    ));
}

fn cloud_movement(time: Res<Time>, mut clouds: Query<(&mut Transform), With<Cloud>>) {
    for mut transform in clouds.iter_mut() {
        if transform.translation.x < -680.0 {
            transform.translation.x = 680.
        }
        transform.translation.x -= 80. * time.delta_seconds();
    }
}

fn keyboard_input_system(
    mut game_phase: ResMut<GamePhase>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Res<Gamepads>,
    button_inputs: Res<ButtonInput<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut player_query: Query<(&mut Transform, &mut TextureAtlas, &mut Sprite), With<Player>>,
) {
    let gamepad = match gamepads.iter().next() {
        Some(gp) => gp,
        None => Gamepad::new(0),
    };

    let mut total_y = 0.0;
    let mut total_x = 0.0;

    // *ptime = time::PhysicsTime::default();
    let pause_key_just_pressed = button_inputs
        .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::Start))
        || keyboard_input.just_pressed(KeyCode::KeyP);

    let left_stick_x = match axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX)) {
        Some(x) => x,
        None => 0.0,
    };

    let left_stick_y = match axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY)) {
        Some(y) => y,
        None => 0.0,
    };

    // TODO figure out why d-pad is not working

    // let left_stick_y = match axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY)) {
    //     Some(y) => y,
    //     None => 0.0,
    // };

    // let jump_key_just_pressed = button_inputs
    //     .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
    //     || keyboard_input.just_pressed(KeyCode::Space);
    let jump_key_just_pressed = button_inputs
        .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
        || keyboard_input.just_pressed(KeyCode::Space);
    // let run_key_just_pressed = button_inputs
    //     .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::West))
    //     || keyboard_input.just_pressed(KeyCode::ShiftLeft);
    let run_key_pressed = button_inputs
        .pressed(GamepadButton::new(gamepad, GamepadButtonType::West))
        || keyboard_input.pressed(KeyCode::ShiftLeft);
    // let left_key_just_pressed =
    //     left_stick_x < 0.0 || keyboard_input.just_pressed(KeyCode::ArrowLeft);
    let left_key_pressed = left_stick_x < 0.0 || keyboard_input.pressed(KeyCode::ArrowLeft);
    // let right_key_just_pressed =
    //     left_stick_x > 0.0 || keyboard_input.just_pressed(KeyCode::ArrowRight);
    let right_key_pressed = left_stick_x > 0.0 || keyboard_input.pressed(KeyCode::ArrowRight);

    let up_key_pressed = left_stick_y > 0.0 || keyboard_input.pressed(KeyCode::ArrowUp);
    // let right_key_just_pressed =
    //     left_stick_x > 0.0 || keyboard_input.just_pressed(KeyCode::ArrowRight);
    let down_key_pressed = left_stick_y < 0.0 || keyboard_input.pressed(KeyCode::ArrowDown);

    let up_key_pressed = keyboard_input.pressed(KeyCode::ArrowUp) || left_stick_y > 0.0;
    let down_key_pressed = keyboard_input.pressed(KeyCode::ArrowDown) || left_stick_y < 0.0;

    let (mut transform, mut texture_atlas, mut sprite_image) = player_query.single_mut();

    if left_key_pressed {
        let x = transform.translation.x - PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        if x > -600. + 20. {
            transform.translation.x -= PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        }
        sprite_image.flip_x = false;
    } else if right_key_pressed {
        let x = transform.translation.x + PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        if x < 600. - 20. {
            transform.translation.x += PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        }
        sprite_image.flip_x = true;
    }
    if up_key_pressed {
        let y = transform.translation.y + PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        // if y < 11200. {
        if y < 1300. {
            transform.translation.y += PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        }
    } else if down_key_pressed {
        let y = transform.translation.y - PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        if y > -725. {
            transform.translation.y -= PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        }
    }

    if jump_key_just_pressed {
        *game_phase = GamePhase::Dialog;
    }
}

fn dialog_system(
    mut commands: Commands,
    mut game_phase: ResMut<GamePhase>,
    open_dialog: Query<Entity, With<DialogBox>>,
    dialog_query: Query<(&Dialog, &Transform)>,
    player_query: Query<(&Transform), (With<Player>, With<ActivePlayer>)>,
) {
    if *game_phase != GamePhase::Dialog {
        for entity in &open_dialog {
            commands.entity(entity).despawn_recursive();
        }
    }

    if *game_phase == GamePhase::Dialog {
        let player_position = player_query.single();
        let dialogs = dialog_query
            .iter()
            .filter(|(_, position)| {
                (position.translation.x - player_position.translation.x).abs()
                    + (position.translation.y - player_position.translation.y).abs()
                    < 160.
            })
            .collect::<Vec<_>>();
        let (dialog, _) = match dialogs.first() {
            Some(d) => d,
            None => {
                *game_phase = GamePhase::Open;
                return;
            }
        };
        if open_dialog.is_empty() {
            commands
                .spawn((
                    Game,
                    DialogBox,
                    NodeBundle {
                        background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                        style: Style {
                            width: Val::Percent(97.0),
                            height: Val::Percent(15.0),
                            position_type: PositionType::Absolute,
                            align_items: AlignItems::Center,
                            bottom: Val::Percent(2.5),
                            left: Val::Percent(1.5),
                            ..default()
                        },

                        ..default()
                    },
                ))
                .with_children(|child| {
                    child.spawn(ImageBundle {
                        image: UiImage {
                            texture: dialog.image.clone(),
                            ..default()
                        },
                        style: Style {
                            position_type: PositionType::Absolute,
                            align_items: AlignItems::Center,
                            top: Val::Percent(10.0),
                            left: Val::Percent(1.0),
                            ..default()
                        },
                        ..default()
                    });
                    child.spawn(
                        TextBundle::from_section(
                            dialog.dialog.sections[0].value.clone(),
                            dialog.dialog.sections[0].style.clone(),
                        )
                        .with_text_justify(JustifyText::Left)
                        .with_style(Style {
                            position_type: PositionType::Absolute,
                            align_items: AlignItems::Center,
                            top: Val::Percent(5.5),
                            left: Val::Px(120.0),
                            ..default()
                        }),
                    );
                });
        }
        // *game_phase = GamePhase::Begining;
    }
}

pub struct PlatformPlugin;
impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), (setup))
            .init_resource::<GamePhase>()
            .add_systems(PreUpdate, camera_tracking::camera_tracking_system)
            .add_systems(
                Update,
                (keyboard_input_system, cloud_movement, dialog_system)
                    .run_if(in_state(AppState::Game)),
            );
    }
}
