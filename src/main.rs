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
        .insert_resource(WordSelection {
            built_word: String::new(),
            positions: Vec::new(),
            changed_this_frame: true,
            current_layer: 0,
            target_word: String::new(),
            complete_solution: Vec::new(),
        })
        .insert_resource(RuneTextStyles::default())
        .insert_resource(MousePosition {
            pos: None,
        })
        .add_event::<WordCompleteEvent>()
        .add_systems(Startup, (load_fonts, spawn_camera, spawn_edit_buttons, spawn_circle).chain())
        .add_systems(Update, (update_active_ring, update_mouse_position, select_letters, handle_backspace, handle_reset, check_complete).chain())
        .add_systems(Update, (draw_selection, update_word_display))
        .run();
}

#[derive(Component, Default)]
struct RingLayer {}


#[derive(Resource)]
struct WordSelection {
    built_word: String,
    positions: Vec<Vec2>,
    changed_this_frame: bool,
    current_layer: u32,
    target_word: String,
    complete_solution: Vec<String>,
}

#[derive(Resource, Default)]
struct RuneTextStyles {
    active: TextStyle,
    idle: TextStyle,
    display: TextStyle,
}

#[derive(Resource)]
struct MousePosition {
    pos: Option<Vec2>,
}

#[derive(Component)]
struct WordDisplay {}

#[derive(Component)]
struct LetterDisplay {
    letter: char,
    active: bool,
    position: Vec2,
    radius: f32,
    layer: u32,
}

#[derive(Component)]
struct LayerRing {
    layer: u32,
}

#[derive(Component)]
struct WordLine {
    layer: u32,
}

#[derive(Event)]
struct WordCompleteEvent {
    now_on_layer: u32,
}

#[derive(Component)]
struct BackspaceButton {}

#[derive(Component)]
struct ResetButton {}

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(0., -50., 0.)),
        projection: OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(700.),
            ..default()
        },
        ..default()
    });
}

fn spawn_edit_buttons(
    mut commands: Commands,
    rune_fonts: ResMut<RuneTextStyles>,
) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("<", rune_fonts.display.clone()),
            transform: Transform::from_translation(Vec3::new(-50., -350., 0.)),
            ..default()
        },
        BackspaceButton {},
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("X", rune_fonts.display.clone()),
            transform: Transform::from_translation(Vec3::new(50., -350., 0.)),
            ..default()
        },
        ResetButton {},
    ));
}

fn load_fonts(
    mut rune_fonts: ResMut<RuneTextStyles>,
    asset_server: Res<AssetServer>,
) {
    let active_font = asset_server.load("fonts/Micro5-Regular.ttf");
    rune_fonts.active = TextStyle {
        font: active_font.clone(),
        font_size: 48.,
        color: Color::WHITE,
    };

    let idle_font = asset_server.load("fonts/sga-pixel.ttf");
    rune_fonts.idle = TextStyle {
        font: idle_font,
        font_size: 24.,
        color: Color::GRAY,
    };

    rune_fonts.display = TextStyle {
        font: active_font.clone(),
        font_size: 64.,
        color: Color::WHITE,
    }
}

fn spawn_circle(
    mut commands: Commands,
    font_settings: Res<RuneTextStyles>,
    mut solution: ResMut<WordSelection>,
) {
    spawn_puzzle(&mut commands, &mut solution, 140., 50., &font_settings.active, &font_settings.idle, vec![
        ("damped", 3, 1), ("exorcise", 1, 2), ("subordinates", 8, 3)
    ]);
}

fn spawn_puzzle(
    commands: &mut Commands,
    solution: &mut WordSelection,
    base_radius: f32,
    spacing: f32,
    active_text_style: &TextStyle,
    idle_text_style: &TextStyle,
    rings: Vec<(&str, usize, usize)>,
) {
    let mut cur_radius = base_radius;
    let active = 0;
    let mut index = 0;
    for (word, step, start) in rings.iter() {
        let is_active = active == index;
        spawn_ring(commands, cur_radius, if is_active { active_text_style } else { idle_text_style }, is_active, word, step, start, index);
        cur_radius += spacing;

        solution.complete_solution.push(word.to_string());

        index += 1;
    }

    solution.target_word = solution.complete_solution[0].clone();

    commands.spawn((
        Text2dBundle {
            transform: Transform::from_translation(Vec3::new(0., -300., 0.)),
            ..default()
        },
        WordDisplay {},
    ));
}

