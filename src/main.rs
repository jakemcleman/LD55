use std::f32::consts::PI;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_prototype_lyon::prelude::*;

use crate::worldlist::*;
use crate::squashes::*;

mod worldlist;
mod squashes;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Summoners Word".to_string(), // ToDo
                    // Bind to canvas included in `index.html`
                    canvas: Some("#bevy".to_owned()),
                    // Tells wasm not to override default event handling, like F5 and Ctrl+R
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }).set(ImagePlugin::default_nearest()))
        .add_plugins(ShapePlugin)
        .insert_resource(WordList::default())
        .insert_resource(WordSelection {
            built_word: String::new(),
            positions: Vec::new(),
            changed_this_frame: true,
            current_layer: 0,
            current_layer_start_time: 0.,
            target_word: String::new(),
            complete_solution: Vec::new(),
        })
        .insert_resource(RuneTextStyles::default())
        .insert_resource(DemonArts::default())
        .insert_resource(MousePosition {
            pos: None,
            just_clicked: false,
        })
        .insert_resource(PuzzlesList {
            list: vec![
                (0, vec![("mayhem", 3, 1)]),
                (0, vec![("entice", 4, 4)]),
                (2, vec![("grasping", 3, 4)]),
                (3, vec![("neuron", 4, 2), ("scraps", 2, 3)]),
                (0, vec![("lethal", 3, 2), ("rocker", 1, 3)]),
                (2, vec![("tyrant", 3, 5), ("turncoat", 2, 3)]),
                (0, vec![("expire", 2, 2), ("lawful", 4, 3), ("gaming", 3, 3)]),
                (3, vec![("threat", 3, 3), ("divulged", 1, 3)]),
                (0, vec![("dashed", 3, 0), ("dunked", 1, 3), ("dumped", 3, 3)]),
                (0, vec![("medium", 1, 4), ("eulogize", 4, 1)]),
                (3, vec![("edible", 4, 3), ("snacks", 2, 3), ("spoils", 3, 1)]),
                (1, vec![("damped", 3, 1), ("exorcise", 1, 2)]),
                (0, vec![("cosmic", 2, 1), ("sundries", 3, 6)]),
                (2, vec![("cleric", 2, 2), ("shirks", 4, 3), ("damned", 3, 3)]),
                (0, vec![("dismayed", 4, 6), ("catholic", 1, 2)]),
                (0, vec![("thrust", 4, 2), ("rapier", 1, 3), ("withdrew", 4, 3)]),
                (2, vec![("teapot", 1, 2), ("subordinates", 5, 3)]),
                (0, vec![("cosmetic", 2, 6), ("ghosting", 4, 3)]),
                (3, vec![("expose", 2, 0), ("gaping", 4, 1), ("shreds", 1, 3)]),
                (4, vec![("gyrating", 2, 5), ("spaceflights", 8, 7)]),
            ],
            current: 0,
        })
        .add_event::<WordCompleteEvent>()
        .add_event::<PuzzleCompleteEvent>()
        .add_systems(Startup, (load_fonts, load_demons, spawn_camera, spawn_edit_buttons, spawn_circle).chain())
        .add_systems(Update, (update_active_ring, update_mouse_position, select_letters, handle_backspace, handle_reset, handle_next_level, check_complete, spawn_next_level).chain())
        .add_systems(Update, (draw_selection, update_word_display, animate_demon, spin_rings))
        .add_systems(Update, squish_effects)
        .run();
}

#[derive(Component, Default)]
struct RingLayer {
    layer: u32,
}

#[derive(Component, Default)]
struct LevelObject {}

