use crate::player_movement::{Jump, Run};
use crate::{Play, Player, RustAnimationAtlas};
use bevy::prelude::*;
use bevy_rapier2d::render::DebugRenderContext;
use iyes_perf_ui::prelude::{PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot};
use std::cmp::PartialEq;
use std::time::Duration;

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

#[derive(Component, Copy, Clone, Debug)]
pub struct PlayerState {
    animation_state: AnimationState,
    direction: AnimationDirection,
}
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            animation_state: AnimationState::Idle,
            direction: AnimationDirection::Right,
        }
    }
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnimationDirection {
    Left,
    Right,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnimationState {
    Idle,
    Walking,
    Running,
    Crouching,
    Jumping,
    CrouchWalking,
    Braking,
}

pub fn update_player_controls(
    mut input_buffering: Local<InputBuffer>,
    mut step_timer: Local<Timer>,
    time: Res<Time>,
    mut player_components_query: Query<(&PlayerState, &mut Run, &mut Jump), With<Player>>,
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
                commands.trigger(Play::Jump)
            }
        }
        if input.any_pressed(jump_buttons) {
            jump.jump_held = true;
        } else {
            jump.jump_held = false;
        }
        if input.any_pressed(jump_buttons) && input_buffering.can_jump() {
            if jump.try_jump() {
                commands.trigger(Play::Jump)
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
                    commands.trigger(Play::Walk);
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

pub fn update_player_states(mut state: Query<(&mut PlayerState, &Jump, &Run), With<Player>>) {
    if let Ok((mut state, jump, run)) = state.get_single_mut() {
        use AnimationDirection::*;
        use AnimationState::*;
        state.direction = match run.running {
            Some(-1.0) => Left,
            Some(1.0) => Right,
            _ => state.direction,
        };

        state.animation_state = match (jump.jumping, run.running, jump.grounded) {
            (false, None, true) => Idle,
            (false, Some(_), true) => Walking,
            (true, _, false) => Jumping,
            (true, _, true) => Jumping,
            _ => Idle,
        };
    }
}

pub fn update_player_animation(
    mut player: Query<(&mut Sprite, &PlayerState, &mut RustAnimationAtlas), With<Player>>,
) {
    if let Ok((mut sprite, state, mut animation)) = player.get_single_mut() {
        animation.set_current(match state.animation_state {
            AnimationState::Idle => 0,
            AnimationState::Walking => 2,
            AnimationState::Running => 2,
            AnimationState::Crouching => 3,
            AnimationState::Jumping => 4,
            AnimationState::CrouchWalking => 5,
            AnimationState::Braking => 4,
        });
        sprite.flip_x = match state.direction {
            AnimationDirection::Left => true,
            AnimationDirection::Right => false,
        };
    }
}
