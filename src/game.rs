use crate::camera_tracking;
use crate::despawn_screen;
use crate::menu::TotalSeconds;
use crate::AppState;
use crate::{
    CHARACTER_DOWNWARD_VELOCITY_PER_FRAME, MAXIMUM_DOWNWARD_VELOCITY, PLAYER_MOVEMENT_SPEED,
};
use bevy::audio::PlaybackMode;
use bevy::text::TextLayoutInfo;
use bevy::time::Stopwatch;
use bevy::{prelude::*, render::texture::ImageLoader};
use bevy_prng::ChaCha8Rng;
use bevy_rand::resource::GlobalEntropy;
use bevy_rapier2d::na::distance;
use bevy_rapier2d::prelude::*;
use rand::prelude::Rng;
use std::collections::HashMap;
use std::ops::Range;
use std::time::Duration;
use std::time::Instant;

const PLAYER_SPRITE_SIZE_Y: f32 = 50.;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Resource)]
enum GamePhase {
    #[default]
    Play,
    Reset,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Resource)]
enum DevPhase {
    #[default]
    Play,
    Debug,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Resource)]
pub struct WaterCollection {
    total_player1: u32,
    total_player2: u32,
}

// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Resource)]
// pub struct IceCollection(pub u32);

// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Resource)]
// pub struct Score(pub u32);

// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Resource)]
// pub struct Altitude(pub u32);

#[derive(Resource, Deref, DerefMut)]
pub struct GustTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
pub struct TotalScore(pub u32);

#[derive(Resource, Deref, DerefMut)]
pub struct TotalTime(Timer);

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

#[derive(Component, Default)]
pub struct CloudSpawner {
    image: Handle<Image>,
    group: usize,
    min_velocity: Vec2,
    max_velocity: Vec2,
    min_height: f32,
    max_height: f32,
    min_time: Timer,
    max_time: Timer,
    retry_time: Timer,
    probability: Range<i32>,
    collider: Collider,
}

#[derive(Component, Default)]
pub struct WaterCollectableSpawner {
    image: Handle<Image>,
    min_height: f32,
    max_height: f32,
    min_time: Timer,
    max_time: Timer,
    retry_time: Timer,
    probability: Range<i32>,
}

#[derive(Component)]
pub struct Cloud {
    group: usize,
    velocity: Vec2,
}

#[derive(Component)]
pub struct DialogBox;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Resource)]
pub struct DialogSpeaker(Option<Entity>);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Deref, DerefMut, Resource)]
pub struct DialogSpeakerOpenDialog(bool);

#[derive(Clone, Debug, Default, Resource)]
pub struct DialogDecisionSelection(usize);

#[derive(Component)]
pub struct Selectable;

#[derive(Component)]
pub struct TextIndicatorParentSelector;

#[derive(Component)]
pub struct TextIndicator;

#[derive(Component)]
pub struct Dialog {
    pub image: Handle<Image>,
    pub dialog: Text,
    pub title: String,
    pub subtitle: String,
}

#[derive(Component)]
pub struct WindGust {
    timer: Timer,
    quarter_lifetime_in_seconds: Timer,
    current_quarter: usize,
}

#[derive(Component)]
pub struct WaterCollectable(Timer);

#[derive(Component)]
pub struct Platform {
    // generally half of player sprite
    height_adjustment: f32,
}

#[derive(Component)]
pub struct WaterCollectionScoreboard;

#[derive(Component)]
pub struct TotalScoreboard;

#[derive(Component)]
pub struct TimeDisplay;

#[derive(Component)]
pub struct BigWaterDrop;

