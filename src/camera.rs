use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Default)]
pub struct Follow;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera2dBundle {
            projection: OrthographicProjection {
                near: -1000.0,
                scaling_mode: ScalingMode::FixedHorizontal(350.0),
                ..default()
            },
            ..default()
        },
    ));
}

pub fn move_camera(
    mut cam_query: Query<(&mut Transform), With<MainCamera>>,
    follow: Query<&Transform, (With<Follow>, Without<MainCamera>)>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = cam_query.get_single_mut() {
        if let Ok(fol) = follow.get_single() {
            let dir = transform.translation.x - fol.translation.x;
            if dir.abs() > 16.0 {
                transform.translation.x -= dir * time.delta_seconds() * 3.0;
            }
            let dir = transform.translation.y - fol.translation.y;
            if dir.abs() > 48.0 {
                transform.translation.y -= dir * time.delta_seconds() * 3.0;
            }
        }
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(PostUpdate, move_camera);
    }
}