#[derive(Resource)]
struct PuzzlesList {
    list: Vec<(usize, Vec<(&'static str, usize, usize)>)>,
    current: usize,
}

#[derive(Resource)]
struct WordSelection {
    built_word: String,
    positions: Vec<Vec2>,
    changed_this_frame: bool,
    current_layer: u32,
    current_layer_start_time: f32,
    target_word: String,
    complete_solution: Vec<String>,
}

#[derive(Resource, Default)]
struct RuneTextStyles {
    active: TextStyle,
    idle: TextStyle,
    display: TextStyle,
}

#[derive(Resource, Default)]
struct DemonArts {
    sprites: Vec<Handle<Image>>,
}

#[derive(Resource)]
struct MousePosition {
    pos: Option<Vec2>,
    just_clicked: bool,
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

#[derive(Event)]
struct PuzzleCompleteEvent {
    
}

#[derive(Component, Default)]
struct BackspaceButton {
    active: bool
}

#[derive(Component, Default)]
struct ResetButton {
    active: bool
}

#[derive(Component, Default)]
struct NextLevelButton {
    active: bool
}

#[derive(Component)]
struct DemonFace {
    base_scale: f32,
    fill_scale: f32,
    transition_time: f32,
}

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
        BackspaceButton {
            active: true,
        },
        SquishEffect::new(Vec3::ONE, Vec3::splat(2.), 0.01, 0., 0.25),
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("X", rune_fonts.display.clone()),
            transform: Transform::from_translation(Vec3::new(50., -350., 0.)),
            ..default()
        },
        ResetButton {
            active: true,
        },
        SquishEffect::new(Vec3::ONE, Vec3::splat(2.), 0.01, 0., 0.25),
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("NEXT", rune_fonts.display.clone()),
            transform: Transform::from_translation(Vec3::new(0., -350., 0.)),
            visibility: Visibility::Hidden,
            ..default()
        },
        NextLevelButton {
            active: false,
        },
    ));
}

fn load_demons(
    mut demon_art: ResMut<DemonArts>,
    asset_server: Res<AssetServer>,
) {
    demon_art.sprites.push(asset_server.load("sprites/demon01.png"));
    demon_art.sprites.push(asset_server.load("sprites/demon02.png"));
    demon_art.sprites.push(asset_server.load("sprites/demon03.png"));
    demon_art.sprites.push(asset_server.load("sprites/demon04.png"));
    demon_art.sprites.push(asset_server.load("sprites/demon05.png"));

    println!("{} demonic arts loaded", demon_art.sprites.len());
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
        color: Color::DARK_GRAY,
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
    demons: Res<DemonArts>,
    time: Res<Time>,
    puzzles_list: Res<PuzzlesList>,
) {
    spawn_level(puzzles_list.current, &puzzles_list,&mut commands,  &font_settings, &mut solution, &demons, &time);
}

fn spawn_next_level(
    mut commands: Commands,
    font_settings: Res<RuneTextStyles>,
    mut solution: ResMut<WordSelection>,
    demons: Res<DemonArts>,
    time: Res<Time>,
    mut puzzles_list: ResMut<PuzzlesList>,
    mut puzzle_completion_reader: EventReader<PuzzleCompleteEvent>,
    mut world_completion_writer: EventWriter<WordCompleteEvent>,
    clear_entities: Query<Entity, With<LevelObject>>,
) {
    for _event in puzzle_completion_reader.read() {
        for entity in clear_entities.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        puzzles_list.current = (puzzles_list.current + 1) % puzzles_list.list.len();

        spawn_level(puzzles_list.current, &puzzles_list,&mut commands,  &font_settings, &mut solution, &demons, &time);

        world_completion_writer.send(WordCompleteEvent{now_on_layer: 0});
    }
}

fn spawn_level(
    index: usize,
    puzzles_list: &PuzzlesList,
    commands: &mut Commands,
    font_settings: &RuneTextStyles,
    solution: &mut WordSelection,
    demons: &DemonArts,
    time: &Time,
) {
    let (demon_choice, rings) = &puzzles_list.list[index];

    solution.complete_solution.clear();

    let demon = &demons.sprites[*demon_choice];
    spawn_puzzle(commands, solution, 140., 50., &font_settings.active, &font_settings.idle, 
        rings,
        demon.clone()
    );

    solution.current_layer_start_time = time.elapsed_seconds();
    solution.built_word.clear();
    solution.positions.clear();
    solution.changed_this_frame = true;
    solution.current_layer = 0;
    solution.target_word = solution.complete_solution[0].clone();
}

