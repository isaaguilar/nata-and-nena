use crate::camera_tracking;
use crate::GameState;
use bevy::{prelude::*, render::texture::ImageLoader};

#[derive(Component)]
pub struct Player;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut bg: ResMut<ClearColor>,
) {
    bg.0 = Color::srgb(0.623, 0.689, 0.876);

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
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
        SpriteBundle {
            texture: texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 10.0),
                ..default()
            },
            sprite: Sprite {
                flip_x: false,
                ..default()
            },
            ..default()
        },
    ));

    let texture_handle = asset_server.load("gradient.png");

    commands.spawn((SpriteBundle {
        texture: texture_handle.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, 6000.0 - 750., -10.0),
            ..default()
        },
        ..default()
    },));
}

pub fn platform_sensor_system(mut commands: Commands) {}

fn keyboard_input_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Res<Gamepads>,
    button_inputs: Res<ButtonInput<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut sprite_position: Query<
        (
            Entity,
            &mut Player,
            &mut Transform,
            &mut TextureAtlas,
            &mut Sprite,
        ),
        With<Player>,
    >,
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
    let jump_key_pressed = button_inputs
        .pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
        || keyboard_input.pressed(KeyCode::Space);
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

    let (entity, mut player, mut transform, mut sprite, mut timer) = sprite_position.single_mut();

    if left_key_pressed {
        transform.translation.x -= 100.0 * time.delta_seconds();
    } else if right_key_pressed {
        transform.translation.x += 100.0 * time.delta_seconds();
    } else if up_key_pressed {
        transform.translation.y += 100.0 * time.delta_seconds();
    } else if down_key_pressed {
        transform.translation.y -= 100.0 * time.delta_seconds();
    }
}

fn non_system() {}

pub struct PlatformPlugin;
impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), (setup))
            .add_systems(PreUpdate, camera_tracking::camera_tracking_system)
            .add_systems(
                Update,
                (keyboard_input_system).run_if(in_state(GameState::Game)),
            );
    }
}
