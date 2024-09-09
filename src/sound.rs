use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

#[derive(Event)]
pub enum Play {
    Jump,
    Walk,
    Finish,
    Land,
    Start,
}

#[derive(Resource)]
pub struct Sounds {
    pub bgm: Handle<AudioSource>,
    pub jump: Handle<AudioSource>,
    pub walk: Handle<AudioSource>,
    pub finish: Handle<AudioSource>,
    pub start: Handle<AudioSource>,
    pub land: Handle<AudioSource>,
}

pub fn play_sounds(trigger: Trigger<Play>, sfx: Res<AudioChannel<MainTrack>>, sounds: Res<Sounds>) {
    let source = match trigger.event() {
        Play::Jump => sounds.jump.clone_weak(),
        Play::Walk => sounds.walk.clone_weak(),
        Play::Finish => sounds.finish.clone_weak(),
        Play::Land => sounds.land.clone_weak(),
        Play::Start => sounds.start.clone_weak(),
    };

    sfx.play(source);
}
