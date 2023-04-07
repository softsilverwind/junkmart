use std::io::Cursor;

use bevy::{log::LogPlugin, prelude::*, window::PrimaryWindow, winit::WinitWindows};
use winit::window::Icon;

mod plugins;
mod states;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Junk-Mart".to_string(),
                        resolution: (800., 600.).into(),
                        canvas: Some("#bevy".to_owned()),
                        resize_constraints: WindowResizeConstraints {
                            min_width: 640.0,
                            min_height: 480.0,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "warn,wgpu_core=error,junkmart=debug".into(),
                    level: bevy::log::Level::DEBUG,
                }),
        )
        .add_plugin(plugins::post_process::PostProcessingPlugin)
        .add_plugin(states::StatePlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(bevy_tweening::TweeningPlugin)
        .add_plugin(bevy_kira_audio::AudioPlugin)
        // .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
        .add_system(set_window_icon.on_startup())
        .run();
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let primary = windows.get_window(primary_entity).unwrap();
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
