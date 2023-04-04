use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use super::GameState;

mod resources;
mod systems;
mod utils;

pub struct PlayPlugin;

impl Plugin for PlayPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .init_resource::<resources::WidgetSize>()
            .init_resource::<resources::Chests>()
            .init_resource::<resources::HoveredChest>()
            .init_resource::<resources::Instructions>()
            .add_loading_state(LoadingState::new(GameState::LoadPlay).continue_to_state(GameState::Play))
            .add_collection_to_loading_state::<_, resources::AssetList>(GameState::LoadPlay)
            .add_system(systems::set_default_font.in_schedule(OnEnter(GameState::Play)))
            .add_system(systems::spawn_level.in_schedule(OnEnter(GameState::Play)))
            .add_system(systems::populate_side_panel.in_set(OnUpdate(GameState::Play)))
            .add_system(systems::mouse_move.in_set(OnUpdate(GameState::Play)))
            .add_system(systems::mouse_click.in_set(OnUpdate(GameState::Play)))
            .add_system(systems::instructions::wait.in_set(OnUpdate(GameState::Play)))
            .add_system(systems::instructions::swap_with_first.in_set(OnUpdate(GameState::Play)));
    }
}
