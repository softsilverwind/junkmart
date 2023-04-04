use bevy::{
    prelude::*,
    pbr::AmbientLight,
    render::camera::Camera,
    input::mouse::MouseButtonInput
};
use bevy_egui::{
    EguiContexts, egui
};
use bevy_tweening::Animator;

use super::{
    resources::{AssetList, WidgetSize, Chests, HoveredChest, Instructions, Instruction},
    utils
};

pub mod instructions;

pub fn set_default_font(mut contexts: EguiContexts)
{
    let ctx = contexts.ctx_mut();
    let mut style = (*ctx.style()).clone();

    *style.text_styles.get_mut(&egui::TextStyle::Body).unwrap() = egui::FontId::new(18.0, egui::FontFamily::Proportional);

    ctx.set_style(style);
}

pub fn populate_side_panel(
    mut contexts: EguiContexts,
    mut widget_size: ResMut<WidgetSize>
)
{
    let ctx = contexts.ctx_mut();

    widget_size.0 = egui::TopBottomPanel::bottom("bottom_panel")
        .height_range(50.0..=300.0)
        .default_height(150.0)
        .resizable(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.label("Tutorial:");
                ui.label("I hear you inherited this junkyard and decided to sell whatever is inside! Good for you! You should click a box to see what is inside!");
                ui.label("Damn, son, I can see that you no longer have money to pay me! You are on your own, bye!");
            });
        })
        .response
        .rect
        .height();
}

pub fn spawn_level(
    mut commands: Commands,
    assets: Res<AssetList>,
    mut ambient_light: ResMut<AmbientLight>,
    mut chests: ResMut<Chests>
)
{
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, -8.0, 17.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let point_light = PointLight {
        color: Color::WHITE,
        intensity: 20.0,
        range: 10.0,
        radius: 0.1,
        shadows_enabled: true,
        ..default()
    };

    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 0.10;

    // Ideally I would search for light objects on the scene.

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
            point_light,
            transform: Transform::from_translation(pos),
            ..default()
        });
    }

    commands.spawn(SceneBundle {
            scene: assets.level.clone(),
            ..default()
        })
        .insert(Name::new("Level"));


    for x in 0..5 {
        for y in 0..4 {
            let id = commands.spawn(SceneBundle {
                scene: assets.chest.clone(),
                transform: Transform::from_xyz(-2.4 + x as f32 * 1.2, -1.8 + y as f32 * 1.2, 0.0),
                ..default()
            })
            .id();

            chests.0.insert((x, y), (id, super::resources::Item::Gun));
        }
    }
}

pub fn mouse_click(
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
        instructions.0.push_back(Instruction::Wait(1.0));
        hovered_chest.0 = None;
    }
}

pub fn mouse_move(
    mut commands: Commands,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut hovered_chest: ResMut<HoveredChest>,
    chests: Res<Chests>,
    mut objects: Query<&mut Transform>,
    instructions: Res<Instructions>
)
{
    if instructions.0.len() > 0 {
        return;
    }

    let window = windows.get_single().expect("No window found!");
    let camera = cameras.get_single().unwrap();

    let Some(position) = window.cursor_position() else { return; };

    let Some(ray) = camera.0.viewport_to_world(&camera.1, position) else { return; };
    let Some(dist) = ray.intersect_plane(Vec3::new(0.0, 0.0, 1.0), Vec3::Z) else { return; };

    let point = ray.get_point(dist);

    if (-2.9..2.9).contains(&point.x) && (-2.3..2.3).contains(&point.y) {
        let newpos = (((point.x + 2.9) / 1.2) as i32, ((point.y + 2.3) / 1.2) as i32);

        if let Some(pos) = hovered_chest.0 {
            if pos == newpos { return; }
            let old_chest = chests.0[&pos].0;
            let start = objects.get_mut(old_chest).unwrap().translation;
            commands.entity(old_chest).insert(Animator::new(utils::lift(start, 0.0)));
        }

        let new_chest = chests.0[&newpos].0;
        let start = objects.get(new_chest).unwrap().translation;
        commands.entity(new_chest).insert(Animator::new(utils::lift(start, 0.5)));

        hovered_chest.0 = Some(newpos);
    }
    else {
        if let Some(pos) = hovered_chest.0 {
            let old_chest = chests.0[&pos].0;
            let start = objects.get_mut(old_chest).unwrap().translation;
            commands.entity(old_chest).insert(Animator::new(utils::lift(start, 0.0)));
            hovered_chest.0 = None;
        }
    }
}
