use crate::{Play, Player, RustAnimationAtlas};
use bevy::prelude::*;
use bevy_rapier2d::control::KinematicCharacterControllerOutput;
use bevy_rapier2d::render::DebugRenderContext;
use iyes_perf_ui::prelude::{PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot};
use std::cmp::PartialEq;
use std::time::Duration;

#[derive(Component, Debug)]
pub struct PlayerControls {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub crouch: bool,
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
}

impl Default for PlayerControls {
    fn default() -> Self {
        Self {
            left: false,
            right: false,
            jump: false,
            crouch: false,
        }
    }
}

pub fn update_player_controls(
    mut step_timer: Local<Timer>,
    time: Res<Time>,
    mut controls: Query<&mut PlayerControls>,
    state: Query<&PlayerState>,
    input: Res<ButtonInput<KeyCode>>,
    mut debug: ResMut<DebugRenderContext>,
    mut commands: Commands,
    ui: Query<Entity, With<PerfUiRoot>>,
) {
    if step_timer.mode() != TimerMode::Repeating {
        step_timer.set_mode(TimerMode::Repeating);
        step_timer.set_duration(Duration::from_secs_f32(0.3));
    }
    if let Ok(mut controls) = controls.get_single_mut() {
        if input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::Space]) {
            controls.jump = true
        } else {
            controls.jump = false
        }
        if input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
            controls.crouch = true
        } else {
            controls.crouch = false
        }
        if input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
            controls.left = true
        } else {
            controls.left = false
        }
        if input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
            controls.right = true
        } else {
            controls.right = false
        }
        if input.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::Space]) {
            commands.trigger(Play::Jump);
        }
        if input.any_pressed([
            KeyCode::KeyA,
            KeyCode::KeyD,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        ]) {
            if let Ok(state) = state.get_single() {
                if state.animation_state == AnimationState::Walking {
                    step_timer.tick(time.delta());
                    if step_timer.just_finished() {
                        commands.trigger(Play::Walk);
                    }
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

pub fn update_player_states(
    mut state: Query<&mut PlayerState, With<Player>>,
    controls: Query<&PlayerControls>,
    ground_query: Query<&KinematicCharacterControllerOutput>,
) {
    if let Ok(mut state) = state.get_single_mut() {
        let controls = if let Ok(controls) = controls.get_single() {
            controls
        } else {
            &PlayerControls::default()
        };
        if let Ok(ground) = ground_query.get_single() {
            use AnimationDirection::*;
            use AnimationState::*;
            state.animation_state = match (
                !ground.grounded,
                controls.left || controls.right,
                controls.crouch,
            ) {
                (false, false, false) => Idle,
                (false, false, true) => Crouching,
                (false, true, true) => CrouchWalking,
                (false, true, false) => Walking,
                (true, _, _) => Jumping,
            };
            state.direction = match (controls.left, controls.right) {
                (true, false) => Left,
                (false, true) => Right,
                (_, _) => state.direction,
            };
        }
    }
}

pub fn update_player_animation(
    mut player: Query<(&mut Sprite, &PlayerState, &mut RustAnimationAtlas), With<Player>>,
) {
    if let Ok((mut sprite, state, mut animation)) = player.get_single_mut() {
        animation.set_current(match state.animation_state {
            AnimationState::Idle => 0,
            AnimationState::Walking => 1,
            AnimationState::Running => 2,
            AnimationState::Crouching => 3,
            AnimationState::Jumping => 4,
            AnimationState::CrouchWalking => 5,
        });
        sprite.flip_x = match state.direction {
            AnimationDirection::Left => true,
            AnimationDirection::Right => false,
        };
    }
}
