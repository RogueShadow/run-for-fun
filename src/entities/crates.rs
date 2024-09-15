use bevy::prelude::{Bundle, SpriteBundle};
use bevy_ecs_ldtk::LdtkEntity;
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::Collider;

#[derive(Bundle, LdtkEntity)]
pub struct Crate {
    #[sprite_bundle]
    sprite_bundle: SpriteBundle,
    rigid_body: RigidBody,
    collider: Collider,
}
impl Default for Crate {
    fn default() -> Self {
        Self {
            collider: Collider::cuboid(8.0, 8.0),
            rigid_body: RigidBody::Dynamic,
            sprite_bundle: Default::default(),
        }
    }
}
