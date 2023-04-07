use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Resource, AssetCollection)]
pub struct AssetList {
    #[asset(path = "images/mainmenu.png")]
    pub mainmenu_image: Handle<Image>,
}
