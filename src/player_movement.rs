use crate::entities::player::PlayerMarker;
use crate::*;
use bevy::math::Vec2;
use bevy::prelude::*;
use num_traits::Inv;
use std::ops::Deref;

#[derive(Debug, Reflect)]
pub enum Distance {
    Meters(f32),
    Pixels(f32),
}
impl Distance {
    pub const PIXELS_PER_METER: f32 = 16.0;
    pub fn pixels(pixels: f32) -> Distance {
        Distance::Pixels(pixels)
    }
    pub fn meters(meters: f32) -> Distance {
        Distance::Meters(meters)
    }
    pub fn to_pixels(&self) -> f32 {
        match self {
            Distance::Meters(m) => m * Distance::PIXELS_PER_METER,
            Distance::Pixels(p) => *p,
        }
    }
    pub fn to_meters(&self) -> f32 {
        match self {
            Distance::Meters(m) => *m,
            Distance::Pixels(p) => *p / Distance::PIXELS_PER_METER,
        }
    }
}
impl Default for Distance {
    fn default() -> Self {
        Self::Meters(0.0)
    }
}
#[derive(Component, Debug)]
pub struct Speedometer {
    pub last_position: Vec2,
    pub speed: Vec2,
    pub timer: Timer,
}
impl Default for Speedometer {
    fn default() -> Self {
        Self {
            last_position: Vec2::ZERO,
            speed: Vec2::ZERO,
            timer: Timer::from_seconds(0.25, TimerMode::Repeating),
        }
    }
}
pub fn update_speedometer(
    mut speedometer_query: Query<(&mut Speedometer, &Transform)>,
    time: Res<Time>,
) {
    for (mut speedometer, transform) in speedometer_query.iter_mut() {
        speedometer.timer.tick(time.delta());
        if speedometer.timer.just_finished() {
            speedometer.speed = (speedometer.last_position - transform.translation.xy()).abs();
            speedometer.last_position = transform.translation.xy();
            let multiplier = 1.0 / speedometer.timer.duration().as_secs_f32();
            info!(
                "Run Speed: {:?} m/s\nAir Speed: {:?}",
                Distance::pixels(speedometer.speed.x * multiplier).to_meters(),
                Distance::pixels(speedometer.speed.y * multiplier).to_meters(),
            );
        }
    }
}

#[derive(Reflect, Component, Debug)]
#[reflect(Component)]
pub struct Jump {
    pub jumping: bool,
    pub max_distance: Distance,
    pub min_distance: Distance,
    pub speed: Distance,
    pub current_distance: f32,
    pub velocity: f32,
    pub grounded: bool,
    pub ceiling: bool,
    pub jump_held: bool,
}
impl Jump {
    pub fn try_jump(&mut self) -> bool {
        if !self.jumping && self.grounded && !self.ceiling {
            self.jumping = true;
            true
        } else {
            false
        }
    }
}
impl Default for Jump {
    fn default() -> Self {
        Self {
            jumping: false,                      // am I actually jumping
            max_distance: Distance::Meters(4.0), // max  height
            min_distance: Distance::Meters(2.0), // min height
            speed: Distance::Meters(10.0),       // max jump speed, also for falling atm.
            current_distance: 0.0,               // height of jump
            velocity: 0.0, // output of this component, used with run output to check position for collision and update.
            grounded: false, // set to reference if on ground by physics component.
            ceiling: false,
            jump_held: false, // set in controls to reference button being held.
        }
    }
}

