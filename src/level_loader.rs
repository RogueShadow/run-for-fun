use crate::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_ldtk::ldtk::{loaded_level::LoadedLevel, TileInstance};
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use entities::{crates::*, flags::*, message::*, player::*};
use std::time::Duration;

pub struct RFFLevelPlugin;
impl Plugin for RFFLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin);
        app.insert_resource(LevelSelection::Uid(0));
        app.register_ldtk_entity::<PlayerBundle>("Player");
        app.register_ldtk_entity::<Crate>("Crate");
        app.register_ldtk_entity::<StartFlag>("Start");
        app.register_ldtk_entity::<FinishFlag>("Finish");
        app.register_ldtk_entity::<WorldMessageBundle>("WorldMessage");
        app.add_systems(
            Update,
            (
                dynamic_collision_layer_building,
                spawn_player,
                spawn_flags,
                spawn_world_message,
            )
                .run_if(in_state(GameState::LoadGame)),
        );
    }
}

pub struct TiledCollisionBuilder {
    building: bool,
    position: Vec2,
    size: Vec2,
    height: f32,
    tile_size: Vec2,
}
impl Default for TiledCollisionBuilder {
    fn default() -> Self {
        Self {
            size: Vec2::ZERO,
            building: false,
            height: 32.0,
            position: Vec2::ZERO,
            tile_size: Vec2::new(16.0, 16.0),
        }
    }
}
impl TiledCollisionBuilder {
    pub fn build(&mut self) -> (SpatialBundle, Collider, RigidBody) {
        let components = (
            SpatialBundle {
                transform: Transform::from_xyz(
                    self.position.x + (self.size.x / 2.0) - self.tile_size.x / 2.0,
                    self.position.y - (self.height - self.tile_size.y) / 2.0,
                    0.0,
                ),
                ..default()
            },
            Collider::cuboid(self.size.x / 2.0, self.height / 2.0),
            RigidBody::Fixed,
        );
        self.reset();
        components
    }
    pub fn begin(&mut self, pos: Vec2, tile_size: Vec2, height: f32) {
        self.size = tile_size;
        self.position = pos;
        self.tile_size = tile_size;
        self.height = height;
        self.building = true;
    }
    pub fn reset(&mut self) {
        let b = Self::default();
        self.building = b.building;
        self.tile_size = b.tile_size;
        self.size = b.size;
        self.height = b.height;
        self.position = b.position;
    }
    pub fn extend(&mut self) {
        if !self.building {
            return;
        }
        self.size.x += self.tile_size.x;
    }
}

pub fn dynamic_collision_layer_building(
    mut cmds: Commands,
    q_tile: Query<(Entity, &TileStorage, &TilemapGridSize, &LayerMetadata), Added<TileStorage>>,
    data: Query<&TileMetadata>,
    map_assets: Res<Assets<LdtkProject>>,
    level_data: Res<Levels>,
) {
    if let Some(map) = map_assets.get(&level_data.level1) {
        if let Some((level_entity, tile_storage, tilemap_gridsize, _)) = q_tile
            .iter()
            .filter(|(_, _, _, l)| l.identifier == "Tiles")
            .next()
        {
            let tile_size = vec2(tilemap_gridsize.x, tilemap_gridsize.y);
            let level_size = (tile_storage.size.x, tile_storage.size.y);
            let mut cells: HashMap<(i32, i32), f32> = HashMap::new();
            for y in 0..level_size.1 {
                for x in 0..level_size.0 {
                    let tile_position = TilePos::new(x, y);
                    if let Some(tile) = tile_storage.checked_get(&tile_position) {
                        if let Ok(tile_metadata) = data.get(tile) {
                            match tile_metadata.data.as_str() {
                                "collider" => {
                                    cells.insert((x as i32, y as i32), tile_size.y);
                                }
                                "half_collider" => {
                                    cells.insert((x as i32, y as i32), tile_size.y * 0.5);
                                }
                                "quarter_collider" => {
                                    cells.insert((x as i32, y as i32), tile_size.y * 0.25);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            let mut builder = TiledCollisionBuilder::default();

            for y in 0..level_size.1 as i32 {
                for x in 0..level_size.0 as i32 {
                    let pos = vec2(x as f32 * tile_size.x, y as f32 * tile_size.y);
                    let cell = cells.get(&(x, y));
                    match (cell.is_some(), builder.building) {
                        (true, false) => {
                            builder.begin(pos, tile_size, *cell.unwrap()); //We checked if it's some first.
                        }
                        (true, true) => {
                            builder.extend();
                        }
                        (false, false) => {}
                        (false, true) => {
                            cmds.spawn(builder.build()).set_parent(level_entity);
                        }
                    }
                }
                if builder.building {
                    cmds.spawn(builder.build()).set_parent(level_entity);
                }
            }
            cmds.trigger(StartBackgroundMusic);
            let data = level_data.level1.path();
            info!("Finished Building Colliders for level. {:?}", data);
        }
    }
}
