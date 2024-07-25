use crate::camera_tracking;
use crate::despawn_screen;
use crate::AppState;
use crate::{
    CHARACTER_DOWNWARD_VELOCITY_PER_FRAME, MAXIMUM_DOWNWARD_VELOCITY, PLAYER_MOVEMENT_SPEED,
};
use bevy::{prelude::*, render::texture::ImageLoader};
use bevy_prng::ChaCha8Rng;
use bevy_rand::resource::GlobalEntropy;
use bevy_rapier2d::prelude::*;
use rand::prelude::Rng;
use std::collections::HashMap;

const PLAYER_SPRITE_SIZE_Y: f32 = 50.;

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

#[derive(Resource, Deref, DerefMut)]
pub struct GustTimer(Timer);

#[derive(Component)]
pub struct Game;

#[derive(Component)]
pub struct Player(usize);

#[derive(Component, Default)]
struct PlayerMovement {
    x_per_second: f32,
    y_per_second: f32,
    airborne: bool,
    falling: bool,
    timer: Timer,
    is_jump_reset: bool,
}

#[derive(Component)]
pub struct ActivePlayer;

#[derive(Component)]
pub struct Cloud {
    velocity: Vec2,
}

#[derive(Component)]
pub struct DialogBox;

#[derive(Component)]
pub struct TextIndicatorParentSelector;

#[derive(Component)]
pub struct TextIndicator;

#[derive(Component)]
pub struct Dialog {
    pub image: Handle<Image>,
    pub dialog: Text,
}

#[derive(Component)]
pub struct WindGust {
    timer: Timer,
    quarter_lifetime_in_seconds: Timer,
    current_quarter: usize,
}

#[derive(Component)]
pub struct Platform {
    height: f32,
}

#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct MovingPlatform {
    pub lifetime_in_seconds: Timer,
    pub velocity: Vec2,
}

