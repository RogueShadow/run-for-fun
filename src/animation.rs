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
        if animation.just_finished() {
            atlas.index = animation.current();
        }
    }
}
pub fn update_rustanimationatlas(
    time: Res<Time>,
    mut query: Query<(&mut RustAnimationAtlas, &mut TextureAtlas)>,
) {
    for (mut animation, mut atlas) in &mut query {
        animation.tick(time.delta());
        if animation.just_finished() {
            atlas.index = animation.current();
        }
    }
}

#[derive(Component)]
pub struct RustAnimation {
    animation_type: RustAnimationType,
    time: Duration,
    step: Duration,
    just_finished: bool,
}
impl RustAnimation {
    pub fn new(animation_type: RustAnimationType, step: f32) -> Self {
        Self {
            animation_type,
            time: Duration::default(),
            step: Duration::from_secs_f32(step),
            just_finished: false,
        }
    }
    pub fn range(start: usize, end: usize, step: f32) -> Self {
        let animation_type = RustAnimationType::range(start, end);
        Self::new(animation_type, step)
    }
    pub fn list(value: impl Into<Vec<usize>>, step: f32) -> Self {
        let animation_type = RustAnimationType::list(value);
        Self::new(animation_type, step)
    }
    pub fn tick(&mut self, duration: Duration) {
        self.time += duration;
        if self.time >= self.step {
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
}
pub enum RustAnimationType {
    IndexList {
        indices: Vec<usize>,
        position: usize,
    },
    IndexRange {
        start: usize,
        end: usize,
        position: usize,
    },
}
impl RustAnimationType {
    pub fn range(start: usize, end: usize) -> Self {
        Self::IndexRange {
            start,
            end,
            position: start,
        }
    }
    pub fn list(value: impl Into<Vec<usize>>) -> Self {
        let value = value.into();
        if value.is_empty() {
            panic!("List must not be empty.")
        }
        let first = value[0];
        Self::IndexList {
            indices: value,
            position: first,
        }
    }
    pub fn next(&mut self) {
        match self {
            Self::IndexList { indices, position } => {
                *position += 1;
                if *position >= indices.len() {
                    *position = 0;
                }
            }
            Self::IndexRange {
                start,
                end,
                position,
            } => {
                *position += 1;
                if position >= end {
                    *position = *start
                }
            }
        }
    }
    pub fn current(&self) -> usize {
        match self {
            RustAnimationType::IndexList { indices, position } => indices[*position],
            RustAnimationType::IndexRange { position, .. } => *position,
        }
    }
}

pub struct RustAnimationPlugin;

impl Plugin for RustAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_rustanimation, update_rustanimationatlas));
    }
}
