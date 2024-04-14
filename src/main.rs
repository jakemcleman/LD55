use std::f32::consts::PI;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_prototype_lyon::prelude::*;

use crate::worldlist::*;

mod worldlist;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Spell Circle".to_string(), // ToDo
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy".to_owned()),
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ShapePlugin)
        .insert_resource(WordList::default())
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_circle)
        .run();
}

#[derive(Component, Default)]
struct RingLayer {}

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(960.),
            ..default()
        },
        ..default()
    });
}

fn spawn_circle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/Micro5-Regular.ttf");
    let text_style = TextStyle {
        font,
        font_size: 64.,
        color: Color::WHITE,
    };

    spawn_puzzle(&mut commands, 200., 60., &text_style, vec![
        ("damped", 3, 1), ("exorcise", 1, 2), ("subordinates", 8, 3)
    ]);
}

fn spawn_puzzle(
    commands: &mut Commands,
    base_radius: f32,
    spacing: f32,
    text_style: &TextStyle,
    rings: Vec<(&str, usize, usize)>,
) {
    let mut cur_radius = base_radius;
    for (word, step, start) in rings.iter() {
        spawn_ring(commands, cur_radius, &text_style, word, step, start);
        cur_radius += spacing;
    }
}

fn spawn_ring(
    commands: &mut Commands,
    radius: f32,
    text_style: &TextStyle,
    word: &str,
    solution_step: &usize,
    solution_start_index: &usize,
) {
    let parent = commands.spawn((TransformBundle::default(), RingLayer::default(), InheritedVisibility::default())).id();
    let length = word.len() - 1;

    let mut shuffled_word = vec![0; length];

    for i in 0..length {
        let wrap_index = i * solution_step;
        let shuffled_index = wrap_index % length;
        shuffled_word[shuffled_index] = word.as_bytes()[i];
    }

    for i in 0..length {
        let angle = i as f32 * (2. * PI / length as f32);
        let offset = Vec2::from_angle(angle) * radius;

        commands.spawn(Text2dBundle {
            text: Text::from_section((shuffled_word[(i + solution_start_index) % length] as char).to_ascii_uppercase(), text_style.clone()),
            transform: Transform::from_translation(offset.extend(0.)),//.with_rotation(Quat::from_rotation_z(angle + (PI / 2.))),
            ..default()
        }).set_parent(parent);
    }

    let shape = shapes::Circle {
        radius: radius + 32.,
        center: Vec2::ZERO,
    };

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..default()
        },
        Stroke::new(Color::WHITE, 4.0),
    )).set_parent(parent);
}