fn spawn_puzzle(
    commands: &mut Commands,
    solution: &mut WordSelection,
    base_radius: f32,
    spacing: f32,
    active_text_style: &TextStyle,
    idle_text_style: &TextStyle,
    rings: &Vec<(&str, usize, usize)>,
    demon: Handle<Image>,
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
        LevelObject {},
    ));

    commands.spawn((
        SpriteBundle {
            texture: demon,
            transform: Transform::from_translation(Vec3::new(0., -0., -0.1)).with_scale(Vec3::new(4., 4., 4.)),
            visibility: Visibility::Hidden,
            ..default()
        },
        DemonFace {
            base_scale: 4.,
            fill_scale: 350.,
            transition_time: 1.,
        },
        LevelObject {}
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
    let parent = commands.spawn((TransformBundle::default(), RingLayer { layer }, InheritedVisibility::default(), LevelObject {})).id();
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
                transform: Transform::from_translation(offset.extend(-0.2)),//.with_rotation(Quat::from_rotation_z(angle + (PI / 2.))),
                ..default()
            },
            LetterDisplay {
                letter: character.to_ascii_uppercase(),
                active,
                position: offset - (offset_direction * 16.),
                radius: 64.,
                layer
            },
            SquishEffect::new(Vec3::ONE, Vec3::splat(2.), 0.01, 0., 0.25),
        )).set_parent(parent);
    }

    let shape = shapes::Circle {
        radius: radius + 32.,
        center: Vec2::ZERO,
    };

    commands.spawn((
        ShapeBundle {
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., -0.5)),
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
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., -0.1)),
                ..default()
            },
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
    mut reset_button: Query<(&mut ResetButton, &mut Visibility), (Without<BackspaceButton>, Without<NextLevelButton>)>,
    mut backspace_button: Query<(&mut BackspaceButton, &mut Visibility), (Without<ResetButton>, Without<NextLevelButton>)>,
    mut next_level_button: Query<(&mut NextLevelButton, &mut Visibility), (Without<BackspaceButton>, Without<ResetButton>)>,
    time: Res<Time>,
) {
    for completion in complete_events.read() {
        println!("completed! advancing to {}", completion.now_on_layer);
        let new_active = completion.now_on_layer;

        selection.current_layer_start_time = time.elapsed_seconds();

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
            stroke.color = if is_active { text_styles.active.color } else { text_styles.idle.color };
        }

        selection.built_word.clear();
        selection.positions.clear();
        selection.changed_this_frame = true;
        selection.current_layer = new_active;

        let in_gameplay_step = (new_active as usize) < selection.complete_solution.len();
        if in_gameplay_step {
            selection.target_word = selection.complete_solution[new_active as usize].clone();
        }
        
        let gameplay_button_vis = if in_gameplay_step {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        let (mut r_button, mut r_vis) = reset_button.single_mut();
        r_button.active = in_gameplay_step;
        *r_vis = gameplay_button_vis;

        let (mut b_button, mut b_vis) = backspace_button.single_mut();
        b_button.active = in_gameplay_step;
        *b_vis = gameplay_button_vis;

        let show_next_button = (new_active as usize) == selection.complete_solution.len();
        let (mut nl_button, mut nl_vis) = next_level_button.single_mut();
        nl_button.active = show_next_button;
        *nl_vis = if show_next_button { Visibility::Visible } else { Visibility::Hidden };

    }
}

fn update_mouse_position(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos_state: ResMut<MousePosition>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
) {
    if let Some(viewport_position) = windows.single().cursor_position() {
        let (camera, camera_transform) = cameras.single();

        mouse_pos_state.pos = camera.viewport_to_world_2d(camera_transform, viewport_position);
        mouse_pos_state.just_clicked = mouse_buttons.just_pressed(MouseButton::Left);
    }
    else if touches.any_just_pressed() {
        if let Some(viewport_position) = touches.first_pressed_position() {
            let (camera, camera_transform) = cameras.single();
            mouse_pos_state.pos = camera.viewport_to_world_2d(camera_transform, viewport_position);
            mouse_pos_state.just_clicked = true;
        }
        else {
            mouse_pos_state.pos = None;
            mouse_pos_state.just_clicked = false;
        }
    }
    else {
        mouse_pos_state.pos = None;
        mouse_pos_state.just_clicked = false;
    }
}

