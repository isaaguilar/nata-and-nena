use crate::camera::MainCamera;
use crate::game::{ActivePlayer, Player};
use bevy::log::info;
use bevy::prelude::*;

pub fn camera_tracking_system(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut MainCamera, &mut Transform), (Without<Player>)>,
) {
    // TODO track two players that have a diff < screen height else game over

    let camera_above_center_const = 100.0;
    // let player_transform = match {
    //     Ok(t) => t,
    //     Err(_) => return,
    // };

    let player_average_position =
        player_query
            .iter()
            .enumerate()
            .fold::<Vec2, _>(Vec2::new(0.0, 0.0), |v, (i, t)| {
                Vec2::new(
                    (v.x + t.translation.x) / (i + 1) as f32,
                    (v.y + t.translation.y) / (i + 1) as f32,
                )
            });

    info!("{:?}", player_average_position);

    let (_, mut camera_transform) = match camera_query.get_single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };

    if camera_transform.translation.is_nan() {
        info!("Init camera");
        camera_transform.translation.x = player_average_position.x;
        camera_transform.translation.y = player_average_position.y + camera_above_center_const;
        return;
    }

    let m = 1.0_f32;
    let k = 8.5_f32;
    let b = 2.0 * (m * k).sqrt();
    let current_y = camera_transform.translation.y;
    let player_y = player_average_position.y + camera_above_center_const;
    let distance_y = current_y - player_y;
    if distance_y.abs() < 1.0 {
        camera_transform.translation.y = player_average_position.y + camera_above_center_const;
    } else {
        let mut v = distance_y / time.delta_seconds();

        let a = -b * v - k * distance_y;
        v += a * time.delta_seconds();
        let y = v * time.delta_seconds();
        let final_y = player_y + y;
        if final_y > -375. && final_y < 10850. {
            camera_transform.translation.y = final_y;
        }
    }
}
