use std::path::Path;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
    render::render_resource::encase::rts_array::Length,
    window::PrimaryWindow,
};

const PANEL_COLOR: Color = Color::srgba(0.798, 0.506, 0.561, 0.3);
const BORDER_COLOR: Color = Color::srgb(0.18, 0.176, 0.259);
const BUTTON_COLOR: Color = Color::srgb(0.443, 0.941, 0.353);
const HOVER_BORDER: Color = Color::srgb(0.831, 0.29, 0.463);
const PRESSED_BORDER: Color = Color::srgb(0.988, 0.565, 0.239);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    LoadAssets,
    InLevelEdit,
}

#[derive(Resource)]
enum ClickState {
    FirstClick,
    SecondClick(Vec3),
    Draw([Vec3; 2]),
}

#[derive(Component)]
struct TextInputBox;

#[derive(Resource, Clone)]
struct TextInput(String);

#[derive(Component)]
struct TextChange;

#[derive(Component)]
struct TileSelectionUi;

#[derive(Resource)]
struct Visible(bool);

#[derive(Component)]
struct Collider {
    pos: Vec3,
    size: Vec2,
}

#[derive(Bundle)]
struct ColliderBundle<T: Component> {
    collider: Collider,
    generic_component: T,
}

impl<T: Component> ColliderBundle<T> {
    fn new(pos: Vec3, size: Vec2, component: T) -> Self {
        Self {
            collider: Collider { pos, size },
            generic_component: component,
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Tile(usize);

#[derive(Component)]
struct Hazard(usize);

#[derive(Component)]
struct Mob;

#[derive(Resource)]
struct SelectedTile(usize);

#[derive(Component)]
struct TileButton(usize);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum ClickAnd {
    DrawTile,
    DrawHazard,
    DrawMob,
    Erase,
    PlacePlayer,
}

#[derive(Component)]
enum ToolType {
    Tile,
    Hazard,
    Mob,
    Erase,
    Player,
}

#[derive(Event, Clone, Copy)]
struct ClickEvent {
    cursor_pos: Vec2,
}

fn detect_inputs(
    mouse: Res<ButtonInput<MouseButton>>,
    mut event_writer: EventWriter<ClickEvent>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if mouse.pressed(MouseButton::Left) {
        let cursor_pos = window_q.single().cursor_position().unwrap_or(Vec2::ZERO);
        event_writer.send(ClickEvent {
            cursor_pos: cursor_pos,
        });
    }
}

fn check_ui_position(transform: &GlobalTransform, node: &Node) -> (Vec2, Vec2) {
    let ui_position = transform.translation().truncate();
    let ui_size = node.size();

    let min = ui_position - (ui_size / 2.0);
    let max = ui_position + (ui_size / 2.0);
    (min, max)
}

fn screen_to_world(camera: &Camera, camera_transform: &GlobalTransform, screen_pos: Vec2) -> Vec3 {
    let size = Vec2::splat(24.0);
    let half_size = Vec2::splat(12.0);

    let world_pos = camera
        .viewport_to_world_2d(camera_transform.into(), screen_pos)
        .unwrap_or_default();

    let tile_pos = (world_pos / size.x).floor() * size.y + half_size;
    tile_pos.extend(1.0)
}

fn handle_mouse_click(
    mut commands: Commands,
    cam_q: Query<(&Camera, &GlobalTransform)>,
    mut click_event_r: EventReader<ClickEvent>,
    state: Res<State<ClickAnd>>,
    mut transform_set: ParamSet<(
        Query<(&Transform, Entity), With<Sprite>>,
        Query<&mut Transform, (With<Player>, Without<Tile>)>,
    )>,
    ent: Query<Entity, (With<Player>, Without<Tile>)>,
    node_q: Query<(&GlobalTransform, &Node)>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    selected_tile: Res<SelectedTile>,
    text_res: Res<TextInput>,
    mut click_state: ResMut<ClickState>,
) {
    let size = Vec2::splat(24.0);

    let texture = asset_server.load(text_res.0.clone());
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::splat(24), 4, 4, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let cam = cam_q.single();
    for click_event in click_event_r
        .par_read()
        .batching_strategy(bevy::ecs::batching::BatchingStrategy::default())
    {
        for (transform, node) in &node_q {
            let min_max = check_ui_position(&transform, &node);
            let min = min_max.0;
            let max = min_max.1;

            if click_event.0.cursor_pos.x >= min.x
                && click_event.0.cursor_pos.x <= max.x
                && click_event.0.cursor_pos.y >= min.y
                && click_event.0.cursor_pos.y <= max.y
            {
                return;
            }
        }
        let click_pos = screen_to_world(cam.0, cam.1, click_event.0.cursor_pos);
        match state.get() {
            ClickAnd::DrawTile => {
                commands
                    .spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(size.clone()),
                                ..default()
                            },
                            transform: Transform::from_translation(click_pos),
                            texture: texture.clone(),
                            ..default()
                        },
                        TextureAtlas {
                            index: selected_tile.0,
                            layout: texture_atlas_handle.clone(),
                        },
                    ))
                    .insert(ColliderBundle::new(
                        click_pos,
                        size.clone(),
                        Tile(selected_tile.0),
                    ));
            }
            ClickAnd::DrawHazard => {
                commands
                    .spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(size.clone()),
                                ..default()
                            },
                            transform: Transform::from_translation(click_pos),
                            texture: texture.clone(),
                            ..default()
                        },
                        TextureAtlas {
                            index: selected_tile.0,
                            layout: texture_atlas_handle.clone(),
                        },
                    ))
                    .insert(ColliderBundle::new(
                        click_pos,
                        size.clone(),
                        Hazard(selected_tile.0),
                    ));
            }
            ClickAnd::DrawMob => {
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgb(0.71, 0.075, 0.031),
                            custom_size: Some(size.clone()),
                            ..default()
                        },
                        transform: Transform::from_translation(click_pos),
                        ..default()
                    })
                    .insert(ColliderBundle::new(click_pos, size.clone(), Mob));
            }
            ClickAnd::Erase => {
                for (transform, entity) in &mut transform_set.p0().iter_mut() {
                    if transform.translation.xy() == click_pos.xy() {
                        commands.entity(entity).despawn();
                    }
                }
            }
            ClickAnd::PlacePlayer => {
                if let Some(ent) = ent.iter().next() {
                    if let Ok(mut transform) = transform_set.p1().get_mut(ent) {
                        transform.translation = click_pos;
                    }
                } else {
                    commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::WHITE,
                                custom_size: Some(size.clone()),
                                ..default()
                            },
                            transform: Transform::from_translation(click_pos),
                            ..default()
                        })
                        .insert(Collider {
                            pos: click_pos,
                            size: Vec2::splat(24.0),
                        })
                        .insert(Player);
                }
            }
        }
    }
}

