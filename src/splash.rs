use crate::despawn_screen;
use crate::AppState;
use bevy::{prelude::*, render::view::RenderLayers};

#[derive(Component)]
pub struct OnSplashScreen;

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), splash_setup)
            .add_systems(Update, countdown.run_if(in_state(AppState::Splash)))
            .add_systems(OnExit(AppState::Splash), despawn_screen::<OnSplashScreen>);
    }
}

#[derive(Component)]
struct SplashCamera;

fn splash_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut bg: ResMut<ClearColor>,
) {
    bg.0 = Color::BLACK;
    let icon = asset_server.load("bevylogo.png");
    commands.spawn((
        OnSplashScreen,
        SplashCamera,
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

    commands
        .spawn((
            RenderLayers::layer(2),
            NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
            OnSplashScreen,
        ))
        .with_children(|parent| {
            parent.spawn((ImageBundle {
                style: Style {
                    width: Val::Px(200.0),
                    ..default()
                },
                image: UiImage::new(icon),
                ..default()
            },));
        });

    commands.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));

    commands.spawn((
        RenderLayers::layer(2),
        OnSplashScreen,
        TextBundle::from_section(
            "Made with Bevy",
            TextStyle {
                font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                font_size: 52.0,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            bottom: Val::Percent(20.0),
            left: Val::Percent(20.0),

            ..default()
        }),
    ));
}

fn countdown(
    mut app_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).finished() {
        app_state.set(AppState::Game);
    }
}
