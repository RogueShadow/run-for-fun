use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::distributions::Slice;
use std::time::Duration;

#[derive(Component)]
pub struct RustAnimationAtlas {
    animations: Vec<RustAnimation>,
    current: usize,
}
impl RustAnimationAtlas {
    pub fn new(animations: impl Into<Vec<RustAnimation>>) -> Self {
        Self {
            animations: animations.into(),
            current: 0,
        }
    }
    pub fn current(&self) -> usize {
        if let Some(anim) = self.animations.get(self.current) {
            anim.current()
        } else {
            panic!("Animation index out of bounds in atlas.")
        }
    }
    pub fn set_current(&mut self, index: usize) {
        if (0..self.animations.len()).contains(&index) {
            self.current = index;
        } else {
            error!("Invalid index");
        }
    }
    pub fn tick(&mut self, delta: Duration) {
        if self.animations.is_empty() {
            return;
        }
        self.animations[self.current].tick(delta);
    }
    pub fn just_finished(&self) -> bool {
        if self.animations.is_empty() {
            return false;
        };
        self.animations[self.current].just_finished()
    }
}
impl Default for RustAnimationAtlas {
    fn default() -> Self {
        Self {
            animations: vec![],
            current: 0,
        }
    }
}

pub fn update_rustanimation(
    time: Res<Time>,
    mut query: Query<(&mut RustAnimation, &mut TextureAtlas)>,
) {
    for (mut animation, mut atlas) in &mut query {
        animation.tick(time.delta());
        atlas.index = animation.current();
    }
}
pub fn update_rustanimationatlas(
    time: Res<Time>,
    mut query: Query<(&mut RustAnimationAtlas, &mut TextureAtlas)>,
) {
    for (mut animation, mut atlas) in &mut query {
        animation.tick(time.delta());
        atlas.index = animation.current();
    }
}

#[derive(Component)]
pub struct RustAnimation {
    animation_type: RustAnimationType,
    time: Duration,
    just_finished: bool,
}
impl RustAnimation {
    pub fn new(animation_type: RustAnimationType) -> Self {
        Self {
            animation_type,
            time: Duration::default(),
            just_finished: false,
        }
    }
    fn variable_timing_list(value: impl Into<Vec<usize>>, timing: impl Into<Vec<f32>>) -> Self {
        Self::new(RustAnimationType::variable_timing_list(value, timing))
    }
    fn variable_timing_range(start: usize, end: usize, timing: impl Into<Vec<f32>>) -> Self {
        Self::new(RustAnimationType::variable_timing_range(start, end, timing))
    }
    pub fn range(start: usize, end: usize, step: f32) -> Self {
        let animation_type = RustAnimationType::range(start, end, step);
        Self::new(animation_type)
    }
    pub fn list(value: impl Into<Vec<usize>>, step: f32) -> Self {
        let animation_type = RustAnimationType::list(value, step);
        Self::new(animation_type)
    }
    pub fn with_timings(self, timing: impl Into<Vec<f32>>) -> Self {
        match self.animation_type {
            RustAnimationType::IndexList { indices, .. } => {
                Self::variable_timing_list(indices, timing)
            }
            RustAnimationType::VariableTimingList { indices, .. } => {
                Self::variable_timing_list(indices, timing)
            }
        }
    }
    pub fn tick(&mut self, duration: Duration) {
        self.time += duration;
        if self.time >= self.step() {
            self.time = Duration::default();
            self.animation_type.next();
            self.just_finished = true;
        } else {
            self.just_finished = false;
        }
    }
    pub fn current(&self) -> usize {
        self.animation_type.current()
    }
    pub fn just_finished(&self) -> bool {
        self.just_finished
    }
    pub fn step(&self) -> Duration {
        match &self.animation_type {
            RustAnimationType::IndexList { step, .. } => *step,
            RustAnimationType::VariableTimingList {
                timing, position, ..
            } => timing[*position],
        }
    }
}
#[derive(Debug)]
pub enum RustAnimationType {
    IndexList {
        indices: Vec<usize>,
        position: usize,
        step: Duration,
    },
    VariableTimingList {
        indices: Vec<usize>,
        timing: Vec<Duration>,
        position: usize,
    },
}
impl RustAnimationType {
    // constructors
    pub fn variable_timing_list(value: impl Into<Vec<usize>>, timing: impl Into<Vec<f32>>) -> Self {
        let value = value.into();
        let timing = timing.into();

        if value.is_empty() {
            panic!("List must not be empty.")
        }
        if value.len() != timing.len() {
            panic!("Must have same number of indices and timings")
        }
        let first = value[0];
        Self::VariableTimingList {
            indices: value,
            position: first,
            timing: timing
                .iter()
                .map(|t| Duration::from_secs_f32(*t))
                .collect::<Vec<_>>(),
        }
    }
    pub fn variable_timing_range(start: usize, end: usize, timing: impl Into<Vec<f32>>) -> Self {
        Self::variable_timing_list((start..=end).collect::<Vec<_>>(), timing)
    }
    pub fn range(start: usize, end: usize, step: f32) -> Self {
        Self::list((start..=end).collect::<Vec<_>>(), step)
    }
    pub fn list(value: impl Into<Vec<usize>>, step: f32) -> Self {
        let value = value.into();
        if value.is_empty() {
            panic!("List must not be empty.")
        }
        let first = value[0];
        Self::IndexList {
            indices: value,
            position: first,
            step: Duration::from_secs_f32(step),
        }
    }
    pub fn next(&mut self) {
        match self {
            Self::IndexList {
                indices, position, ..
            } => {
                if *position + 1 >= indices.len() {
                    *position = 0;
                } else {
                    *position += 1
                }
            }
            RustAnimationType::VariableTimingList {
                indices, position, ..
            } => {
                if *position + 1 >= indices.len() {
                    *position = 0;
                } else {
                    *position += 1
                }
            }
        }
    }
    pub fn current(&self) -> usize {
        match self {
            RustAnimationType::IndexList {
                indices, position, ..
            } => {
                if let Some(index) = indices.get(*position) {
                    *index
                } else {
                    warn!("Index was out of bounds for {:?}", self);
                    indices[0]
                }
            }
            RustAnimationType::VariableTimingList {
                indices, position, ..
            } => {
                if let Some(index) = indices.get(*position) {
                    *index
                } else {
                    warn!("Index was out of bounds for {:?}", self);
                    indices[0]
                }
            }
        }
    }
}