fn spawn_ring(
    commands: &mut Commands,
    radius: f32,
    text_style: &TextStyle,
    active: bool,
    word: &str,
    solution_step: &usize,
    solution_start_index: &usize,
    layer: u32,
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
        let offset_direction = Vec2::from_angle(angle);
        let offset = offset_direction * radius;

        let character = if active {
            (shuffled_word[(i + solution_start_index) % length] as char).to_ascii_uppercase()
        }
        else {  
            (shuffled_word[(i + solution_start_index) % length] as char).to_ascii_lowercase()
        };

        commands.spawn((Text2dBundle {
                text: Text::from_section(character, text_style.clone()),
                transform: Transform::from_translation(offset.extend(-0.1)),//.with_rotation(Quat::from_rotation_z(angle + (PI / 2.))),
                ..default()
            },
            LetterDisplay {
                letter: character.to_ascii_uppercase(),
                active,
                position: offset - (offset_direction * 16.),
                radius: 64.,
                layer
            },
        )).set_parent(parent);
    }

    let shape = shapes::Circle {
        radius: radius + 32.,
        center: Vec2::ZERO,
    };

    commands.spawn((
        ShapeBundle {
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., -0.1)),
                ..default()
            },
            path: GeometryBuilder::build_as(&shape),
            ..default()
        },
        Stroke::new(text_style.color, 4.0),
        LayerRing {
            layer
,        }
    )).set_parent(parent);

    commands.spawn((
        ShapeBundle {
            ..default()
        },
        Stroke::new(Color::rgb(1.0, 0.3, 0.3), 4.0),
        WordLine {
            layer,
        }
    )).set_parent(parent);
}

fn update_active_ring(
    mut complete_events: EventReader<WordCompleteEvent>,
    mut selection: ResMut<WordSelection>,
    mut letters: Query<(&mut Text, &mut LetterDisplay)>,
    mut rings: Query<(&mut Stroke, &LayerRing)>,
    text_styles: Res<RuneTextStyles>,
) {
    for completion in complete_events.read() {
        println!("completed! advancing to {}", completion.now_on_layer);
        let new_active = completion.now_on_layer;

        

        for (mut text, mut letter) in letters.iter_mut() {
            let is_active = letter.layer == new_active;    
            text.sections[0].style = if is_active { text_styles.active.clone() } else { text_styles.idle.clone() };

            if is_active {
                text.sections[0].value = text.sections[0].value.to_ascii_uppercase();
            }
            else {
                text.sections[0].value = text.sections[0].value.to_ascii_lowercase();
            }
            letter.active = is_active;
        }

        for (mut stroke, ring) in rings.iter_mut() {
            let is_active = ring.layer == new_active;
            stroke.color = if is_active { Color::WHITE } else { Color::GRAY };
        }

        selection.built_word.clear();
        selection.positions.clear();
        selection.changed_this_frame = true;
        selection.current_layer = new_active;

        if new_active as usize >= selection.complete_solution.len() {
            return;
        }

        selection.target_word = selection.complete_solution[new_active as usize].clone();
        
    }
}

fn update_mouse_position(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos_state: ResMut<MousePosition>,
) {
    if let Some(viewport_position) = windows.single().cursor_position() {
        let (camera, camera_transform) = cameras.single();

        mouse_pos_state.pos = camera.viewport_to_world_2d(camera_transform, viewport_position);
    }
    else {
        mouse_pos_state.pos = None;
    }
}