fn setup_path_input_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let node = NodeBundle {
        style: Style {
            width: Val::Px(200.0),
            height: Val::Px(200.0),
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            ..default()
        },
        background_color: BackgroundColor(PANEL_COLOR),
        ..default()
    };

    let text_style = TextStyle {
        font: asset_server.load("../assets/FiraSans-Bold.ttf"),
        font_size: 25.0,
        color: Color::BLACK,
    };
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(TextBundle {
            style: Style {
                width: Val::Px(260.0),
                height: Val::Px(30.0),
                ..default()
            },
            background_color: BackgroundColor(Color::WHITE),
            text: Text::from_section("".to_string(), text_style.clone()),
            ..default()
        })
        .insert(TextInputBox);
}

fn setup_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_tool_bar_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button = ButtonBundle {
        style: Style {
            width: Val::Px(60.0),
            height: Val::Px(40.0),
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        background_color: BackgroundColor(BUTTON_COLOR),
        border_color: BorderColor(BORDER_COLOR),
        ..default()
    };
    let text_style = TextStyle {
        font: asset_server.load("../assets/FiraSans-Bold.ttf"),
        font_size: 25.0,
        color: HOVER_BORDER,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                align_self: AlignSelf::Start,
                width: Val::Px(80.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Baseline,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                display: Display::Grid,
                ..default()
            },
            background_color: BackgroundColor(PANEL_COLOR),
            border_color: BorderColor(BORDER_COLOR),
            ..default()
        })
        .with_children(|parent| {
            // level editor toolbar buttons

            parent
                .spawn(button.clone())
                .with_children(|p| {
                    p.spawn(TextBundle::from_section("Tile", text_style.clone()));
                })
                .insert(ToolType::Tile);

            parent
                .spawn(button.clone())
                .with_children(|p| {
                    p.spawn(TextBundle::from_section("Hazard", text_style.clone()));
                })
                .insert(ToolType::Hazard);

            parent
                .spawn(button.clone())
                .with_children(|p| {
                    p.spawn(TextBundle::from_section("Mob", text_style.clone()));
                })
                .insert(ToolType::Mob);

            parent
                .spawn(button.clone())
                .with_children(|p| {
                    p.spawn(TextBundle::from_section("Erase", text_style.clone()));
                })
                .insert(ToolType::Erase);

            parent
                .spawn(button.clone())
                .with_children(|p| {
                    p.spawn(TextBundle::from_section("Player", text_style.clone()));
                })
                .insert(ToolType::Player);
        });
}
fn setup_text_guide(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("../assets/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 20.0,
        ..default()
    };

    let node = NodeBundle {
        style: Style {
            width: Val::Px(400.0),
            height: Val::Px(100.0),
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        ..default()
    };

    commands.spawn(node.clone()).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "open tile selector\nTAB",
            text_style.clone(),
        ));
        parent.spawn(TextBundle::from_section(
            "save level\nCTRL-s",
            text_style.clone(),
        ));
        parent.spawn(TextBundle::from_section(
            "clear canvas\nCTRL-r",
            text_style.clone(),
        ));
        parent
            .spawn(TextBundle::from_section("FPS \n", text_style.clone()))
            .insert(TextChange);
    });
}

