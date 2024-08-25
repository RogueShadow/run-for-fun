use bevy::prelude::*;

#[derive(Component)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}
impl AnimationIndices {
    pub fn new(first: usize, last: usize) -> Self {
        Self { first, last }
    }
}
#[derive(Component)]
pub struct AnimationAtlas {
    animations: Vec<Animation>,
    current: usize,
}
impl AnimationAtlas {
    pub fn new(animations: impl Into<Vec<Animation>>) -> Self {
        Self {
            animations: animations.into(),
            current: 0,
        }
    }
    pub fn next(&mut self) {
        if let Some(anim) = self.animations.get_mut(self.current) {
            anim.next();
        }
    }
    pub fn current(&self) -> usize {
        if let Some(anim) = self.animations.get(self.current) {
            anim.current()
        } else {
            0
        }
    }
    pub fn set_current(&mut self, index: usize) {
        if (0..self.animations.len()).contains(&index) {
            self.current = index;
        } else {
            error!("Invalid index");
        }
    }
}
impl Default for AnimationAtlas {
    fn default() -> Self {
        Self {
            animations: vec![],
            current: 0,
        }
    }
}

pub struct Animation {
    sprites: Vec<usize>,
    position: usize,
}
impl Animation {
    pub fn new(sprites: impl Into<Vec<usize>>) -> Self {
        Self {
            sprites: sprites.into(),
            position: 0,
        }
    }
    pub fn next(&mut self) {
        self.position += 1;
        if self.position >= self.sprites.len() {
            self.position = 0;
        }
    }
    pub fn current(&self) -> usize {
        self.sprites[self.position]
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct AnimationTimer(pub Timer);

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}
