use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::{
    default, Added, Bundle, Component, Query, Text, Text2dBundle, TextStyle, Transform,
};
use bevy_ecs_ldtk::prelude::LdtkFields;
use bevy_ecs_ldtk::{EntityInstance, LdtkEntity};

#[derive(Component, Default)]
pub struct WorldMessage;
#[derive(LdtkEntity, Bundle, Default)]
pub struct WorldMessageBundle {
    world_message: WorldMessage,
    #[with(world_text)]
    text: Text2dBundle,
}
pub fn spawn_world_message(mut messages: Query<&mut Transform, Added<WorldMessage>>) {
    for mut transform in messages.iter_mut() {
        transform.scale = Vec3::splat(0.1);
    }
}
fn world_text(entity_instance: &EntityInstance) -> Text2dBundle {
    let msg = entity_instance
        .get_string_field("message")
        .expect("There should be a message field");
    Text2dBundle {
        text: Text::from_section(
            msg,
            TextStyle {
                color: Color::srgb(0.0, 0.0, 0.0),
                font_size: 64.0,
                ..default()
            },
        ),
        ..default()
    }
}
