use crate::animation::{RustAnimation, RustAnimationAtlas};
use crate::camera::Follow;
use crate::player_controls::PlayerState;
use crate::player_movement::{Jump, Run, SideChecks};
use crate::PlayerText;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component, Default)]
pub struct PlayerMarker {
    position: Vec2,
}
#[derive(Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle("character.png", 32, 32, 3, 2, 0, 0, 2)]
    sprite_bundle: LdtkSpriteSheetBundle,
    player: PlayerMarker,
    follow: Follow,
    side_checks: SideChecks,
    jump: Jump,
    run: Run,
    state: PlayerState,
    rigid_body: RigidBody,
    rust_animation_atlas: RustAnimationAtlas,
    collider: Collider,
    locked_axis: LockedAxes,
    kinematic_character_controller: KinematicCharacterController,
    #[worldly]
    worldly: Worldly,
}
impl Default for PlayerBundle {
    fn default() -> Self {
        PlayerBundle {
            rigid_body: RigidBody::KinematicVelocityBased,
            rust_animation_atlas: RustAnimationAtlas::new([
                RustAnimation::list([0], 0.1),
                RustAnimation::list([0, 1, 2, 3], 0.1),
                RustAnimation::list([0, 1, 2, 3, 4], 0.1),
                RustAnimation::list([0], 0.1),
                RustAnimation::list([5], 0.1),
                RustAnimation::list([0, 1, 2, 3], 0.1),
            ]),
            collider: Collider::cuboid(7.75, 7.75),
            locked_axis: LockedAxes::ROTATION_LOCKED,
            player: Default::default(),
            follow: Default::default(),
            side_checks: Default::default(),
            jump: Default::default(),
            run: Default::default(),
            state: Default::default(),
            sprite_bundle: Default::default(),
            kinematic_character_controller: KinematicCharacterController {
                translation: None,
                offset: CharacterLength::Relative(0.01),
                normal_nudge_factor: 0.001,
                slide: true,
                snap_to_ground: Some(CharacterLength::Relative(0.03)),
                apply_impulse_to_dynamic_bodies: true,
                ..default()
            },
            worldly: Default::default(),
        }
    }
}
#[derive(Bundle)]
pub struct PlayerChildBundle {
    player_text: PlayerText,
    text_2d: Text2dBundle,
}
impl Default for PlayerChildBundle {
    fn default() -> Self {
        Self {
            player_text: PlayerText,
            text_2d: Text2dBundle {
                text: Text::from_section(
                    "Hello World",
                    TextStyle {
                        font: Default::default(),
                        font_size: 40.0,
                        color: Color::srgb(0.0, 0.0, 0.0),
                    },
                )
                .with_justify(JustifyText::Center),
                transform: Transform::from_xyz(0.0, 20.0, 0.0).with_scale(Vec3::splat(0.1)),
                ..default()
            },
        }
    }
}

pub fn spawn_player(
    mut new_player: Query<(Entity, &mut Sprite), Added<PlayerMarker>>,
    mut commands: Commands,
) {
    for (player, mut sprite) in new_player.iter_mut() {
        sprite.anchor = Anchor::Custom(vec2(0.0, -0.25));
        commands
            .spawn(PlayerChildBundle::default())
            .set_parent(player);
    }
}
