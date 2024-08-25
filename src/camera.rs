use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Follow;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera2dBundle {
            transform: Transform::default().with_scale(Vec3::splat(0.2)),
            ..default()
        },
    ));
}

pub fn move_camera(
    mut camera: Query<&mut Transform, With<MainCamera>>,
    follow: Query<&Transform, (With<Follow>, Without<MainCamera>)>,
) {
    if let Ok(mut cam) = camera.get_single_mut() {
        if let Ok(fol) = follow.get_single() {
            cam.translation = fol.translation;
        }
    }
}
