use bevy::{app::AppExit, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_egui::{
    egui::{Align, CentralPanel, Color32, Label, Layout, RichText, Sense, TextureId},
    EguiContexts,
};

use super::GameState;

mod resources;

use resources::AssetList;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui.in_set(OnUpdate(GameState::MainMenu)))
            .add_loading_state(
                LoadingState::new(GameState::LoadMainMenu).continue_to_state(GameState::MainMenu),
            )
            .add_collection_to_loading_state::<_, AssetList>(GameState::LoadMainMenu);
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
    mut image: Local<Option<TextureId>>,
    asset_list: Res<AssetList>,
) {
    if image.is_none() {
        *image = Some(contexts.add_image(asset_list.mainmenu_image.clone_weak()));
    }

    let ctx = contexts.ctx_mut();

    CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.heading(
                RichText::new("Welcome to Junk-Mart")
                    .size(63.0)
                    .color(Color32::LIGHT_BLUE),
            );
            ui.label(RichText::new("Where dreams come to die").strikethrough());
            ui.label(RichText::new("Where we fnid the itmespbb").strikethrough());
            ui.label(
                RichText::new("We will find what you ask, or die trying!")
                    .size(21.0)
                    .color(Color32::LIGHT_BLUE),
            );
            ui.add_space(20.0);
            ui.image(image.unwrap(), [400.0, 225.0])
        });

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            if ui
                .add(Label::new(RichText::new("EXIT").size(63.0)).sense(Sense::click()))
                .on_hover_text("WHY THO")
                .clicked()
            {
                exit.send(AppExit);
            }

            ui.add_space(50.0);
            if ui
                .add(Label::new(RichText::new("START").size(63.0)).sense(Sense::click()))
                .clicked()
            {
                next_state.set(GameState::LoadPlay);
            }
        })
    });
}
