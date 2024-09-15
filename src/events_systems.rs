use crate::assets::Sounds;
use crate::entities::player::PlayerMarker;
use crate::PlayerText;
use crate::RaceTime;
use crate::{BackgroundMusic, SoundEffects};
use bevy::prelude::*;
use bevy_ecs_ldtk::LevelSelection;
use bevy_kira_audio::prelude::*;

pub struct EventsPlugin;
impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.observe(play_sounds);
        app.observe(player_touched_flags);
        app.observe(start_background_music);
    }
}

#[derive(Event)]
pub struct SpawnMessageEvent {
    pub message: String,
    pub position: Vec2,
}

#[derive(Event)]
pub enum TouchedFlag {
    Start,
    Finish,
}

pub fn player_touched_flags(
    trigger: Trigger<TouchedFlag>,
    mut text_query: Query<&mut Text, With<PlayerText>>,
    mut commands: Commands,
    mut race_time_query: Query<&mut RaceTime>,
    player: Query<Entity, With<PlayerMarker>>,
    level_selection: Res<LevelSelection>,
) {
    let player_entity = player.single();
    let mut msg = |msg: &str| {
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = msg.into();
        }
    };
    match trigger.event() {
        TouchedFlag::Start => {
            if let Ok(time) = race_time_query.get_single_mut() {
                if time.level != *level_selection {
                    msg("One race at a time fella!");
                } else {
                    msg("You've already started, why you back here?!");
                }
            } else {
                commands.trigger(PlaySoundEffect::Start);
                msg("Run to the finish line!");
                commands.entity(player_entity).insert(RaceTime {
                    time: Time::default(),
                    level: level_selection.clone(),
                });
            }
        }
        TouchedFlag::Finish => {
            if let Ok(time) = race_time_query.get_single_mut() {
                if time.level == *level_selection {
                    commands.trigger(PlaySoundEffect::Finish);
                    msg(&format!(
                        "You've finished! {:.3}",
                        time.time.elapsed_seconds()
                    ));
                    commands.entity(player_entity).remove::<RaceTime>();
                } else {
                    msg("Wrong flag silly goose.");
                }
            }
        }
    }
}

#[derive(Event)]
pub struct StartBackgroundMusic;

pub fn start_background_music(
    _: Trigger<StartBackgroundMusic>,
    sounds: Res<Sounds>,
    audio: Res<AudioChannel<BackgroundMusic>>,
) {
    audio
        .play(sounds.bgm.clone_weak())
        .looped()
        .with_volume(0.25);
}

#[derive(Event)]
pub enum PlaySoundEffect {
    Jump,
    Walk,
    Finish,
    Land,
    Start,
}

pub fn play_sounds(
    trigger: Trigger<PlaySoundEffect>,
    sfx: Res<AudioChannel<SoundEffects>>,
    sounds: Res<Sounds>,
) {
    let source = match trigger.event() {
        PlaySoundEffect::Jump => sounds.jump.clone_weak(),
        PlaySoundEffect::Walk => sounds.walk.clone_weak(),
        PlaySoundEffect::Finish => sounds.finish.clone_weak(),
        PlaySoundEffect::Land => sounds.land.clone_weak(),
        PlaySoundEffect::Start => sounds.start.clone_weak(),
    };
    sfx.play(source);
}