#[derive(Component)]
pub struct MovingPlatform {
    pub lifetime_in_seconds: Timer,
    pub velocity: Vec2,
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    mut total_time: ResMut<TotalTime>,
    mut total_score: ResMut<TotalScore>,
    mut water_collection: ResMut<WaterCollection>,
    total_seconds: Res<TotalSeconds>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut bg: ResMut<ClearColor>,
) {
    bg.0 = Color::BLACK;
    let text_style = TextStyle {
        font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
        font_size: 18.0,
        ..default()
    };

    total_time
        .0
        .set_duration(Duration::from_secs_f32(total_seconds.0));
    total_time.0.reset();
    total_score.0 = 0;
    *water_collection = WaterCollection::default();

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

    let texture_handle = asset_server.load("person.png");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(40, 50),
        1,
        1,
        Some(UVec2::new(0, 0)),
        Some(UVec2::new(0, 0)),
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands
        .spawn((
            Game,
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
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
            SpriteBundle {
                texture: texture_handle.clone(),
                transform: Transform {
                    translation: Vec3::new(210.0, 00.0, 11.0),
                    ..default()
                },
                sprite: Sprite {
                    flip_x: false,
                    ..default()
                },
                ..default()
            },
        ))
        .insert(CollisionGroups::new(
            Group::from(Group::GROUP_1),
            Group::from(Group::GROUP_1),
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
    commands
        .spawn((
            Game,
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
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
            SpriteBundle {
                texture: texture_handle.clone(),
                transform: Transform {
                    translation: Vec3::new(-100.0, 0.0, 11.0),
                    ..default()
                },
                sprite: Sprite {
                    flip_x: false,
                    ..default()
                },
                ..default()
            },
        ))
        .insert(CollisionGroups::new(
            Group::from(Group::GROUP_1),
            Group::from(Group::GROUP_1),
        ));

    commands.spawn((
        Game,
        CloudSpawner {
            image: asset_server.load("raincloud.png"),
            group: 0,
            min_velocity: Vec2::new(5.0, 0.),
            max_velocity: Vec2::new(55.0, 0.),
            min_height: -250.,
            max_height: -160.,
            min_time: Timer::from_seconds(2.5, TimerMode::Once),
            max_time: Timer::from_seconds(10., TimerMode::Once),
            retry_time: Timer::from_seconds(0.1, TimerMode::Repeating),
            probability: Range { start: 1, end: 200 },
            collider: Collider::cuboid(56.5, 25.0),
            ..default()
        },
    ));

    commands.spawn((
        Game,
        CloudSpawner {
            image: asset_server.load("cloud1.png"),
            group: 1,
            min_height: -150.,
            max_height: 0.,
            min_velocity: Vec2::new(10.0, 0.),
            max_velocity: Vec2::new(75.0, 0.),
            min_time: Timer::from_seconds(0.5, TimerMode::Once),
            max_time: Timer::from_seconds(5., TimerMode::Once),
            retry_time: Timer::from_seconds(0.1, TimerMode::Repeating),
            probability: Range { start: 1, end: 60 },
            collider: Collider::cuboid(20., 10.0),
            ..default()
        },
    ));
    commands.spawn((
        Game,
        WaterCollectableSpawner {
            image: asset_server.load("droplet.png"),
            // atlas: TextureAtlas {
            //     layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            //         UVec2::new(20, 20),
            //         1,
            //         1,
            //         Some(UVec2::new(0, 0)),
            //         Some(UVec2::new(0, 0)),
            //     )),
            //     index: rng.gen_range(0..1),
            // },
            // grid: GridOptions { rows: 1, cols: 1 },
            min_height: -350.,
            max_height: 200.,
            min_time: Timer::from_seconds(5.0, TimerMode::Once),
            max_time: Timer::from_seconds(60., TimerMode::Once),
            retry_time: Timer::from_seconds(0.1, TimerMode::Repeating),
            probability: Range { start: 1, end: 20 },
            // collider: Collider::cuboid(10., 10.0),
        },
    ));

    commands.spawn((
        Game,
        CloudSpawner {
            image: asset_server.load("raincloud.png"),
            group: 1,
            min_velocity: Vec2::new(55.0, 0.),
            max_velocity: Vec2::new(59.0, 0.),
            min_height: 300.,
            max_height: 400.,
            min_time: Timer::from_seconds(2.5, TimerMode::Once),
            max_time: Timer::from_seconds(6., TimerMode::Once),
            retry_time: Timer::from_seconds(0.1, TimerMode::Repeating),
            probability: Range { start: 1, end: 60 },
            collider: Collider::cuboid(56.5, 25.0),
            ..default()
        },
    ));

    commands.spawn((
        Game,
        CloudSpawner {
            image: asset_server.load("cloud1.png"),
            group: 1,
            min_height: 600.,
            max_height: 700.,
            min_velocity: Vec2::new(75.0, 0.),
            max_velocity: Vec2::new(79.0, 0.),
            min_time: Timer::from_seconds(0.5, TimerMode::Once),
            max_time: Timer::from_seconds(2.5, TimerMode::Once),
            retry_time: Timer::from_seconds(0.1, TimerMode::Repeating),
            probability: Range { start: 1, end: 20 },
            collider: Collider::cuboid(20., 10.0),
            ..default()
        },
    ));
    commands.spawn((
        Game,
        CloudSpawner {
            image: asset_server.load("cloud1.png"),
            group: 1,
            min_height: 700.,
            max_height: 800.,
            min_velocity: Vec2::new(75.0, 0.),
            max_velocity: Vec2::new(79.0, 0.),
            min_time: Timer::from_seconds(0.5, TimerMode::Once),
            max_time: Timer::from_seconds(3., TimerMode::Once),
            retry_time: Timer::from_seconds(0.1, TimerMode::Repeating),
            probability: Range { start: 1, end: 35 },
            collider: Collider::cuboid(20., 10.0),
            ..default()
        },
    ));
    commands.spawn((
        Game,
        CloudSpawner {
            image: asset_server.load("cloud1.png"),
            group: 1,
            min_height: 800.,
            max_height: 1000.,
            min_velocity: Vec2::new(75.0, 0.),
            max_velocity: Vec2::new(79.0, 0.),
            min_time: Timer::from_seconds(0.5, TimerMode::Once),
            max_time: Timer::from_seconds(4., TimerMode::Once),
            retry_time: Timer::from_seconds(0.1, TimerMode::Repeating),
            probability: Range { start: 1, end: 50 },
            collider: Collider::cuboid(20., 10.0),
            ..default()
        },
    ));

    let texture_handle = asset_server.load("cloudplatform.png");
    commands.spawn((
        Game,
        RigidBody::Fixed,
        Collider::cuboid(196.0, 28.5),
        Platform {
            height_adjustment: 28.0,
        }, // sprite_center as(107/2) - half_y as(2.0)
        CollisionGroups::new(Group::GROUP_10, Group::ALL),
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(-404., 200.0, -1.0),
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
        Platform {
            height_adjustment: 28.0,
        }, // sprite_center as(107/2) - half_y as(2.0)
        CollisionGroups::new(Group::GROUP_10, Group::ALL),
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(404., 500.0, -1.0),
                ..default()
            },
            sprite: Sprite {
                flip_y: true,
                flip_x: true,
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("mediumcloudplatform.png");
    commands.spawn((
        Game,
        RigidBody::Fixed,
        Collider::cuboid(250., 18.5),
        Platform {
            height_adjustment: 28.0,
        }, // sprite_center as(107/2) - half_y as(2.0)
        CollisionGroups::new(Group::GROUP_10, Group::ALL),
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(375., 1050.0, -1.0),
                ..default()
            },
            sprite: Sprite {
                flip_y: true,
                flip_x: true,
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("mediumcloudplatform.png");
    commands.spawn((
        Game,
        RigidBody::Fixed,
        Collider::cuboid(250., 18.5),
        Platform {
            height_adjustment: 28.0,
        }, // sprite_center as(107/2) - half_y as(2.0)
        CollisionGroups::new(Group::GROUP_10, Group::ALL),
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(-375., 1050.0, -1.0),
                ..default()
            },
            sprite: Sprite {
                flip_y: false,
                flip_x: false,
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("topcloud.png");
    commands.spawn((
        Game,
        RigidBody::Fixed,
        Collider::cuboid(215.5, 30.),
        Platform {
            height_adjustment: 28.0,
        }, // sprite_center as(107/2) - half_y as(2.0)
        CollisionGroups::new(Group::GROUP_10, Group::ALL),
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0., 1150.0, -1.0),
                ..default()
            },
            sprite: Sprite {
                flip_y: false,
                flip_x: false,
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

    commands.spawn((
        Game,
        TimeDisplay,
        TextBundle::from_sections([
            TextSection {
                value: String::from("TIME "),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 16.0,
                    ..default()
                },
            },
            TextSection {
                value: String::from("0"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 16.0,
                    ..default()
                },
            },
        ])
        .with_text_justify(JustifyText::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(2.0),
            left: Val::Percent(3.0),
            ..default()
        }),
    ));

    commands.spawn((
        Game,
        WaterCollectionScoreboard,
        TextBundle::from_sections([
            TextSection {
                value: String::from("Nata's Water: "),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 16.0,
                    ..default()
                },
            },
            TextSection {
                value: String::from("0"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 16.0,
                    ..default()
                },
            },
            TextSection {
                value: String::from("\n\nNena's Water: "),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 16.0,
                    ..default()
                },
            },
            TextSection {
                value: String::from("0"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 16.0,
                    ..default()
                },
            },
            TextSection {
                value: String::from("\n\nTotal: "),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 16.0,
                    ..default()
                },
            },
            TextSection {
                value: String::from("0"),
                style: TextStyle {
                    font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                    font_size: 16.0,
                    ..default()
                },
            },
        ])
        .with_text_justify(JustifyText::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(7.0),
            left: Val::Percent(3.0),
            ..default()
        }),
    ));

    let texture_handle = asset_server.load("tlaloc.png");
    commands.spawn((
        Game,
        Dialog {
            image: texture_handle.clone(),
            dialog: Text {
                sections: vec![
                    TextSection {
                        value: String::from("Offer me water droplets to collect points."),
                        style: text_style.clone(),
                    },
                    TextSection {
                        value: String::from("Your offering pleases me. Will you surrender your tribute now to receive points?\n\n▶ No   Yes"),
                        style: text_style.clone(),
                    },
                    TextSection {
                        value: String::from("Your offering pleases me. Will you surrender your tribute now to receive points?\n\n  No ▶ Yes"),
                        style: text_style.clone(),
                    }
                ],
                ..default()
            },
            title: String::from("Tlaloc"),
            subtitle: String::from("The god of rain")
        },
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0., 1250.0, -2.0),
                ..default()
            },
            sprite: Sprite {
                flip_y: false,
                flip_x: false,
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("chalchiuhtlicue.png");
    commands.spawn((
        Game,
        Dialog {
            image: asset_server.load("chalchiuhtlicue-bust2.png"),
            dialog: Text {
                sections: vec![
                    TextSection {
                        value: String::from("Collect the water droplets and offer them to Tlaloc (the rain god) at the top of the sky!\n\nRemember, the more water you collect, the heavier you get."),
                        style: text_style.clone(),
                    },
                    TextSection {
                        value: String::from("You've collected water. Do you want to give it to me to become as light as a wisp?\n\n▶ No   Yes"),
                        style: text_style.clone(),
                    },
                    TextSection {
                        value: String::from("You've collected water. Do you want to give it to me to become as light as a wisp?\n\n  No ▶ Yes"),
                        style: text_style.clone(),
                    }
                ],
                ..default()
            },
            title: String::from("Chalchiuhtlicue"),
            subtitle: String::from("The river goddess")
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
}

fn time_count_system(
    time: Res<Time>,
    mut app_state: ResMut<NextState<AppState>>,
    dialog_speaker_open_dialog: Res<DialogSpeakerOpenDialog>,
    mut total_time: ResMut<TotalTime>,
    mut time_display_query: Query<&mut Text, With<TimeDisplay>>,
) {
    if dialog_speaker_open_dialog.0 {
        return;
    }
    total_time.0.tick(time.delta());
    match time_display_query.get_single_mut() {
        Ok(mut d) => d.sections[1].value = total_time.0.remaining_secs().ceil().to_string(),
        Err(_) => {}
    }
    if total_time.just_finished() {
        app_state.set(AppState::GameOver);
    }
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
            let platform_height = platform_transform.translation().y - platform.height_adjustment;

            if player_bottom >= platform_height {
                platform_collision_groups.memberships.insert(filter_groups);
            } else {
                platform_collision_groups.memberships.remove(filter_groups);
            }
        }
    }
}

fn collectable_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    rapier_context: Res<RapierContext>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    mut water_collection: ResMut<WaterCollection>,
    total_score: Res<TotalScore>,
    mut water_collectable_spawner_query: Query<&mut WaterCollectableSpawner>,
    mut water_collectable_query: Query<(Entity, &mut WaterCollectable), With<WaterCollectable>>,
    player_query: Query<(Entity, &Player)>,
    mut scoreboard_query: Query<&mut Text, With<WaterCollectionScoreboard>>,
) {
    for mut spawner in water_collectable_spawner_query.iter_mut() {
        spawner.min_time.tick(time.delta());
        spawner.max_time.tick(time.delta());
        spawner.retry_time.tick(time.delta());

        if spawner.min_time.just_finished() {
            continue;
        }

        if spawner.min_time.finished() {
            let make_spawn = if spawner.max_time.just_finished() {
                true
            } else {
                if spawner.retry_time.just_finished() {
                    rng.gen_range(spawner.probability.clone()) == 1
                } else {
                    false
                }
            };

            if make_spawn {
                spawner.min_time.reset();
                spawner.max_time.reset();

                commands
                    .spawn((
                        Game,
                        WaterCollectable(Timer::from_seconds(30.0, TimerMode::Once)),
                        Sensor,
                        Collider::cuboid(10., 10.0),
                        // CollisionGroups::new(Group::GROUP_12 | Group::GROUP_13, Group::ALL),
                        TextureAtlas {
                            layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                                UVec2::new(20, 20),
                                1,
                                1,
                                Some(UVec2::new(0, 0)),
                                Some(UVec2::new(0, 0)),
                            )),
                            index: rng.gen_range(0..1),
                        },
                        SpriteBundle {
                            texture: spawner.image.clone(),
                            transform: Transform {
                                translation: Vec3::new(
                                    rng.gen_range(-580.0..=580.0),
                                    rng.gen_range(spawner.min_height..spawner.max_height),
                                    0.0,
                                ),
                                ..default()
                            },
                            ..default()
                        },
                    ))
                    .insert(ActiveEvents::COLLISION_EVENTS);
            }
        }
    }
    for (entity, mut water_collectable) in water_collectable_query.iter_mut() {
        water_collectable.0.tick(time.delta());
        if water_collectable.0.just_finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        for (player_entity, player) in player_query.iter() {
            if rapier_context
                .intersection_pair(entity, player_entity)
                .is_some()
            {
                if player.0 == 1 {
                    water_collection.total_player1 += 1;
                } else {
                    water_collection.total_player2 += 1;
                }
                commands.spawn(AudioBundle {
                    source: asset_server.load("collect.mp3"),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        ..default()
                    },
                });
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    let mut score = scoreboard_query.single_mut();

    score.sections[1].value = water_collection.total_player2.to_string();
    score.sections[3].value = water_collection.total_player1.to_string();
    score.sections[5].value = total_score.0.to_string();
}

fn cloud_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    mut cloud_spawner_query: Query<&mut CloudSpawner>,
    mut clouds: Query<(Entity, &mut Transform, &Cloud), With<Cloud>>,
) {
    for mut spawner in cloud_spawner_query.iter_mut() {
        spawner.min_time.tick(time.delta());
        spawner.max_time.tick(time.delta());
        spawner.retry_time.tick(time.delta());

        if spawner.min_time.just_finished() {
            continue;
        }

        if spawner.min_time.finished() {
            let make_spawn = if spawner.max_time.just_finished() {
                true
            } else {
                if spawner.retry_time.just_finished() {
                    rng.gen_range(spawner.probability.clone()) == 1
                } else {
                    false
                }
            };

            if make_spawn {
                spawner.min_time.reset();
                spawner.max_time.reset();

                commands.spawn((
                    Game,
                    Cloud {
                        group: 0,
                        velocity: Vec2::new(
                            rng.gen_range(spawner.min_velocity.x..=spawner.max_velocity.x),
                            rng.gen_range(spawner.min_velocity.y..=spawner.max_velocity.y),
                        ),
                    },
                    RigidBody::KinematicPositionBased,
                    spawner.collider.clone(),
                    Platform {
                        height_adjustment: 25.0,
                    }, // sprite_center as(107/2) - half_y as(2.0)
                    CollisionGroups::new(Group::GROUP_10, Group::ALL),
                    SpriteBundle {
                        texture: spawner.image.clone(),
                        transform: Transform {
                            translation: Vec3::new(
                                680.0,
                                rng.gen_range(spawner.min_height..spawner.max_height),
                                0.0,
                            ),
                            ..default()
                        },
                        ..default()
                    },
                ));
            }
        }
    }

    for (entity, mut transform, wind) in clouds.iter_mut() {
        if transform.translation.x < -680.0 {
            commands.entity(entity).despawn_recursive();
            continue;
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
                        Platform {
                            height_adjustment: 51.5,
                        }, // sprite_center as(107/2) - half_y as(2.0)
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

fn player_gravity_system(
    time: Res<Time>,
    mut water_collection: Res<WaterCollection>,
    mut player_query: Query<(&mut PlayerMovement, &Player)>,
) {
    for (mut player_movement, player) in player_query.iter_mut() {
        let water_collected = if player.0 == 1 {
            water_collection.total_player1
        } else {
            water_collection.total_player2
        };
        let mut gravity_multiplier = if water_collected > 15 {
            0.4 - 0.015 * (water_collected - 14) as f32
        } else if water_collected > 8 {
            0.8 - 0.05 * (water_collected - 7) as f32
        } else {
            1.4 - 0.1 * water_collected as f32
        };
        if gravity_multiplier < 0.3 {
            gravity_multiplier = 0.2
        }
        player_movement
            .timer
            .set_duration(Duration::from_secs_f32(gravity_multiplier));
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

fn debug_system(
    mut commands: Commands,
    time: Res<Time>,
    mut dev: ResMut<DevPhase>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut active_player_query: Query<
        Entity,
        (With<ActivePlayer>, With<KinematicCharacterController>),
    >,
    mut active_player_debug: Query<
        (
            Entity,
            &mut Transform,
            &mut TextureAtlas,
            &mut Sprite,
            &mut PlayerMovement,
        ),
        (With<ActivePlayer>, Without<KinematicCharacterController>),
    >,
    mut player_query: Query<(Entity, &Player), With<Player>>,
) {
    // *ptime = time::PhysicsTime::default();
    let d_key_just_pressed = keyboard_input.just_pressed(KeyCode::KeyD);

    if *dev == DevPhase::Play {
        if d_key_just_pressed {
            *dev = DevPhase::Debug;
        } else {
            return;
        }
    } else {
        if d_key_just_pressed {
            player_query.iter().for_each(|(entity, player)| {
                let group = if player.0 == 1 {
                    Group::GROUP_11
                } else {
                    Group::GROUP_12
                };
                commands
                    .entity(entity)
                    .insert(KinematicCharacterController {
                        filter_groups: Some(CollisionGroups::new(Group::ALL, group)),
                        ..default()
                    });
            });
            *dev = DevPhase::Play;
            return;
        }
    }

    match active_player_query.get_single() {
        Ok(e) => {
            commands.entity(e).remove::<KinematicCharacterController>();
            return;
        }
        Err(_) => {}
    }

    let (
        mut active_player_entity,
        mut transform,
        _texture_atlas,
        mut sprite_image,
        _player_movement,
    ) = active_player_debug.single_mut();

    let left_key_pressed = keyboard_input.pressed(KeyCode::ArrowLeft);
    let right_key_pressed = keyboard_input.pressed(KeyCode::ArrowRight);
    let up_key_pressed = keyboard_input.pressed(KeyCode::ArrowUp);
    let down_key_pressed = keyboard_input.pressed(KeyCode::ArrowDown);

    if left_key_pressed {
        let x = transform.translation.x - 2.0 * PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        if x > -600. + 20. {
            transform.translation.x -= 2.0 * PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        }
        sprite_image.flip_x = false;
    } else if right_key_pressed {
        let x = transform.translation.x + 2.0 * PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        if x < 600. - 20. {
            transform.translation.x += 2.0 * PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        }
        sprite_image.flip_x = true;
    }
    if up_key_pressed {
        let y = transform.translation.y + 2.0 * PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        // if y < 11200. {
        if y < 1300. {
            transform.translation.y += 2.0 * PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        }
    } else if down_key_pressed {
        let y = transform.translation.y - 2.0 * PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        if y > -725. {
            transform.translation.y -= 2.0 * PLAYER_MOVEMENT_SPEED * time.delta_seconds();
        }
    }

    let switch_key_just_pressed = keyboard_input.just_pressed(KeyCode::ShiftLeft);
    if switch_key_just_pressed {
        player_query.iter_mut().for_each(|(e, _)| {
            if e == active_player_entity {
                commands.entity(e).remove::<ActivePlayer>();
            } else {
                commands.entity(e).insert(ActivePlayer);
            }
        });
    }
}

fn keyboard_input_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    dialog_speaker: Res<DialogSpeaker>,
    dialog_speaker_open_dialog: Res<DialogSpeakerOpenDialog>,
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
    mut player_query: Query<(Entity, &mut Transform), With<Player>>,
) {
    if dialog_speaker_open_dialog.0 {
        return;
    }
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
    ) = match active_player_query.get_single_mut() {
        Ok(e) => e,
        Err(_) => return,
    };

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
        // Checks for dialog and disable jumping when in range of speaker
        match dialog_speaker.0 {
            Some(_) => {}
            None => {
                if grounded && player_movement.is_jump_reset {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("jump.wav"),
                        settings: PlaybackSettings {
                            mode: PlaybackMode::Despawn,
                            ..default()
                        },
                    });

                    // player can now jump
                    player_movement.timer.reset();
                    player_movement.airborne = true;
                    player_movement.falling = false;
                    player_movement.is_jump_reset = false;
                    total_y = 600.0;
                }
            }
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
        total_x = -PLAYER_MOVEMENT_SPEED; //* time.delta_seconds();
        sprite_image.flip_x = false;
    } else if right_key_pressed {
        total_x = PLAYER_MOVEMENT_SPEED; //* time.delta_seconds();
        sprite_image.flip_x = true;
    }

    if switch_key_just_pressed {
        player_query.iter_mut().for_each(|(e, mut t)| {
            if e == active_player_entity {
                commands.entity(e).remove::<ActivePlayer>();
                t.translation.z = 10.0;
            } else {
                commands.entity(e).insert(ActivePlayer);
                t.translation.z = 11.0;
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

fn dialog_selection_system(
    mut time: ResMut<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,

    mut dialog_speaker: ResMut<DialogSpeaker>,
    mut dialog_speaker_open_dialog: ResMut<DialogSpeakerOpenDialog>,
    dialog_query: Query<(Entity, &Dialog, &Transform)>,
    mut text_indicator_query: Query<Entity, With<TextIndicator>>,
    selected_text_query: Query<Entity, With<TextIndicatorParentSelector>>,
    mut active_player_query: Query<
        ((&Transform, &mut PlayerMovement)),
        (With<Player>, With<ActivePlayer>),
    >,
) {
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

            *dialog_speaker = DialogSpeaker(None);
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
    *dialog_speaker = DialogSpeaker(Some(entity));
}

fn resetting(
    mut commands: Commands,
    mut time: ResMut<Time>,
    asset_server: Res<AssetServer>,
    mut game_phase: ResMut<GamePhase>,
    mut kinematic_player_query: Query<(Entity, &KinematicCharacterController), With<Player>>,
    mut player_query: Query<(Entity, &mut Transform, &Player), With<Player>>,
    mut drops: Query<Entity, With<BigWaterDrop>>,
) {
    let mut done = false;
    if *game_phase == GamePhase::Reset {
        for (entity, _) in kinematic_player_query.iter_mut() {
            let mut c = commands.entity(entity);
            c.with_children(|child| {
                let texture_handle = asset_server.load("transparentdrop.png");
                child.spawn((
                    Game,
                    SpriteBundle {
                        texture: texture_handle.clone(),
                        transform: Transform {
                            translation: Vec3::new(0., 5.0, 15.0),
                            ..default()
                        },
                        ..default()
                    },
                ));
            });

            c.remove::<KinematicCharacterController>();
        }

        let heights = player_query
            .iter()
            .map(|(_, transform, _)| transform.translation.y)
            .collect::<Vec<_>>();
        let sum: f32 = heights.iter().sum();
        let mut height = sum / heights.len() as f32;

        height -= 3.0 * PLAYER_MOVEMENT_SPEED * time.delta_seconds();

        for (entity, mut transform, player) in player_query.iter_mut() {
            transform.translation.y = height;
        }

        if height < -400.0 {
            for (entity, _, player) in player_query.iter() {
                let group = if player.0 == 1 {
                    Group::GROUP_11
                } else {
                    Group::GROUP_12
                };

                let mut c = commands.entity(entity);
                c.insert(KinematicCharacterController {
                    filter_groups: Some(CollisionGroups::new(Group::ALL, group)),
                    ..default()
                });

                c.despawn_descendants();
            }
            if drops.is_empty() {
                commands.spawn(AudioBundle {
                    source: asset_server.load("collect.mp3"),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        ..default()
                    },
                });
                *game_phase = GamePhase::Play;
                return;
            }
        }

        time.advance_by(Duration::ZERO)
    }
}

fn active_dialog_system(
    mut time: ResMut<Time>,
    mut commands: Commands,
    mut yes_or_no: ResMut<DialogDecisionSelection>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Res<Gamepads>,
    asset_server: Res<AssetServer>,
    axes: Res<Axis<GamepadAxis>>,
    mut water_collection: ResMut<WaterCollection>,
    mut total_score: ResMut<TotalScore>,
    mut game_phase: ResMut<GamePhase>,
    button_inputs: Res<ButtonInput<GamepadButton>>,
    dialog_speaker: ResMut<DialogSpeaker>,
    mut dialog_speaker_open_dialog: ResMut<DialogSpeakerOpenDialog>,
    open_dialog: Query<Entity, With<DialogBox>>,
    mut dialog_query: Query<(Entity, &Dialog, &Transform)>,
    mut player_query: Query<(&Transform, &Player), With<Player>>,
) {
    if !dialog_speaker_open_dialog.0 {
        for entity in &open_dialog {
            commands.entity(entity).despawn_recursive();
        }
    }

    let entity = match dialog_speaker.0 {
        Some(e) => e,
        None => {
            dialog_speaker_open_dialog.0 = false;
            return;
        }
    };

    let (_, dialog, transform) = match dialog_query.iter_mut().find(|(e, _, _)| *e == entity) {
        Some(d) => d,
        None => return,
    };

    let gamepad = match gamepads.iter().next() {
        Some(gp) => gp,
        None => Gamepad::new(0),
    };
    let left_stick_x = match axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX)) {
        Some(x) => x,
        None => 0.0,
    };

    let jump_key_just_pressed = button_inputs
        .just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
        || keyboard_input.just_pressed(KeyCode::Space);

    let left_key_pressed = left_stick_x < -0.10 || keyboard_input.pressed(KeyCode::ArrowLeft);
    let right_key_pressed = left_stick_x > 0.10 || keyboard_input.pressed(KeyCode::ArrowRight);

    let index = if water_collection.total_player1 + water_collection.total_player2 <= 0 {
        0
    } else if !dialog_speaker_open_dialog.0 {
        1
    } else {
        if left_key_pressed {
            yes_or_no.0 = 0
        }

        if right_key_pressed {
            yes_or_no.0 = 1
        }

        if yes_or_no.0 == 0 {
            1
        } else {
            2
        }
    };

    let platform_warning = if dialog.title == "Tlaloc" && (index == 1 || index == 2) {
        let players_too_low = player_query
            .iter()
            .filter(|(t, _)| t.translation.y < 1110.0)
            .collect::<Vec<_>>();
        if players_too_low.is_empty() {
            String::new()
        } else {
            players_too_low
                .iter()
                .fold::<String, _>(String::new(), |s, (_, player)| {
                    if player.0 == 1 {
                        String::from("  (Nena's drops will not contributed)")
                    } else {
                        String::from("  (Nata's drop will not be contributed)")
                    }
                })
        }
    } else {
        String::new()
    };

    if index > 0 && !open_dialog.is_empty() {
        time.advance_by(Duration::ZERO);
        if right_key_pressed || left_key_pressed {
            for entity in &open_dialog {
                commands.entity(entity).despawn_recursive();
            }
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
                    child.spawn((
                        Game,
                        ImageBundle {
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
                        },
                    ));
                    child.spawn((
                        Game,
                        TextBundle::from_section(
                            format!(
                                "{} {}",
                                dialog.dialog.sections[index].value.clone(),
                                platform_warning
                            ),
                            dialog.dialog.sections[0].style.clone(),
                        )
                        .with_text_justify(JustifyText::Left)
                        .with_style(Style {
                            position_type: PositionType::Absolute,
                            align_items: AlignItems::Center,
                            top: Val::Percent(20.5),
                            left: Val::Px(120.0),
                            right: Val::Px(50.0),
                            ..default()
                        }),
                    ));
                });

            return;
        }
        if jump_key_just_pressed {
            if yes_or_no.0 == 1 {
                yes_or_no.0 = 0; // Reset to no after every selection
                if dialog.title == "Tlaloc" {
                    let total = player_query
                        .iter()
                        .filter(|(t, _)| t.translation.y >= 1110.0)
                        .fold::<u32, _>(0, |mut s, (_, player)| {
                            if player.0 == 1 {
                                s += water_collection.total_player1;
                            } else {
                                s += water_collection.total_player2;
                            }
                            s
                        });
                    total_score.0 += total;
                    info!(total);
                    *game_phase = GamePhase::Reset;
                    commands.spawn(AudioBundle {
                        source: asset_server.load("createrain.wav"),
                        settings: PlaybackSettings {
                            mode: PlaybackMode::Despawn,
                            ..default()
                        },
                    });
                }
                water_collection.total_player1 = 0;
                water_collection.total_player2 = 0;
            }
            dialog_speaker_open_dialog.0 = false;
            return;
        }
    } else if !open_dialog.is_empty() {
        if jump_key_just_pressed {
            dialog_speaker_open_dialog.0 = false;
            return;
        }
        time.advance_by(Duration::ZERO);
    } else if jump_key_just_pressed || dialog_speaker_open_dialog.0 {
        dialog_speaker_open_dialog.0 = true;

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
                child.spawn((
                    Game,
                    ImageBundle {
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
                    },
                ));
                child.spawn((
                    Game,
                    TextBundle::from_section(
                        format!(
                            "{} {}",
                            dialog.dialog.sections[index].value.clone(),
                            platform_warning
                        ),
                        dialog.dialog.sections[0].style.clone(),
                    )
                    .with_text_justify(JustifyText::Left)
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        top: Val::Percent(20.5),
                        left: Val::Px(120.0),
                        right: Val::Px(50.0),
                        ..default()
                    }),
                ));
            });
    }
}

pub struct PlatformPlugin;
impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), (setup))
            .init_resource::<GamePhase>()
            .init_resource::<DevPhase>()
            .insert_resource(WaterCollection::default())
            .insert_resource(GustTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
            .insert_resource(DialogSpeaker::default())
            .insert_resource(DialogSpeakerOpenDialog::default())
            .insert_resource(DialogDecisionSelection::default())
            .insert_resource(TotalTime(Timer::from_seconds(999.0, TimerMode::Once)))
            .insert_resource(TotalScore(0))
            .add_systems(PreUpdate, camera_tracking::camera_tracking_system)
            .add_systems(Update, time_count_system)
            .add_systems(
                Update,
                (
                    dialog_selection_system,
                    active_dialog_system,
                    keyboard_input_system,
                    resetting,
                    player_gravity_system,
                    translate_player_system,
                    cloud_movement,
                    platform_sensor_system,
                    gust_system,
                    collectable_system,
                    debug_system,
                )
                    .chain()
                    .run_if(in_state(AppState::Game)),
            )
            .add_systems(OnExit(AppState::Game), despawn_screen::<Game>);
    }
}