fn fps_debug_text_system(
    diagnostics: Res<DiagnosticsStore>,
    mut text_q: Query<&mut Text, With<TextChange>>,
) {
    text_q.par_iter_mut().for_each(|mut text| {
        let mut fps = 0.0;
        if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                fps = fps_smoothed;
            }
        }
        text.sections[0].value = format!("{fps:.1}");
    });
}

fn text_input_system(
    mut text_q: Query<&mut Text, (With<TextInputBox>, Without<TextChange>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut text_res: ResMut<TextInput>,
) {
    for mut text in &mut text_q {
        // Handle backspace key to remove characters
        if keyboard_input.pressed(KeyCode::Backspace) {
            text.sections[0].value.pop();
        }
        if keyboard_input.just_pressed(KeyCode::Slash) {
            text.sections[0].value.push('/');
        }
        if keyboard_input.just_pressed(KeyCode::KeyA) {
            text.sections[0].value.push('a');
        }
        if keyboard_input.just_pressed(KeyCode::KeyB) {
            text.sections[0].value.push('b');
        }
        if keyboard_input.just_pressed(KeyCode::KeyC) {
            text.sections[0].value.push('c');
        }
        if keyboard_input.just_pressed(KeyCode::KeyD) {
            text.sections[0].value.push('d');
        }
        if keyboard_input.just_pressed(KeyCode::KeyE) {
            text.sections[0].value.push('e');
        }
        if keyboard_input.just_pressed(KeyCode::KeyF) {
            text.sections[0].value.push('f');
        }
        if keyboard_input.just_pressed(KeyCode::KeyG) {
            text.sections[0].value.push('g');
        }
        if keyboard_input.just_pressed(KeyCode::KeyH) {
            text.sections[0].value.push('h');
        }
        if keyboard_input.just_pressed(KeyCode::KeyI) {
            text.sections[0].value.push('i');
        }
        if keyboard_input.just_pressed(KeyCode::KeyJ) {
            text.sections[0].value.push('j');
        }
        if keyboard_input.just_pressed(KeyCode::KeyK) {
            text.sections[0].value.push('k');
        }
        if keyboard_input.just_pressed(KeyCode::KeyL) {
            text.sections[0].value.push('l');
        }
        if keyboard_input.just_pressed(KeyCode::KeyM) {
            text.sections[0].value.push('m');
        }
        if keyboard_input.just_pressed(KeyCode::KeyN) {
            text.sections[0].value.push('n');
        }
        if keyboard_input.just_pressed(KeyCode::KeyO) {
            text.sections[0].value.push('o');
        }
        if keyboard_input.just_pressed(KeyCode::KeyP) {
            text.sections[0].value.push('p');
        }
        if keyboard_input.just_pressed(KeyCode::KeyQ) {
            text.sections[0].value.push('q');
        }
        if keyboard_input.just_pressed(KeyCode::KeyR) {
            text.sections[0].value.push('r');
        }
        if keyboard_input.just_pressed(KeyCode::KeyS) {
            text.sections[0].value.push('s');
        }
        if keyboard_input.just_pressed(KeyCode::KeyT) {
            text.sections[0].value.push('t');
        }
        if keyboard_input.just_pressed(KeyCode::KeyU) {
            text.sections[0].value.push('u');
        }
        if keyboard_input.just_pressed(KeyCode::KeyV) {
            text.sections[0].value.push('v');
        }
        if keyboard_input.just_pressed(KeyCode::KeyW) {
            text.sections[0].value.push('w');
        }
        if keyboard_input.just_pressed(KeyCode::KeyX) {
            text.sections[0].value.push('x');
        }
        if keyboard_input.just_pressed(KeyCode::KeyY) {
            text.sections[0].value.push('y');
        }
        if keyboard_input.just_pressed(KeyCode::KeyZ) {
            text.sections[0].value.push('z');
        }
        if keyboard_input.any_just_pressed([KeyCode::ShiftLeft, KeyCode::Minus]) {
            text.sections[0].value.push('_');
        }
        if keyboard_input.just_pressed(KeyCode::Period) {
            text.sections[0].value.push('.');
        }
        if keyboard_input.pressed(KeyCode::Enter) {
            text_res.0 = text.sections[0].value.clone();
            app_state.set(AppState::InLevelEdit);
        }
    }
}

//tool bar interactions
fn tool_button_interaction(
    mut interaction_q: Query<
        (&Interaction, &mut BorderColor, &ToolType),
        (Changed<Interaction>, With<Button>, Without<TileButton>),
    >,
    mut tool_state: ResMut<NextState<ClickAnd>>,
) {
    for (interact, mut color, tool) in &mut interaction_q {
        match *interact {
            Interaction::Pressed => {
                *color = BorderColor(PRESSED_BORDER);
                match *tool {
                    ToolType::Tile => {
                        tool_state.set(ClickAnd::DrawTile);
                    }
                    ToolType::Hazard => {
                        tool_state.set(ClickAnd::DrawHazard);
                    }
                    ToolType::Mob => {
                        tool_state.set(ClickAnd::DrawMob);
                    }
                    ToolType::Erase => {
                        tool_state.set(ClickAnd::Erase);
                    }
                    ToolType::Player => {
                        tool_state.set(ClickAnd::PlacePlayer);
                    }
                }
            }
            Interaction::Hovered => {
                *color = BorderColor(HOVER_BORDER);
            }
            Interaction::None => {
                *color = BorderColor(BORDER_COLOR);
            }
        }
    }
}
fn tile_selector_interaction(
    mut interaction_q: Query<
        (&Interaction, &mut BorderColor, &TileButton),
        (Changed<Interaction>, With<Button>, Without<ToolType>),
    >,
    mut tile_selected: ResMut<SelectedTile>,
) {
    for (interaction, mut color, tile_button) in &mut interaction_q {
        match *interaction {
            Interaction::Pressed => {
                *color = BorderColor(PRESSED_BORDER);
                tile_selected.0 = tile_button.0;
            }
            Interaction::Hovered => {
                *color = BorderColor(HOVER_BORDER);
            }
            Interaction::None => {
                *color = BorderColor(BORDER_COLOR);
            }
        }
    }
}

fn setup_pop_up_tile_selector(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    text_res: Res<TextInput>,
) {
    let texture_handle = asset_server.load(text_res.0.clone());
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::splat(24), 4, 4, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let atlas_length = texture_atlases
        .get(&texture_atlas_handle)
        .expect("failed to fetch texture")
        .textures
        .len();

    let button = ButtonBundle {
        style: Style {
            width: Val::Px(24.0),
            height: Val::Px(24.0),
            border: UiRect::all(Val::Px(2.0)),
            justify_items: JustifyItems::Center,
            ..default()
        },
        image: UiImage::new(texture_handle),
        border_color: BorderColor(BORDER_COLOR),
        ..default()
    };
    let mut node = NodeBundle {
        style: Style {
            width: Val::Px(230.0),
            height: Val::Px(200.0),
            display: Display::None,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,

            grid_template_columns: RepeatedGridTrack::flex(4, 0.1),
            // Set the grid to have 4 rows all with sizes minmax(0, 1fr)
            // This creates 4 exactly evenly sized rows
            grid_template_rows: RepeatedGridTrack::flex(4, 0.1),

            ..default()
        },
        background_color: BackgroundColor(PANEL_COLOR),
        ..default()
    };

    commands
        .spawn(node.clone())
        .with_children(|parent| {
            for index in 0..atlas_length {
                parent
                    .spawn((
                        button.clone(),
                        TextureAtlas {
                            index: index,
                            layout: texture_atlas_handle.clone(),
                        },
                    ))
                    .insert(TileButton(index));
            }
        })
        .insert(TileSelectionUi);
}

fn toggle_tile_selector(
    mut tile_selection_query: Query<&mut Style, With<TileSelectionUi>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<Visible>,
) {
    for mut style in &mut tile_selection_query {
        if keyboard_input.just_pressed(KeyCode::Tab) {
            visible.0 = !visible.0;
        }
        match visible.0 {
            true => style.display = Display::Grid,
            false => style.display = Display::None,
        };
    }
}

fn reset_on_key_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut sprite_q: Query<Entity, With<Sprite>>,
    mut commands: Commands,
) {
    if !keyboard_input.all_pressed([KeyCode::ControlLeft, KeyCode::KeyR]) {
        return;
    }

    for ent in &sprite_q {
        commands.entity(ent).despawn_recursive();
    }
}

