use crate::entities::player::PlayerMarker;
use crate::player_movement::{
    player_wall_ceiling_checks, update_character_position_from_velocity, update_jump_component,
    update_run_component, update_speedometer, Jump, Run,
};
use crate::{PlaySoundEffect, RustAnimationAtlas};
use bevy::prelude::*;
use bevy_rapier2d::render::DebugRenderContext;
use iyes_perf_ui::prelude::{PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot};
use std::cmp::PartialEq;
use std::time::Duration;

pub struct PlayerControlPlugin;
impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (update_player_controls, player_wall_ceiling_checks),
        );
        app.add_systems(
            Update,
            (
                update_speedometer,
                update_character_position_from_velocity,
                update_jump_component.before(update_character_position_from_velocity),
                update_run_component.before(update_character_position_from_velocity),
                update_player_states,
                update_player_animation,
            ),
        );
    }
}

#[derive(Resource)]
pub struct InputBuffer {
    jump: Timer,
}
impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            jump: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}
impl InputBuffer {
    pub fn tick(&mut self, delta: Duration) {
        self.jump.tick(delta);
    }
    pub fn reset(&mut self) {
        self.jump.reset();
    }
    pub fn can_jump(&self) -> bool {
        !self.jump.finished()
    }
}

#[derive(Component, Copy, Clone, Debug, Default)]
pub struct PlayerState {
    animation_state: AnimationState,
    direction: AnimationDirection,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum AnimationDirection {
    #[default]
    Left,
    Right,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum AnimationState {
    // platformer, left/right by image flip.
    #[default]
    Idle,
    Walking,
    Running,
    Crouching,
    Jumping,
    CrouchWalking,
    Braking,
    Pushing,
    Grabbing,
    GrabWalk,
}

pub fn update_player_controls(
    mut input_buffering: Local<InputBuffer>,
    mut step_timer: Local<Timer>,
    time: Res<Time>,
    mut player_components_query: Query<(&PlayerState, &mut Run, &mut Jump), With<PlayerMarker>>,
    input: Res<ButtonInput<KeyCode>>,
    mut debug: ResMut<DebugRenderContext>,
    mut commands: Commands,
    ui: Query<Entity, With<PerfUiRoot>>,
) {
    let jump_buttons = [KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::Space];
    let crouch_buttons = [KeyCode::KeyS, KeyCode::ArrowDown];
    let left_buttons = [KeyCode::KeyA, KeyCode::ArrowLeft];
    let right_buttons = [KeyCode::KeyD, KeyCode::ArrowRight];

    if step_timer.mode() != TimerMode::Repeating {
        step_timer.set_mode(TimerMode::Repeating);
        step_timer.set_duration(Duration::from_secs_f32(0.3));
    }
    if let Ok((state, mut run, mut jump)) = player_components_query.get_single_mut() {
        input_buffering.tick(time.delta());
        if input.any_just_pressed(jump_buttons) {
            input_buffering.reset();
            if jump.try_jump() {
                commands.trigger(PlaySoundEffect::Jump)
            }
        }
        if input.any_pressed(jump_buttons) {
            jump.jump_held = true;
        } else {
            jump.jump_held = false;
        }
        if input.any_pressed(jump_buttons) && input_buffering.can_jump() {
            if jump.try_jump() {
                commands.trigger(PlaySoundEffect::Jump)
            }
        }
        if input.any_pressed(crouch_buttons) {
            //Not yet implemented (or decided..)
        }
        run.running = match (
            input.any_pressed(left_buttons),
            input.any_pressed(right_buttons),
        ) {
            (true, false) => Some(-1.0),
            (false, true) => Some(1.0),
            _ => None,
        };

        if input.any_pressed([
            KeyCode::KeyA,
            KeyCode::KeyD,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        ]) {
            if state.animation_state == AnimationState::Walking {
                step_timer.tick(time.delta());
                if step_timer.just_finished() {
                    commands.trigger(PlaySoundEffect::Walk);
                }
            }
        }
    }

    if input.just_pressed(KeyCode::Digit2) {
        debug.enabled = !debug.enabled;
    }

    if input.just_pressed(KeyCode::Digit1) {
        if let Ok(ui) = ui.get_single() {
            commands.entity(ui).despawn_recursive();
        } else {
            commands.spawn((
                PerfUiRoot {
                    display_labels: false,
                    layout_horizontal: true,
                    ..default()
                },
                PerfUiEntryFPSWorst::default(),
                PerfUiEntryFPS::default(),
            ));
        }
    }
}

pub fn update_player_states(mut state: Query<(&mut PlayerState, &Jump, &Run), With<PlayerMarker>>) {
    if let Ok((mut state, jump, run)) = state.get_single_mut() {
        use AnimationDirection::*;
        use AnimationState::*;
        state.direction = match run.running {
            Some(-1.0) => Left,
            Some(1.0) => Right,
            _ => state.direction,
        };

        state.animation_state = match (jump.jumping, run.running, jump.grounded) {
            (_, Some(_), true) => Walking,
            (_, _, false) => Jumping,
            _ => Idle,
        };
    }
}

pub fn update_player_animation(
    mut player: Query<(&mut Sprite, &PlayerState, &mut RustAnimationAtlas), With<PlayerMarker>>,
) {
    if let Ok((mut sprite, state, mut animation)) = player.get_single_mut() {
        animation.set_current(match state.animation_state {
            AnimationState::Idle => 0,
            AnimationState::Walking => 2,
            AnimationState::Jumping => 4,
            _ => 0,
        });
        sprite.flip_x = match state.direction {
            AnimationDirection::Left => true,
            AnimationDirection::Right => false,
        };
    }
}
