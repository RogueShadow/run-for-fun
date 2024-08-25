use crate::*;
use bevy::math::Vec2;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct PlayerMovement {
    pub velocity: Vec2,
    pub damping: Vec2,
    pub speed: f32,
    pub jump: f32,
    pub just_jumped: bool,
    pub jumping: bool,
    pub jump_duration: Time,
    pub jump_max_duration: f32,
}
impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            damping: Vec2::new(0.98, 0.98),
            speed: 100.0,
            jump: 550.0,
            just_jumped: false,
            jumping: false,
            jump_duration: Time::default(),
            jump_max_duration: 0.5,
        }
    }
}

impl PlayerMovement {
    pub(crate) fn damping(&mut self, delta: f32) {
        self.velocity *= self.damping * delta;
    }
}

pub fn update_character_position_from_velocity(
    mut player_query: Query<(&mut KinematicCharacterController, &PlayerMovement), With<Player>>,
) {
    if let Ok((mut controller, movement)) = player_query.get_single_mut() {
        controller.translation = Some(movement.velocity);
    }
}

pub fn update_movement_component(
    mut entities: Query<(
        &mut PlayerMovement,
        &PlayerControls,
        &KinematicCharacterControllerOutput,
    )>,
    config: Res<RapierConfiguration>,
    time: Res<Time<Fixed>>,
    time_virtual: Res<Time<Virtual>>,
) {
    for (mut movement, controls, output) in entities.iter_mut() {
        let delta = time.delta().as_secs_f32();
        movement.damping(delta);
        match (controls.jump, movement.just_jumped, movement.jumping) {
            (true, false, false) => {
                movement.just_jumped = true;
                movement.jumping = true;
                let jump_strength_mod =
                    1.0 - (movement.jump_duration.elapsed_seconds() / movement.jump_max_duration);
                if output.grounded {
                    movement.velocity.y += movement.jump * delta * jump_strength_mod;
                } else if movement.jump_duration.elapsed_seconds() < 0.5 {
                    movement.velocity.y += movement.jump * delta * jump_strength_mod;
                } else {
                    movement.jumping = false
                }
            }
            (true, true, true) => {
                let jump_strength_mod =
                    1.0 - (movement.jump_duration.elapsed_seconds() / movement.jump_max_duration);
                if output.grounded {
                    movement.velocity.y += movement.jump * delta * jump_strength_mod;
                } else if movement.jump_duration.elapsed_seconds() < 0.5 {
                    movement.velocity.y += movement.jump * delta * jump_strength_mod;
                } else {
                    movement.jumping = false
                }
            }
            (false, _, _) => {
                movement.just_jumped = false;
                movement.jumping = false;
            }
            _ => {}
        }

        if controls.left {
            movement.velocity.x -= movement.speed * delta;
        }
        if controls.right {
            movement.velocity.x += movement.speed * delta;
        }
        if controls.crouch {}

        if output.grounded {
            movement.jump_duration = Time::default();
        } else {
            movement.jump_duration.advance_by(time_virtual.delta());
        }

        movement.velocity += config.gravity * delta;
    }
}
