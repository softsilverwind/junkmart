use std::{
    collections::VecDeque,
    fmt::{Display, Formatter},
    ops::{AddAssign, SubAssign},
};

use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use bevy_egui::egui::RichText;
use bevy_kira_audio::prelude::*;

use super::{
    systems::instructions::Instruction,
    utils::{item::Item, StatusEffect},
};

#[derive(Resource, AssetCollection)]
pub struct AssetList {
    #[asset(path = "objects/level.glb#Scene0")]
    pub level: Handle<Scene>,
    #[asset(path = "objects/chest.glb#Scene0")]
    pub chest: Handle<Scene>,
    #[asset(path = "objects/barrel.glb#Scene0")]
    pub barrel: Handle<Scene>,
    #[asset(path = "objects/burger.glb#Scene0")]
    pub burger: Handle<Scene>,
    #[asset(path = "objects/gun.glb#Scene0")]
    pub gun: Handle<Scene>,
    #[asset(path = "objects/pill.glb#Scene0")]
    pub pill: Handle<Scene>,
    #[asset(path = "objects/screwdriver.glb#Scene0")]
    pub screwdriver: Handle<Scene>,
}

pub enum Sound {
    Death,
    Eat,
    Energized,
    Fart,
    Flush,
    Gunshot,
    LargeHit,
    SadTrombone,
    Siren,
    SmallHit,
    Strange,
}

#[derive(Resource, AssetCollection)]
pub struct SoundList {
    #[asset(path = "sounds/correct.ogg")]
    pub correct: Handle<AudioSource>,
    #[asset(path = "sounds/death.ogg")]
    pub death: Handle<AudioSource>,
    #[asset(path = "sounds/eat.ogg")]
    pub eat: Handle<AudioSource>,
    #[asset(path = "sounds/energized.ogg")]
    pub energized: Handle<AudioSource>,
    #[asset(path = "sounds/fart.ogg")]
    pub fart: Handle<AudioSource>,
    #[asset(path = "sounds/flush.ogg")]
    pub flush: Handle<AudioSource>,
    #[asset(path = "sounds/gunshot.ogg")]
    pub gunshot: Handle<AudioSource>,
    #[asset(path = "sounds/large_hit.ogg")]
    pub large_hit: Handle<AudioSource>,
    #[asset(path = "sounds/nuke_siren.ogg")]
    pub nuke_siren: Handle<AudioSource>,
    #[asset(path = "sounds/sad_trombone.ogg")]
    pub sad_trombone: Handle<AudioSource>,
    #[asset(path = "sounds/siren.ogg")]
    pub siren: Handle<AudioSource>,
    #[asset(path = "sounds/small_hit.ogg")]
    pub small_hit: Handle<AudioSource>,
    #[asset(path = "sounds/strange.ogg")]
    pub strange: Handle<AudioSource>,
    #[asset(path = "sounds/win_music.ogg")]
    pub win_music: Handle<AudioSource>,
}

#[derive(Default, Resource)]
pub struct Chests(pub HashMap<(i32, i32), (Entity, Item)>);
#[derive(Default, Resource)]
pub struct HoveredChest(pub Option<(i32, i32)>);
#[derive(Default, Resource)]
pub struct Instructions(pub VecDeque<Instruction>);
#[derive(Default, Resource)]
pub struct StatusEffects(pub HashMap<StatusEffect, i32>);
#[derive(Default, Resource)]
pub struct NewsFeed(pub VecDeque<RichText>);
#[derive(Default, Resource)]
pub struct ActiveItem(pub Option<(Item, Entity)>);
#[derive(Default, Resource)]
pub struct RequestedItem(pub Option<(String, Item)>);
#[derive(Default, Resource)]
pub struct PrevRequestedItem(pub Option<Item>);
#[derive(Default, Resource)]
pub struct CustomerNumber(pub i32);
#[derive(Default, Resource)]
pub struct GlobalNews(pub VecDeque<String>);
#[derive(Default, Resource)]
pub struct WarNews(pub VecDeque<String>);
#[derive(Default, Resource)]
pub struct War(pub bool);
#[derive(Default, Resource)]
pub struct Win(pub bool);
#[derive(Clone, Copy, Default, Resource)]
pub struct Money(i32);

impl Money {
    pub fn new(x: i32) -> Self {
        Money(x)
    }
}
impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}
impl AddAssign<Money> for Money {
    fn add_assign(&mut self, rhs: Money) {
        self.0 += rhs.0;
    }
}
impl SubAssign<Money> for Money {
    fn sub_assign(&mut self, rhs: Money) {
        self.0 -= rhs.0;
    }
}

pub fn init_resources(app: &mut App) {
    app.init_resource::<Chests>()
        .init_resource::<HoveredChest>()
        .init_resource::<Instructions>()
        .init_resource::<StatusEffects>()
        .init_resource::<NewsFeed>()
        .init_resource::<ActiveItem>()
        .init_resource::<RequestedItem>()
        .init_resource::<PrevRequestedItem>()
        .init_resource::<CustomerNumber>()
        .init_resource::<GlobalNews>()
        .init_resource::<WarNews>()
        .init_resource::<War>()
        .init_resource::<Win>()
        .insert_resource::<Money>(Money(1000));
}