#[derive(Component, Default)]
pub struct SideChecks {
    left_wall: bool,
    right_wall: bool,
    ceiling: bool,
    ground: bool,
}
pub fn player_wall_ceiling_checks(
    mut player_query: Query<
        (
            Entity,
            &Collider,
            &Transform,
            &mut SideChecks,
            &KinematicCharacterControllerOutput,
        ),
        With<PlayerMarker>,
    >,
    physics: Res<RapierContext>,
) {
    if let Ok((player, collider, transform, mut sides, output)) = player_query.get_single_mut() {
        sides.ground = output.grounded;
        let filter = QueryFilter::new()
            .exclude_collider(player)
            .exclude_sensors();
        if let Some((_, _)) = physics.cast_shape(
            transform.translation.xy(),
            0.0,
            vec2(-1.0, 0.0),
            collider,
            ShapeCastOptions {
                max_time_of_impact: 1.0,
                target_distance: 0.0,
                stop_at_penetration: true,
                compute_impact_geometry_on_penetration: false,
            },
            filter,
        ) {
            sides.left_wall = true;
        } else {
            sides.left_wall = false;
        }

        if let Some((_, _)) = physics.cast_shape(
            transform.translation.xy(),
            0.0,
            vec2(1.0, 0.0),
            collider,
            ShapeCastOptions {
                max_time_of_impact: 1.0,
                target_distance: 0.0,
                stop_at_penetration: true,
                compute_impact_geometry_on_penetration: false,
            },
            filter,
        ) {
            sides.right_wall = true;
        } else {
            sides.right_wall = false;
        }

        if let Some((_, _)) = physics.cast_shape(
            transform.translation.xy(),
            0.0,
            vec2(0.0, 1.0),
            collider,
            ShapeCastOptions {
                max_time_of_impact: 1.0,
                target_distance: 0.0,
                stop_at_penetration: true,
                compute_impact_geometry_on_penetration: false,
            },
            QueryFilter::exclude_dynamic()
                .exclude_sensors()
                .exclude_collider(player),
        ) {
            sides.ceiling = true;
        } else {
            sides.ceiling = false;
        }
    }
}
pub fn update_jump_component(mut entities: Query<(&mut Jump, &SideChecks)>, time: Res<Time>) {
    for (mut jump, output) in entities.iter_mut() {
        jump.grounded = output.ground;
        jump.ceiling = output.ceiling;

        let max_complete = (jump.current_distance / jump.max_distance.to_pixels()).min(1.0);
        let min_complete = (jump.current_distance / jump.min_distance.to_pixels()).min(1.0);
        if output.ceiling {
            jump.jumping = false;
        }
        if (jump.jumping && jump.jump_held && max_complete < 1.0)
            || (jump.jumping && min_complete < 1.0)
        {
            let jump_speed = jump.speed.to_pixels() * time.delta_seconds();
            jump.velocity = jump_speed;
            jump.current_distance += jump_speed;
        } else if !jump.grounded {
            jump.jumping = false;
            let fall_speed = jump.speed.to_pixels() * time.delta_seconds();
            jump.velocity = -fall_speed;
        } else if jump.grounded {
            jump.jumping = false;
            jump.current_distance = 0.0;
        }
    }
}

#[derive(Reflect, Component, Debug)]
#[reflect(Component)]
pub struct Run {
    pub base_speed: Distance,    // start running at this speed
    pub max_speed: Distance,     // end up running at this speed
    pub time_for_max_speed: f32, // after this many seconds
    pub running: Option<f32>, // set this negative 1 left, positive 1 right, none idle. other values will adjust speed.
    pub current_run_time: f32, // how long you've been running.
    pub velocity: f32,        // this is used to set x movement requested in physics controller
}

impl Default for Run {
    fn default() -> Self {
        Self {
            base_speed: Distance::meters(5.0),
            max_speed: Distance::meters(10.0),
            time_for_max_speed: 2.0,
            running: None,
            current_run_time: 0.0,
            velocity: 0.0,
        }
    }
}

pub fn update_run_component(mut run_query: Query<&mut Run>, time: Res<Time>) {
    for mut run in run_query.iter_mut() {
        if let Some(dir) = run.running {
            run.current_run_time += time.delta_seconds();
            let current_speed = run.base_speed.to_pixels()
                + (run.max_speed.to_pixels() - run.base_speed.to_pixels())
                    * (run.current_run_time / run.time_for_max_speed).min(1.0);
            run.velocity = current_speed * time.delta_seconds() * dir;
        } else {
            run.current_run_time = 0.0;
            run.velocity = 0.0;
        }
    }
}

pub fn update_character_position_from_velocity(
    mut player_query: Query<(&mut KinematicCharacterController, &Jump, &Run), With<PlayerMarker>>,
) {
    if let Ok((mut controller, jump, run)) = player_query.get_single_mut() {
        controller.translation = Some(Vec2::new(run.velocity, jump.velocity));
    }
}
