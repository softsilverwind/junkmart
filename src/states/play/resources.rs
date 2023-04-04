use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;

#[derive(Resource, AssetCollection)]
pub struct AssetList
{
    #[asset(path = "objects/level.glb#Scene0")] pub level: Handle<Scene>,
    #[asset(path = "objects/chest.glb#Scene0")] pub chest: Handle<Scene>
}

#[derive(Clone)]
pub enum Item
{
    Barrel,
    Burger,
    Gun,
    Pill,
    Screwdriver
}

pub enum Instruction
{
    SwapWithFirst((i32, i32)),
    CameraToFirst,
    CameraToRest,
    PresentItem,
    Wait(f32)
}

#[derive(Default, Resource)] pub struct WidgetSize(pub f32);
#[derive(Default, Resource)] pub struct Chests(pub HashMap<(i32, i32), (Entity, Item)>);
#[derive(Default, Resource)] pub struct HoveredChest(pub Option<(i32, i32)>);
#[derive(Default, Resource)] pub struct Instructions(pub VecDeque<Instruction>);