fn select_letters(
    mut selection: ResMut<WordSelection>,
    letters: Query<(&LetterDisplay, &Transform)>,
    mouse_state: Res<MousePosition>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut character_events: EventReader<ReceivedCharacter>,
) {
    let mut recieved_chars = Vec::new();
    for ev in character_events.read() {
        recieved_chars.push((ev.char.as_bytes()[0] as char).to_ascii_uppercase());
    }
    let recieved_chars = recieved_chars;

    let mut changed_this_frame = false;

    for (letter, transform) in letters.iter() {
        if letter.active {
            let mut mouse_selected = false;

            if mouse_buttons.just_pressed(MouseButton::Left) {
                if let Some(mouse_pos) = mouse_state.pos {
                    if transform.translation.truncate().distance(mouse_pos) < letter.radius {
                        mouse_selected = true;
                    }
                }
            }

            let keyboard_selected = recieved_chars.contains(&letter.letter);
            
            if mouse_selected || keyboard_selected {
                selection.built_word.push(letter.letter);
                selection.positions.push(letter.position);

                changed_this_frame = true;

                println!("selected {}", letter.letter);
            }
        }
    }

    selection.changed_this_frame = changed_this_frame;
}

fn check_complete(
    selection: ResMut<WordSelection>,
    word_list: Res<WordList>,
    mut complete_writer: EventWriter<WordCompleteEvent>,
) {
    let bytes = selection.built_word.as_bytes();

    if selection.built_word.len() >= selection.target_word.len() && bytes[0].to_ascii_uppercase() == bytes[bytes.len() - 1].to_ascii_uppercase() {
        if selection.built_word.to_ascii_uppercase() == selection.target_word.to_ascii_uppercase() {
            complete_writer.send(WordCompleteEvent { now_on_layer: selection.current_layer + 1});
            println!("perfect solve!");
        }
        else if word_list.all_valid_words.contains(&selection.built_word) {
            complete_writer.send(WordCompleteEvent { now_on_layer: selection.current_layer + 1});
            println!("alternate solve!");
        }
    }
}

fn handle_reset(
    mut selection: ResMut<WordSelection>,
    button_query: Query<&Transform, With<ResetButton>>,
    mouse_pos: Res<MousePosition>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    let mut is_clicked = false;

    if let Some(pos) = mouse_pos.pos {
        let button_pos = button_query.single().translation.truncate();
        if mouse_buttons.just_pressed(MouseButton::Left) && pos.distance(button_pos) < 50. {
            is_clicked = true;
        }
    }

    if is_clicked {
        selection.built_word.clear();
        selection.positions.clear();
        selection.changed_this_frame = true;
    }
}

fn handle_backspace(
    mut selection: ResMut<WordSelection>,
    keys: Res<ButtonInput<KeyCode>>,
    button_query: Query<&Transform, With<BackspaceButton>>,
    mouse_pos: Res<MousePosition>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    let mut is_clicked = false;

    if let Some(pos) = mouse_pos.pos {
        let button_pos = button_query.single().translation.truncate();
        if mouse_buttons.just_pressed(MouseButton::Left) && pos.distance(button_pos) < 50. {
            is_clicked = true;
        }
    }

    if is_clicked || keys.just_pressed(KeyCode::Backspace) {
        if selection.built_word.len() > 0 {
            selection.built_word.pop();
            selection.positions.pop();
            selection.changed_this_frame = true;
        }
    }
}

fn draw_selection(
    selection: Res<WordSelection>,
    mut line_query: Query<(&WordLine, &mut Path)>
) {
    for (word_line, mut path) in line_query.iter_mut() {
        if word_line.layer == selection.current_layer && selection.changed_this_frame {
            let mut path_builder = PathBuilder::new();
            if selection.positions.len() > 1 {
                path_builder.move_to(selection.positions[0]);

                for i in 1..selection.positions.len() {
                    path_builder.line_to(selection.positions[i]);
                }
            }
            // path_builder.close();

            *path = path_builder.build();
        }
    }
}

fn update_word_display(
    selection: Res<WordSelection>,
    mut display_query: Query<&mut Text, With<WordDisplay>>,
    rune_fonts: ResMut<RuneTextStyles>,
) {
    *(display_query.single_mut()) = Text::from_section(selection.built_word.clone(), rune_fonts.display.clone());
}
