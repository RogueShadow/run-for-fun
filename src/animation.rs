use bevy::prelude::*;
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
        app.add_systems(Update, (update_rustanimation, update_rustanimationatlas));
    }
}
