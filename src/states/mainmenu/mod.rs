use bevy::{prelude::*, app::AppExit};
use bevy_egui::{
    EguiContexts,
    egui::{
        Align, RichText, CentralPanel, Layout, Color32, Sense, Label
    }
};

use super::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_system(ui.in_set(OnUpdate(GameState::MainMenu)));
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>
)
{
    let ctx = contexts.ctx_mut();

    CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.heading(RichText::new("Welcome to Junk-Mart").size(63.0).color(Color32::LIGHT_BLUE));
            ui.label(RichText::new("Where dreams come to die").strikethrough());
            ui.label(RichText::new("Where we fnid the itmespbb").strikethrough());
            ui.label(RichText::new("We will find what you ask, or die trying!").size(21.0).color(Color32::LIGHT_BLUE));
        });

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            if ui.add(Label::new(RichText::new("EXIT").size(63.0)).sense(Sense::click())).on_hover_text("WHY THO").clicked() {
                exit.send(AppExit);
            }

            ui.add_space(50.0);
            if ui.add(Label::new(RichText::new("START").size(63.0)).sense(Sense::click())).clicked() {
                next_state.set(GameState::LoadPlay);
            }
        })
    });
}
