use bevy::asset::Handle;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::assets::LdtkProject;
use bevy_kira_audio::AudioSource;

#[derive(AssetCollection, Resource)]
pub struct Sounds {
    #[asset(path = "Caketown 1.mp3")]
    pub bgm: Handle<AudioSource>,
    #[asset(path = "jump_01.wav")]
    pub jump: Handle<AudioSource>,
    #[asset(path = "03_Step_grass_03.wav")]
    pub walk: Handle<AudioSource>,
    #[asset(path = "Won!.wav")]
    pub finish: Handle<AudioSource>,
    #[asset(path = "Start_Sounds_003.wav")]
    pub start: Handle<AudioSource>,
    #[asset(path = "45_Landing_01.wav")]
    pub land: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct Levels {
    #[asset(path = "run_level.ldtk")]
    pub level1: Handle<LdtkProject>,
}