fn select_letters(
    mut selection: ResMut<WordSelection>,
    mut letters: Query<(&LetterDisplay, &Transform, &mut SquishEffect)>,
    mouse_state: Res<MousePosition>,
    mut character_events: EventReader<ReceivedCharacter>,
) {
    let mut recieved_chars = Vec::new();
    for ev in character_events.read() {
        recieved_chars.push((ev.char.as_bytes()[0] as char).to_ascii_uppercase());
    }
    let recieved_chars = recieved_chars;

    let mut changed_this_frame = false;

    for (letter, transform, mut squish) in letters.iter_mut() {
        if letter.active {
            let mut mouse_selected = false;

            if mouse_state.just_clicked {
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

                squish.reset();
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
        println!("checking if {} is valid solution for {}", selection.built_word, selection.target_word);
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

fn animate_demon(
    selection: Res<WordSelection>,
    mut demon_query: Query<(&mut Transform, &mut Visibility, &DemonFace)>,
    time: Res<Time>,
    mut complete_writer: EventWriter<PuzzleCompleteEvent>,
) {
    let (mut transform, mut vis, demon) = demon_query.single_mut();

    if selection.current_layer as usize >= selection.complete_solution.len() {
        *vis = Visibility::Visible;

        if selection.current_layer as usize > selection.complete_solution.len() {
            let t = (time.elapsed_seconds() - selection.current_layer_start_time) / demon.transition_time;
            let scale = demon.base_scale.lerp(demon.fill_scale, t);
            transform.scale = Vec3::splat(scale);

            if t > 1. {
                complete_writer.send(PuzzleCompleteEvent {});
            }
        }
    }
}

fn handle_next_level(
    button_query: Query<(&Transform, &NextLevelButton)>,
    mouse_pos: Res<MousePosition>,
    mut complete_writer: EventWriter<WordCompleteEvent>,
    selection: Res<WordSelection>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut is_clicked = false;

    let (transform, button) = button_query.single();

    if button.active {
        if let Some(pos) = mouse_pos.pos {
            let button_pos = transform.translation.truncate();
            if mouse_pos.just_clicked && pos.distance(button_pos) < 50. {
                is_clicked = true;
            }
        }

        if is_clicked || keys.just_pressed(KeyCode::Enter) {
            complete_writer.send(WordCompleteEvent { now_on_layer: selection.current_layer + 1} );
        }
    }
}

fn handle_reset(
    mut selection: ResMut<WordSelection>,
    mut button_query: Query<(&Transform, &ResetButton, &mut SquishEffect)>,
    mouse_pos: Res<MousePosition>,
) {
    let mut is_clicked = false;

    let (transform, button, mut squish) = button_query.single_mut();

    if button.active {
        if let Some(pos) = mouse_pos.pos {
            let button_pos = transform.translation.truncate();
            if mouse_pos.just_clicked && pos.distance(button_pos) < 50. {
                is_clicked = true;
            }
        }

        if is_clicked {
            selection.built_word.clear();
            selection.positions.clear();
            selection.changed_this_frame = true;

            squish.reset();
        }
    }
}

fn handle_backspace(
    mut selection: ResMut<WordSelection>,
    keys: Res<ButtonInput<KeyCode>>,
    mut button_query: Query<(&Transform, &BackspaceButton, &mut SquishEffect)>,
    mouse_pos: Res<MousePosition>,
) {
    let mut is_clicked = false;

    let (transform, button, mut squish) = button_query.single_mut();

    if button.active {
        if let Some(pos) = mouse_pos.pos {
            let button_pos = transform.translation.truncate();
            if mouse_pos.just_clicked && pos.distance(button_pos) < 50. {
                is_clicked = true;
            }
        }
    
        if is_clicked || keys.just_pressed(KeyCode::Backspace) {
            if selection.built_word.len() > 0 {
                selection.built_word.pop();
                selection.positions.pop();
                selection.changed_this_frame = true;

                squish.reset();
            }
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

fn spin_rings(
    time: Res<Time>,
    selection: Res<WordSelection>,
    mut rings: Query<(&mut Transform, &RingLayer)>,
) {
    if selection.current_layer as usize >= selection.complete_solution.len() {
        for (mut transform, ring) in rings.iter_mut() {
            let direction = if ring.layer % 2 == 0 { -1. } else { 1. };

            transform.rotate_z(direction * time.delta_seconds() * 0.3);
        }
    }
}