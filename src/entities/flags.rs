use crate::animation::RustAnimation;
use crate::{Finish, Start};
use bevy::math::vec2;
use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkEntity, LdtkSpriteSheetBundle};
use bevy_rapier2d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};

#[derive(Bundle, LdtkEntity, Default)]
pub struct StartFlag {
    start: Start,
    #[sprite_sheet_bundle("flag_red_green.png", 125, 250, 4, 2, 0, 0, 4)]
    sprite_bundle: LdtkSpriteSheetBundle,
    flag_bundle: FlagBundle,
}
#[derive(Bundle, LdtkEntity)]
pub struct FlagBundle {
    collider: Collider,
    sensor: Sensor,
    active_collision_types: ActiveCollisionTypes,
    active_events: ActiveEvents,
}
impl Default for FlagBundle {
    fn default() -> Self {
        Self {
            collider: Collider::cuboid(8.0, 16.0),
            active_collision_types: ActiveCollisionTypes::KINEMATIC_STATIC,
            active_events: ActiveEvents::COLLISION_EVENTS,
            sensor: Sensor,
        }
    }
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct FinishFlag {
    finish: Finish,
    #[sprite_sheet_bundle("flag_red_green.png", 125, 250, 4, 2, 0, 0, 0)]
    sprite_bundle: LdtkSpriteSheetBundle,
    flag_bundle: FlagBundle,
}

pub fn spawn_flags(
    mut start: Query<(Entity, &mut Sprite), (Added<Start>, Without<Finish>)>,
    mut finish: Query<(Entity, &mut Sprite), (Added<Finish>, Without<Start>)>,
    mut commands: Commands,
) {
    for (entity, mut sprite) in start.iter_mut() {
        sprite.custom_size = Some(vec2(16.0, 32.0));
        commands
            .entity(entity)
            .insert(RustAnimation::range(4, 7, 0.1));
    }
    for (entity, mut sprite) in finish.iter_mut() {
        sprite.custom_size = Some(vec2(16.0, 32.0));
        commands
            .entity(entity)
            .insert(RustAnimation::range(0, 3, 0.1));
    }
}