// TODO add a timer
// TODO add a point resource

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let text_style = TextStyle {
        font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
        font_size: 18.0,
        ..default()
    };

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
        Player(1),
        PlayerMovement {
            timer: Timer::from_seconds(0.4, TimerMode::Once),
            falling: true,
            ..default()
        },
        ActivePlayer,
        Collider::cuboid(20.0, 25.),
        KinematicCharacterController {
            filter_groups: Some(CollisionGroups::new(
                Group::ALL,
                Group::from_iter([Group::GROUP_11]),
            )),
            ..default()
        },
        Game,
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(210.0, -400.0, 10.0),
                ..default()
            },
            sprite: Sprite {
                flip_x: false,
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("person2.png");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(40, 50),
        1,
        1,
        Some(UVec2::new(0, 0)),
        Some(UVec2::new(0, 0)),
    );

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn((
        Player(2),
        PlayerMovement {
            timer: Timer::from_seconds(0.4, TimerMode::Once),
            falling: true,
            ..default()
        },
        Collider::cuboid(20.0, 25.0),
        KinematicCharacterController {
            filter_groups: Some(CollisionGroups::new(
                Group::ALL,
                Group::from_iter([Group::GROUP_12]),
            )),
            ..default()
        },
        Game,
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(-100.0, -300.0, 10.0),
                ..default()
            },
            sprite: Sprite {
                flip_x: false,
                ..default()
            },
            ..default()
        },
    ));

    commands.spawn((
        Game,
        RigidBody::Fixed,
        Collider::polyline(
            vec![
                Vec2::new(-599.0, -725.0),
                Vec2::new(599., -725.),
                Vec2::new(599., 11200.),
                Vec2::new(-599., 11200.),
                Vec2::new(-599.0, -725.0),
            ],
            None,
        ),
    ));

    let texture_handle = asset_server.load("cloud1.png");

    commands.spawn((
        Game,
        Cloud {
            velocity: Vec2::new(73.5, 0.0),
        },
        RigidBody::KinematicPositionBased,
        Collider::cuboid(20., 10.0),
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, -110.0, 0.0),
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("cloudplatform.png");

    commands.spawn((
        Game,
        RigidBody::Fixed,
        Collider::cuboid(196.0, 28.5),
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(-404., 200.0, -1.0),
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("raincloud.png");

    commands.spawn((
        Game,
        Cloud {
            velocity: Vec2::new(70., 0.),
        },
        RigidBody::KinematicPositionBased,
        Collider::cuboid(56.5, 25.0),
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(-200.0, -200.0, 0.0),
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

    commands.spawn((
        Game,
        Dialog {
            image: texture_handle.clone(),
            dialog: Text {
                sections: vec![TextSection {
                    value: String::from("Hello, Frankie.\nHere is a random number:"),
                    style: text_style.clone(),
                }],
                ..default()
            },
        },
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(-450.0, -690.0, -9.0),
                ..default()
            },
            ..default()
        },
    ));

    let half_x = 20.;
    let half_y = 20.;
    commands.spawn((
        Game,
        Collider::cuboid(half_x, half_y),
        RigidBody::Fixed,
        TransformBundle::from_transform(Transform {
            translation: Vec3::new(-500., -680.0, 0.),
            ..default()
        }),
        CollisionGroups::new(Group::from_iter([Group::GROUP_10]), Group::ALL),
    ));

    commands.spawn((
        Game,
        Collider::cuboid(half_x, half_y),
        RigidBody::Fixed,
        TransformBundle::from_transform(Transform {
            translation: Vec3::new(-400., -680.0, 0.),
            ..default()
        }),
        CollisionGroups::new(Group::from_iter([Group::GROUP_11]), Group::ALL),
    ));

    commands.spawn((
        Game,
        Collider::cuboid(half_x, half_y),
        RigidBody::Fixed,
        TransformBundle::from_transform(Transform {
            translation: Vec3::new(-300., -680.0, 0.),
            ..default()
        }),
        CollisionGroups::new(
            Group::from_iter([Group::GROUP_12, Group::GROUP_13]),
            Group::ALL,
        ),
    ));
}

pub fn platform_sensor_system(
    mut commands: Commands,
    player: Query<(Entity, &Player, &Transform, &KinematicCharacterController), With<Player>>,
    rapier_context: Res<RapierContext>,
    mut platform_query: Query<
        (Entity, &GlobalTransform, &Platform, &mut CollisionGroups),
        (Without<Player>, With<Platform>),
    >,
) {
    for (player_entity, _player, player_transform, kinematic_controller) in player.iter() {
        let player_bottom = player_transform.translation.y - (PLAYER_SPRITE_SIZE_Y);

        let filter_groups = kinematic_controller.filter_groups.unwrap().filters;

        for (platform_entity, platform_transform, platform, mut platform_collision_groups) in
            platform_query.iter_mut()
        {
            let platform_height = platform_transform.translation().y - platform.height;

            if player_bottom >= platform_height {
                platform_collision_groups.memberships.insert(filter_groups);
            } else {
                platform_collision_groups.memberships.remove(filter_groups);
            }
        }
    }
}

fn cloud_movement(time: Res<Time>, mut clouds: Query<(&mut Transform, &Cloud), With<Cloud>>) {
    for (mut transform, wind) in clouds.iter_mut() {
        if transform.translation.x < -680.0 {
            transform.translation.x = 680.
        }
        transform.translation.x -= wind.velocity.x * time.delta_seconds();
    }
}

fn gust_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut gust_timer: ResMut<GustTimer>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    mut platform_query: Query<(
        Entity,
        &mut Transform,
        &mut MovingPlatform,
        &mut TextureAtlas,
        &mut WindGust,
    )>,
) {
    gust_timer.tick(time.delta());
    if gust_timer.just_finished() {
        if rng.gen_range(0..=1) == 0 {
            let texture_handle = asset_server.load("wind1-Sheet.png");
            let layout = TextureAtlasLayout::from_grid(
                UVec2::new(80, 107),
                9,
                4,
                Some(UVec2::new(0, 0)),
                Some(UVec2::new(0, 0)),
            );

            let texture_atlas_layout = texture_atlas_layouts.add(layout);
            let lifetime_in_seconds = rng.gen_range(15.0..20.0);
            commands
                .spawn((
                    Game,
                    MovingPlatform {
                        lifetime_in_seconds: Timer::from_seconds(
                            lifetime_in_seconds,
                            TimerMode::Once,
                        ),
                        velocity: Vec2::new(0.0, rng.gen_range(20.0..60.0)),
                    },
                    WindGust {
                        timer: Timer::from_seconds(0.15, TimerMode::Repeating),
                        quarter_lifetime_in_seconds: Timer::from_seconds(
                            lifetime_in_seconds / 4.0,
                            TimerMode::Repeating,
                        ),
                        current_quarter: 1,
                    },
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: rng.gen_range(0..9),
                    },
                    SpriteBundle {
                        texture: texture_handle,
                        transform: Transform {
                            translation: Vec3::new(rng.gen_range(-600.0..600.0), -700.0, 10.0),
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|child| {
                    child.spawn((
                        Game,
                        RigidBody::KinematicPositionBased,
                        Collider::cuboid(50.0, 2.0),
                        Platform { height: 51.5 }, // sprite_center as(107/2) - half_y as(2.0)
                        CollisionGroups::new(Group::GROUP_10, Group::ALL),
                        TransformBundle::from_transform(Transform {
                            translation: Vec3::new(0., 40.0, -9.0),
                            ..default()
                        }),
                    ));
                });
        }
    }

    for (entity, mut transform, mut moving_platform, mut atlas, mut gust) in
        platform_query.iter_mut()
    {
        moving_platform.lifetime_in_seconds.tick(time.delta());
        gust.timer.tick(time.delta());
        gust.quarter_lifetime_in_seconds.tick(time.delta());

        if moving_platform.lifetime_in_seconds.just_finished() {
            commands.entity(entity).despawn_recursive();
        } else {
            transform.translation.x += moving_platform.velocity.x * time.delta_seconds();
            transform.translation.y += moving_platform.velocity.y * time.delta_seconds();

            if gust.quarter_lifetime_in_seconds.just_finished() {
                if gust.current_quarter < 4 {
                    gust.current_quarter += 1;
                }
            }
            if gust.timer.just_finished() {
                if atlas.index >= 8 * gust.current_quarter + gust.current_quarter - 1 {
                    atlas.index = 9 * gust.current_quarter - 9;
                } else {
                    atlas.index += 1;
                }
            }
        }
    }
}

fn player_gravity_system(time: Res<Time>, mut player_query: Query<&mut PlayerMovement>) {
    for mut player_movement in player_query.iter_mut() {
        if player_movement.falling {
            if player_movement.airborne && !player_movement.timer.finished() {
                player_movement.timer.tick(2 * time.delta());
                let current_jump_time = player_movement.timer.elapsed().as_secs_f32();
                let total_jump_time = player_movement.timer.duration().as_secs_f32();
                let jump_percent = current_jump_time / total_jump_time;
                player_movement.y_per_second = 600. * (1. - jump_percent);
            } else {
                let d = (MAXIMUM_DOWNWARD_VELOCITY
                    - (player_movement.y_per_second + MAXIMUM_DOWNWARD_VELOCITY))
                    / MAXIMUM_DOWNWARD_VELOCITY;
                if player_movement.y_per_second > 0. {
                    player_movement.y_per_second = 0.0;
                } else if player_movement.y_per_second > -MAXIMUM_DOWNWARD_VELOCITY {
                    player_movement.y_per_second +=
                        CHARACTER_DOWNWARD_VELOCITY_PER_FRAME * time.delta_seconds() * d.cos();
                }
            }
        }
    }
}

fn translate_player_system(
    time: Res<Time>,
    mut player_query: Query<(&mut KinematicCharacterController, &mut PlayerMovement)>,
) {
    for (mut transform, mut trajectory_velocity) in player_query.iter_mut() {
        transform.translation = Some(Vec2::new(
            trajectory_velocity.x_per_second * time.delta_seconds(),
            trajectory_velocity.y_per_second * time.delta_seconds(),
        ))
    }
}

fn keyboard_input_system(
    mut commands: Commands,
    mut game_phase: ResMut<GamePhase>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Res<Gamepads>,
    button_inputs: Res<ButtonInput<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut active_player_query: Query<
        (
            Entity,
            &mut KinematicCharacterController,
            &mut TextureAtlas,
            &mut Sprite,
            &mut PlayerMovement,
        ),
        With<ActivePlayer>,
    >,
    active_player_kinematic_output_query: Query<(Entity, &KinematicCharacterControllerOutput)>,
    mut player_query: Query<Entity, With<Player>>,
) {
    let gamepad = match gamepads.iter().next() {
        Some(gp) => gp,
        None => Gamepad::new(0),
    };

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

    let jump_key_pressed = button_inputs
        .pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
        || keyboard_input.pressed(KeyCode::Space);
    let jump_key_just_pressed = button_inputs
        .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
        || keyboard_input.just_pressed(KeyCode::Space);
    // let run_key_just_pressed = button_inputs
    //     .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::West))
    //     || keyboard_input.just_pressed(KeyCode::ShiftLeft);
    let switch_key_just_pressed = button_inputs
        .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::West))
        || keyboard_input.just_pressed(KeyCode::ShiftLeft);
    // let left_key_just_pressed =
    //     left_stick_x < 0.0 || keyboard_input.just_pressed(KeyCode::ArrowLeft);
    let left_key_pressed = left_stick_x < -0.10 || keyboard_input.pressed(KeyCode::ArrowLeft);
    // let right_key_just_pressed =
    //     left_stick_x > 0.0 || keyboard_input.just_pressed(KeyCode::ArrowRight);
    let right_key_pressed = left_stick_x > 0.10 || keyboard_input.pressed(KeyCode::ArrowRight);

    let up_key_pressed = left_stick_y > 0.10 || keyboard_input.pressed(KeyCode::ArrowUp);
    // let right_key_just_pressed =
    //     left_stick_x > 0.0 || keyboard_input.just_pressed(KeyCode::ArrowRight);
    let down_key_pressed = left_stick_y < -0.10 || keyboard_input.pressed(KeyCode::ArrowDown);

    let up_key_pressed = keyboard_input.pressed(KeyCode::ArrowUp) || left_stick_y > 0.10;
    let down_key_pressed = keyboard_input.pressed(KeyCode::ArrowDown) || left_stick_y < -0.10;

    let (
        mut active_player_entity,
        mut transform,
        mut texture_atlas,
        mut sprite_image,
        mut player_movement,
    ) = active_player_query.single_mut();

    let mut total_x = 0.;
    let mut total_y = 0.;

    let mut grounded = match active_player_kinematic_output_query
        .iter()
        .find(|e| e.0 == active_player_entity)
    {
        Some((_, k)) => k.grounded,
        None => false,
    };

    if jump_key_just_pressed {
        // Checks for dialog
        *game_phase = GamePhase::Dialog;
        if grounded && player_movement.is_jump_reset {
            // player can now jump
            player_movement.timer.reset();
            player_movement.airborne = true;
            player_movement.falling = false;
            player_movement.is_jump_reset = false;
            total_y = 600.0;
        }
    } else if jump_key_pressed {
        if player_movement.airborne && player_movement.timer.finished() {
            player_movement.falling = true;
        }

        if player_movement.airborne && !player_movement.timer.finished() {
            player_movement.timer.tick(time.delta());

            let current_jump_time = player_movement.timer.elapsed().as_secs_f32();
            let total_jump_time = player_movement.timer.duration().as_secs_f32();
            let jump_percent = current_jump_time / total_jump_time;
            total_y = 600. * (1. - jump_percent);
        }
    }

    if !jump_key_pressed {
        if player_movement.airborne && !player_movement.timer.finished() {
            player_movement.timer.tick(2 * time.delta());
            let current_jump_time = player_movement.timer.elapsed().as_secs_f32();
            let total_jump_time = player_movement.timer.duration().as_secs_f32();
            let jump_percent = current_jump_time / total_jump_time;
            total_y = 600. * (1. - jump_percent);
        } else {
            player_movement.falling = true;
        }

        if grounded {
            player_movement.is_jump_reset = true;
        }
    }

    if left_key_pressed {
        // let x = transform.translation.x - PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        // if x > -600. + 20. {
        //     transform.translation.x -= PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        // }
        total_x = -PLAYER_MOVEMENT_SPEED; //* time.delta_seconds();
        sprite_image.flip_x = false;
    } else if right_key_pressed {
        // let x = transform.translation.x + PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        // if x < 600. - 20. {
        //     transform.translation.x += PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        // }
        total_x = PLAYER_MOVEMENT_SPEED; //* time.delta_seconds();
        sprite_image.flip_x = true;
    }

    // if up_key_pressed {
    //     // let y = transform.translation.y + PLAYER_MOVEMENT_SPEED * time.delta_seconds();
    //     // // if y < 11200. {
    //     // if y < 1300. {
    //     //     transform.translation.y += PLAYER_MOVEMENT_SPEED * time.delta_seconds();
    //     // }
    //     total_y = PLAYER_MOVEMENT_SPEED; // * time.delta_seconds();
    // } else if down_key_pressed {
    //     // let y = transform.translation.y - PLAYER_MOVEMENT_SPEED * time.delta_seconds();
    //     // if y > -725. {
    //     //     transform.translation.y -= PLAYER_MOVEMENT_SPEED * time.delta_seconds();
    //     // }
    //     total_y = -PLAYER_MOVEMENT_SPEED; // * time.delta_seconds();
    // }

    if switch_key_just_pressed {
        player_query.iter_mut().for_each(|e| {
            if e == active_player_entity {
                commands.entity(e).remove::<ActivePlayer>();
            } else {
                commands.entity(e).insert(ActivePlayer);
            }
        });
        total_x = 0.;
        player_movement.falling = true;
    }

    if !player_movement.falling {
        player_movement.y_per_second = total_y;
    }
    player_movement.x_per_second = total_x;
}

fn dialog_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_phase: ResMut<GamePhase>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    open_dialog: Query<Entity, With<DialogBox>>,
    mut dialog_query: Query<(Entity, &Dialog, &Transform)>,
    mut selected_text_query: Query<Entity, With<TextIndicatorParentSelector>>,
    mut text_indicator_query: Query<Entity, With<TextIndicator>>,
    mut active_player_query: Query<
        ((&Transform, &mut PlayerMovement)),
        (With<Player>, With<ActivePlayer>),
    >,
) {
    if *game_phase != GamePhase::Dialog {
        for entity in &open_dialog {
            commands.entity(entity).despawn_recursive();
        }
    }

    let (player_position, mut player_movement) = active_player_query.single_mut();
    let mut dialogs = dialog_query
        .iter()
        .filter(|(_, _, t)| {
            (t.translation.x - player_position.translation.x).abs()
                + (t.translation.y - player_position.translation.y).abs()
                < 100.
        })
        .map(|(e, d, t)| {
            (
                e,
                d,
                t,
                (t.translation.x - player_position.translation.x).abs()
                    + (t.translation.y - player_position.translation.y).abs(),
            )
        })
        .collect::<Vec<_>>();

    dialogs.sort_by(|(_, _, _, a), (_, _, _, b)| a.total_cmp(b));

    let (mut entity, dialog, _, _) = match dialogs.first() {
        Some(d) => {
            player_movement.is_jump_reset = false;
            d
        }
        None => {
            for entity in text_indicator_query.iter_mut() {
                commands.entity(entity).despawn();
            }
            *game_phase = GamePhase::Open;
            return;
        }
    };

    commands.entity(entity).insert(TextIndicatorParentSelector);
    for e in selected_text_query.iter() {
        if e != entity {
            commands.entity(e).remove::<TextIndicatorParentSelector>();
            commands.entity(e).despawn_descendants();
        } else {
            if text_indicator_query.is_empty() {
                commands.entity(entity).with_children(|child| {
                    let texture_handle = asset_server.load("textindicator.png");
                    child.spawn((
                        Game,
                        TextIndicator,
                        SpriteBundle {
                            texture: texture_handle.clone(),
                            transform: Transform {
                                translation: Vec3::new(0.0, 50.0, 0.0),
                                ..default()
                            },
                            ..default()
                        },
                    ));
                });
            }
        }
    }

    if *game_phase == GamePhase::Dialog {
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
                            format!(
                                "{}{}",
                                dialog.dialog.sections[0].value.clone(),
                                rng.gen_range(0..=100)
                            ),
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
            .insert_resource(GustTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
            .add_systems(PreUpdate, camera_tracking::camera_tracking_system)
            .add_systems(
                Update,
                (
                    keyboard_input_system,
                    player_gravity_system,
                    translate_player_system,
                    cloud_movement,
                    platform_sensor_system,
                    gust_system,
                    dialog_system,
                )
                    .run_if(in_state(AppState::Game)),
            );
    }
}