fn camera_movemovent(
    mut camera_q: Query<&mut Transform, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if !keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]) {
        return;
    }
    let mut transform = camera_q.single_mut();
    let mut direction = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    let move_delta = direction.normalize_or_zero() * 300.0 * time.delta_seconds();
    transform.translation += move_delta.extend(0.);
}

/*
fn save_level(
    mut level_set: ParamSet<(
        Query<(&Transform, &Sprite), With<Player>>,
        Query<(&Transform, &Sprite), With<Tile>>,
    )>,
    key_pressed: Res<ButtonInput<KeyCode>>,
) {
    if key_pressed.pressed(KeyCode::ControlLeft) && key_pressed.just_pressed(KeyCode::KeyS) {
        let mut tiles: Vec<TileData> = Vec::new();
        let mut player_data = PlayerData {
            pos: Vec3::ZERO,
            size: Vec2::ZERO,
        };

        //this code queries tile positions, and player position and then serializes that data to json

        for (player_transform, player_sprite) in level_set.p0().iter() {
            player_data.pos = player_transform.translation;
            player_data.size = player_sprite.custom_size.unwrap_or(Vec2::splat(24.0));
        }

        for (tile_transform, tile_sprite) in level_set.p1().iter() {
            tiles.push(TileData {
                pos: tile_transform.translation,
                size: tile_sprite.custom_size.unwrap_or(Vec2::splat(24.0)),
            });
        }

        let level = LevelData {
            player_data,
            tile_data: tiles,
        };
        let json = serde_json::to_string_pretty(&level).expect("failed tp serialize");
        std::fs::write("level.json", json).expect("Failed to write level to file");
    }
}*/

