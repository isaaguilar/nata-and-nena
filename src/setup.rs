use bevy::{
    asset::AssetMetaCheck,
    log::{Level, LogPlugin},
    prelude::*,
    window::WindowMode,
};

pub struct WindowSetup;

impl Plugin for WindowSetup {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Nata and Nena".into(),
                        mode: WindowMode::Windowed,
                        resize_constraints: WindowResizeConstraints {
                            max_width: 1200.0,
                            max_height: 750.0,
                            min_width: 1200.0,
                            min_height: 750.0,
                        },
                        resolution: (1200., 750.).into(),
                        prevent_default_event_handling: false,
                        visible: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::INFO,
                    filter: "wgpu=error,bevy_render=info,bevy_ecs=info".to_string(),
                    custom_layer: |_| None,
                }),
        )
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::srgb(0.623, 0.689, 0.876)));
    }
}
