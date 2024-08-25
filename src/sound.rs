use bevy::audio::PlaybackMode;
use bevy::prelude::*;

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
    pub jump: Handle<AudioSource>,
    pub walk: Handle<AudioSource>,
    pub finish: Handle<AudioSource>,
    pub start: Handle<AudioSource>,
    pub land: Handle<AudioSource>,
}

pub fn play_sounds(trigger: Trigger<Play>, mut commands: Commands, sounds: Res<Sounds>) {
    let source = match trigger.event() {
        Play::Jump => sounds.jump.clone_weak(),
        Play::Walk => sounds.walk.clone_weak(),
        Play::Finish => sounds.finish.clone_weak(),
        Play::Land => sounds.land.clone_weak(),
        Play::Start => sounds.start.clone_weak(),
    };
    commands.spawn(AudioSourceBundle {
        source,
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    });
}
