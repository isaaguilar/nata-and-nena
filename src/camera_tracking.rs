use crate::camera::MainCamera;
use crate::game::Player;
use bevy::prelude::*;

pub fn camera_tracking_system(
    time: Res<Time>,
    sprite_position: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut MainCamera, &mut Transform), (Without<Player>)>,
) {
    // TODO track two players that have a diff < screen height else game over

    let m = 1.0_f32;
    let k = 1.3_f32;

    let b = 2.0 * (m * k).sqrt();

    let camera_above_center_const = 52.5;
    let player_transform = match sprite_position.get_single() {
        Ok(t) => t,
        Err(_) => return,
    };

    // let player_transform = sprite_position.single();

    let (mut camera_data, mut camera_transform) = match camera_query.get_single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };

    if camera_transform.translation.is_nan() {
        info!("Init camera");
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y + camera_above_center_const;
        return;
    }

    let current_x = camera_transform.translation.x;
    let player_x = player_transform.translation.x;
    let distance = current_x - player_x;
    if distance.abs() < 1.0 {
        // info!("Snapped to center");
        camera_data.time_delta = 0.0;
        camera_transform.translation.x = player_transform.translation.x;
    } else {
        let mut v = distance / time.delta_seconds();
        let mut delta = camera_data.time_delta;
        delta += time.delta_seconds();
        camera_data.time_delta = delta;
        // d = vt

        let a = -b * v - k * distance;
        v += a * time.delta_seconds();
        let x = v * time.delta_seconds();
        camera_transform.translation.x = player_x + x;
    }

    let m = 1.0_f32; // Mass
    let k = 8.5_f32; // Spring constant
    let b = 2.0 * (m * k).sqrt();
    let current_y = camera_transform.translation.y;
    let player_y = player_transform.translation.y + camera_above_center_const;
    let distance_y = current_y - player_y;
    if distance_y.abs() < 1.0 {
        camera_transform.translation.y = player_transform.translation.y + camera_above_center_const;
    } else {
        let mut v = distance_y / time.delta_seconds();

        let a = -b * v - k * distance_y;
        v += a * time.delta_seconds();
        let y = v * time.delta_seconds();
        camera_transform.translation.y = player_y + y;
    }
}
