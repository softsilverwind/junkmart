use std::{
    f32::consts::PI,
    iter
};

use rand::prelude::*;

use bevy::{
    prelude::*,
    pbr::AmbientLight,
    render::camera::Camera,
    input::mouse::MouseButtonInput
};
use bevy_egui::{
    EguiContexts, egui::{self, RichText, Color32}
};
use bevy_tweening::Animator;

use crate::plugins::post_process::PostProcessCamera;

use super::{
    GameState,
    components::{Rotate, PointerLight},
    events::{NewsFeedUpdate, NewsLevel},
    resources::{AssetList, Chests, HoveredChest, Instructions, NewsFeed},
    utils::{self, item::Item}
};

pub mod instructions;

use instructions::Instruction;

pub fn add_systems(app: &mut App)
{
    instructions::add_instruction_systems(app);

    app
        .add_system(set_default_font.in_schedule(OnEnter(GameState::Play)))
        .add_system(spawn_level.in_schedule(OnEnter(GameState::Play)))
        .add_system(initialize_newsfeed.in_schedule(OnEnter(GameState::Play)))
        .add_system(update_newsfeed.in_set(OnUpdate(GameState::Play)))
        .add_system(write_newsfeed.in_set(OnUpdate(GameState::Play)))
        .add_system(mouse_move.in_set(OnUpdate(GameState::Play)))
        .add_system(mouse_click.in_set(OnUpdate(GameState::Play)))
        .add_system(rotate.in_set(OnUpdate(GameState::Play)))
    ;
}

fn set_default_font(mut contexts: EguiContexts)
{
    let ctx = contexts.ctx_mut();
    let mut style = (*ctx.style()).clone();

    *style.text_styles.get_mut(&egui::TextStyle::Body).unwrap() = egui::FontId::new(18.0, egui::FontFamily::Proportional);

    ctx.set_style(style);
}

fn spawn_level(
    mut commands: Commands,
    assets: Res<AssetList>,
    mut ambient_light: ResMut<AmbientLight>,
    mut chests: ResMut<Chests>,
)
{
    let mut rng = thread_rng();

    commands.spawn(
        PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                intensity: 40.0,
                range: 6.0,
                radius: 0.1,
                shadows_enabled: false,
                ..default()
            },
            ..default()
    })
    .insert((
        Camera3dBundle {
            transform: *utils::CAMERA_REST_POS,
            ..default()
        },
        PostProcessCamera
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight { intensity: 0.0, ..*utils::POINTER_LIGHT },
        transform: Transform::from_xyz(-1000.0, -1000.0, 1000.0),
        ..default()
    })
    .insert(PointerLight);

    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 0.10;

    // Ideally search for lamp objects on the scene and do not use arbitrary numbers until it works.

    let light_pos = [
        Vec3::new(-4.0, 2.0, 3.5),
        Vec3::new(-4.0, -1.5, 3.5),
        Vec3::new(4.0, 1.75, 3.5),
        Vec3::new(4.0, -1.5, 3.5),
        Vec3::new(-1.5, 4.0, 3.5),
        Vec3::new(1.5, 4.0, 3.5)
    ];

    for pos in light_pos {
        commands.spawn(PointLightBundle {
            point_light: *utils::POINT_LIGHT,
            transform: Transform::from_translation(pos),
            ..default()
        });
    }

    commands.spawn(SceneBundle {
            scene: assets.level.clone(),
            ..default()
        })
        .insert(Name::new("Level"));

    let mut items: Vec<Item> =
        iter::once(Item::Barrel).cycle().take(2)
        .chain(iter::once(Item::Burger).cycle().take(6))
        .chain(iter::once(Item::Gun).cycle().take(3))
        .chain(iter::once(Item::Pill).cycle().take(3))
        .chain(iter::once(Item::Screwdriver).cycle().take(6))
        .collect();

    items.shuffle(&mut rng);

    for x in 0..5 {
        for y in 0..4 {
            let id = commands.spawn(SceneBundle {
                scene: assets.chest.clone(),
                transform: Transform::from_xyz(-2.4 + x as f32 * 1.2, -1.8 + y as f32 * 1.2, 0.0),
                ..default()
            })
            .id();

            chests.0.insert((x, y), (id, items.pop().unwrap()));
        }
    }
}

fn initialize_newsfeed(
    mut ev_news: EventWriter<NewsFeedUpdate>,
)
{
    ev_news.send(NewsFeedUpdate(NewsLevel::EXTERNAL, "News Flash: The local junkyard has a new owner! Maybe now our little town will forget the tragic demise of the previous owner...".to_string()));
    ev_news.send(NewsFeedUpdate(NewsLevel::EVENT, "It is my honor to welcome my new employer! My name is Trevor Utorial! With any luck, we can redouble the $1000 in your pocket within three days! We should probably write down the inventory before opening shop! Bring a crate to the front to see what is inside!".to_string()));
}