fn despawn_path_input(mut commands: Commands, mut input_q: Query<Entity, With<TextInputBox>>) {
    for ent in &mut input_q {
        commands.entity(ent).despawn();
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .insert_state(AppState::LoadAssets)
        .add_event::<ClickEvent>()
        .insert_resource(TextInput(String::new()))
        .insert_resource(Visible(false))
        .insert_resource(SelectedTile(0))
        .insert_resource(ClickState::FirstClick)
        .insert_state(ClickAnd::DrawTile)
        .add_systems(Startup, setup_path_input_ui)
        .add_systems(
            OnEnter(AppState::InLevelEdit),
            (
                setup_pop_up_tile_selector,
                setup_tool_bar_ui,
                setup_text_guide,
                despawn_path_input,
            ),
        )
        .add_systems(
            Update,
            (
                tool_button_interaction.run_if(in_state(AppState::InLevelEdit)),
                tile_selector_interaction.run_if(in_state(AppState::InLevelEdit)),
                toggle_tile_selector.run_if(in_state(AppState::InLevelEdit)),
                fps_debug_text_system.run_if(in_state(AppState::InLevelEdit)),
                text_input_system.run_if(in_state(AppState::LoadAssets)),
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                detect_inputs,
                handle_mouse_click,
                reset_on_key_input,
                camera_movemovent,
            )
                .run_if(in_state(AppState::InLevelEdit)),
        )
        .run();
}
