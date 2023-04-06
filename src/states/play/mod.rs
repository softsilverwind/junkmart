use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use super::GameState;

mod resources;
mod systems;
mod utils;

mod components
{
    use bevy::prelude::*;

    #[derive(Component)] pub struct Rotate;
    #[derive(Component)] pub struct PointerLight;
}

mod events
{
    pub enum NewsLevel {
        EXTERNAL, EVENT, CORRECT, WRONG
    }

    pub struct NewsFeedUpdate(pub NewsLevel, pub String);
}

pub struct PlayPlugin;

impl Plugin for PlayPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .add_event::<events::NewsFeedUpdate>()
            .add_loading_state(LoadingState::new(GameState::LoadPlay).continue_to_state(GameState::Play))
            .add_collection_to_loading_state::<_, resources::AssetList>(GameState::LoadPlay)
            .add_collection_to_loading_state::<_, resources::SoundList>(GameState::LoadPlay)
        ;

        resources::init_resources(app);
        systems::add_systems(app);
    }
}