fn update_newsfeed(
    mut ev_news: EventReader<NewsFeedUpdate>,
    mut news_feed: ResMut<NewsFeed>
)
{
    const MAX_LEN: usize = 20;
    for NewsFeedUpdate(level, text) in ev_news.iter() {
        let color = match level {
            NewsLevel::EXTERNAL => Color32::GRAY,
            NewsLevel::EVENT => Color32::WHITE,
            NewsLevel::CORRECT => Color32::GREEN,
            NewsLevel::WRONG => Color32::RED
        };

        news_feed.0.push_back(RichText::new(text).color(color));
    }

    let len = news_feed.0.len();
    if len > MAX_LEN {
        news_feed.0.drain(0..(len - MAX_LEN));
    }
}

fn write_newsfeed(
    mut contexts: EguiContexts,
    ev_news: EventReader<NewsFeedUpdate>,
    newsfeed: Res<NewsFeed>
)
{
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::bottom("bottom_panel")
        .height_range(50.0..=300.0)
        .default_height(150.0)
        .resizable(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for text in newsfeed.0.iter() {
                    let response = ui.label(text.clone());
                    if ev_news.len() > 0 {
                        response.scroll_to_me(None);
                    }
                }
            });
        });
}

fn mouse_click(
    mut instructions: ResMut<Instructions>,
    mut hovered_chest: ResMut<HoveredChest>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>
)
{
    if instructions.0.len() > 0 {
        return;
    }

    if !mouse_button_input_events.iter().any(|evt| evt.button == MouseButton::Left && evt.state.is_pressed()) {
        return;
    }

    if let Some(pos) = hovered_chest.0 {
        instructions.0.push_back(Instruction::SwapWithFirst(pos));
        instructions.0.push_back(Instruction::CameraToFirstChest);
        instructions.0.push_back(Instruction::PresentItem);
        instructions.0.push_back(Instruction::HandleEffects);
        instructions.0.push_back(Instruction::HideItem);
        instructions.0.push_back(Instruction::CameraToRest);
        instructions.0.push_back(Instruction::HandleStatusEffects);
        instructions.0.push_back(Instruction::EndOfTurn);
        hovered_chest.0 = None;
    }
}

fn mouse_move(
    mut commands: Commands,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut hovered_chest: ResMut<HoveredChest>,
    chests: Res<Chests>,
    mut objects: Query<&mut Transform, Without<PointerLight>>,
    instructions: Res<Instructions>,
    mut pointer_light: Query<&mut Transform, With<PointerLight>>
)
{
    if instructions.0.len() > 0 {
        return;
    }

    let window = windows.single();
    let camera = cameras.single();

    let Some(position) = window.cursor_position() else { return; };

    let Some(ray) = camera.0.viewport_to_world(&camera.1, position) else { return; };
    let Some(dist) = ray.intersect_plane(Vec3::new(0.0, 0.0, 1.0), Vec3::Z) else { return; };

    let point = ray.get_point(dist);

    pointer_light.single_mut().translation = Vec3 { z: 3.0, ..point };

    if (-2.9..2.9).contains(&point.x) && (-2.3..2.3).contains(&point.y) {
        let newpos = (((point.x + 2.9) / 1.2) as i32, ((point.y + 2.3) / 1.2) as i32);

        if let Some(pos) = hovered_chest.0 {
            if pos == newpos { return; }
            let old_chest = chests.0[&pos].0;
            let start = objects.get_mut(old_chest).unwrap().translation;
            commands.entity(old_chest).insert(Animator::new(utils::tween::lift(start, 0.0, 100)));
        }

        let new_chest = chests.0[&newpos].0;
        let start = objects.get(new_chest).unwrap().translation;
        commands.entity(new_chest).insert(Animator::new(utils::tween::lift(start, 0.5, 100)));

        hovered_chest.0 = Some(newpos);
    }
    else {
        if let Some(pos) = hovered_chest.0 {
            let old_chest = chests.0[&pos].0;
            let start = objects.get_mut(old_chest).unwrap().translation;
            commands.entity(old_chest).insert(Animator::new(utils::tween::lift(start, 0.0, 100)));
            hovered_chest.0 = None;
        }
    }
}

fn rotate(
    mut query: Query<&mut Transform, With<Rotate>>,
    time: Res<Time>
)
{
    let angle = time.delta_seconds() * 2.0 * PI;

    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_rotation_z(angle))
    }
}
