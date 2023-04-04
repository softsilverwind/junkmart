use bevy::prelude::*;

mod mainmenu;
mod play;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState
{
    #[default]
    MainMenu,
    LoadPlay,
    Play,
}

pub struct StatePlugin;

impl Plugin for StatePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_state::<GameState>()
            .add_plugin(mainmenu::MainMenuPlugin)
            .add_plugin(play::PlayPlugin);
    }
}