pub struct RustAnimationPlugin;

impl Plugin for RustAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_rustanimation,
                update_rustanimationatlas,
                update_tween_sprite,
            ),
        );
    }
}

pub fn linear_path<T: Interpolation<T>>(values: &[T], time: f32) -> T {
    if values.len() < 2 {
        panic!("Must have at least 2 values");
    }
    let steps = values.len() as f32 - 1.0;
    let step_size = 1.0 / steps;
    let index = (time / step_size).floor() as usize;
    let start_time = index as f32 * step_size;
    let current_time = time - start_time;
    let t = current_time / step_size;
    values[index].linear(&values[index + 1], t)
}
pub fn spline_path<T: Interpolation<T>>(values: &[T], time: f32) -> T {
    if values.len() < 2 {
        panic!("Must have at least 2 values");
    }
    let steps = values.len() as f32 - 1.0;
    let step_size = 1.0 / steps;
    let index = (time / step_size).floor() as usize;
    let start_time = index as f32 * step_size;
    let current_time = time - start_time;
    let t = current_time / step_size;
    values[index].spline(&values[index + 1], 0.0, 1.0, t)
}
pub trait Interpolation<T> {
    fn linear(&self, next: &T, time: f32) -> T;
    fn spline(&self, next: &T, m0: f32, m1: f32, time: f32) -> T;
}
impl Interpolation<f32> for f32 {
    fn linear(&self, next: &f32, time: f32) -> f32 {
        match time {
            time if time <= 0.0 => self.to_owned(),
            time if time >= 1.0 => next.to_owned(),
            time => (1.0 - time) * self + time * next,
        }
    }
    fn spline(&self, next: &f32, m0: f32, m1: f32, time: f32) -> f32 {
        match time {
            time if time <= 0.0 => self.to_owned(),
            time if time >= 1.0 => next.to_owned(),
            time => {
                let t3 = time.powi(3);
                let t2 = time.powi(2);
                let t = time;
                (2.0 * t3 - 3.0 * t2 + 1.0) * self
                    + (t3 - 2.0 * t2 + t) * m0
                    + (-2.0 * t3 + 3.0 * t2) * next
                    + (t3 - t2) * m1
            }
        }
    }
}
impl Interpolation<Vec2> for Vec2 {
    fn linear(&self, next: &Vec2, time: f32) -> Vec2 {
        Vec2::new(self.x.linear(&next.x, time), self.y.linear(&next.y, time))
    }
    fn spline(&self, next: &Vec2, m0: f32, m1: f32, time: f32) -> Vec2 {
        Vec2::new(
            self.x.spline(&next.x, m0, m1, time),
            self.y.spline(&next.y, m0, m1, time),
        )
    }
}

#[derive(Component)]
pub struct TweenSprite {
    start: f32,
    length: f32,
    path: Vec<Vec2>,
    finished: bool,
}
impl TweenSprite {
    pub fn new(path: impl Into<Vec<Vec2>>, start: f32, length: f32) -> Self {
        let path = path.into();
        if path.len() < 2 {
            panic!("Must have at least 2 points in path.")
        }
        Self {
            start,
            length,
            path,
            finished: false,
        }
    }
    pub fn linear(&mut self, elapsed: f32) -> Vec2 {
        let time = elapsed - self.start;
        let time = time / self.length;
        if time < 0.0 {
            return self.path[0];
        }
        if time > 1.0 {
            self.finished = true;
            return *self.path.last().unwrap();
        }
        linear_path(self.path.as_slice(), time)
    }
    pub fn spline(&mut self, elapsed: f32) -> Vec2 {
        let time = elapsed - self.start;
        let time = time / self.length;
        if time < 0.0 {
            return self.path[0];
        }
        if time > 1.0 {
            self.finished = true;
            return *self.path.last().unwrap();
        }
        spline_path(self.path.as_slice(), time)
    }
    pub fn finished(&self) -> bool {
        self.finished
    }
}

pub fn update_tween_sprite(mut query: Query<(&mut Sprite, &mut TweenSprite)>, time: Res<Time>) {
    for (mut sprite, mut tween) in query.iter_mut() {
        let pos = tween.linear(time.elapsed_seconds());
        sprite.anchor = Anchor::Custom(pos);
        if tween.finished() {
            tween.start = time.elapsed_seconds() + 2.0;
            tween.finished = false;
        }
    }
}
