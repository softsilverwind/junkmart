use bevy::prelude::*;
use bevy_tweening::Animator;

use crate::states::{
    GameState,
    play::{
        components::Rotate,
        events::{NewsFeedUpdate, NewsLevel},
        resources::{Instructions, Chests, ActiveItem, AssetList, RequestedItem, Money, StatusEffects},
        utils::{self, item::Item, StatusEffect}
    }
};

#[derive(Debug)]
pub enum Instruction
{
    Wait(f32),
    SwapWithFirst((i32, i32)),
    CameraToFirstChest,
    CameraToRest,
    PresentItem,
    HideItem,
    HandleEffects,
    EndOfTurn
}

pub fn add_instruction_systems(app: &mut App)
{
    app
        .add_system(wait.in_set(OnUpdate(GameState::Play)))
        .add_system(swap_with_first.in_set(OnUpdate(GameState::Play)))
        .add_system(camera_to_first.in_set(OnUpdate(GameState::Play)))
        .add_system(camera_to_rest.in_set(OnUpdate(GameState::Play)))
        .add_system(present_item.in_set(OnUpdate(GameState::Play)))
        .add_system(hide_item.in_set(OnUpdate(GameState::Play)))
        .add_system(handle_effects.in_set(OnUpdate(GameState::Play)))
        .add_system(end_of_turn.in_set(OnUpdate(GameState::Play)))
    ;
}

fn wait(
    mut instructions: ResMut<Instructions>,
    time: Res<Time>
)
{
    let Some(Instruction::Wait(remaining)) = instructions.0.front_mut() else { return };

    *remaining -= time.delta_seconds();
    if *remaining <= 0.0 {
        instructions.0.pop_front();
    }
}

fn swap_with_first(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    mut chests: ResMut<Chests>,
    query: Query<&Transform>
)
{
    let Some(Instruction::SwapWithFirst(pos)) = instructions.0.front() else { return };

    let chest1 = chests.0[&(0, 0)].clone();
    let chest2 = chests.0[&pos].clone();

    let pos1 = query.get(chest1.0).unwrap().translation;
    let pos2 = query.get(chest2.0).unwrap().translation;

    commands.entity(chest1.0).insert(Animator::new(utils::tween::move_between(pos1, Vec3 { z: 0.0, ..pos2 }, 2.4)));
    commands.entity(chest2.0).insert(Animator::new(utils::tween::move_between(pos2, Vec3 { z: 0.0, ..pos1 }, 1.2)));

    *chests.0.get_mut(&(0, 0)).unwrap() = chest2;
    *chests.0.get_mut(&pos).unwrap() = chest1;

    *instructions.0.front_mut().unwrap() = Instruction::Wait(1.0);
}

fn camera_to_first(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    cameras: Query<Entity, With<Camera>>
)
{
    let Some(Instruction::CameraToFirstChest) = instructions.0.front() else { return };
    let camera = cameras.get_single().unwrap();

    commands.entity(camera).insert(Animator::new(utils::tween::camera_to_first_chest()));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(1.0);
}

fn camera_to_rest(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    cameras: Query<Entity, With<Camera>>
)
{
    let Some(Instruction::CameraToRest) = instructions.0.front() else { return };
    let camera = cameras.get_single().unwrap();

    commands.entity(camera).insert(Animator::new(utils::tween::camera_to_rest()));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(1.0);
}

fn present_item(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    mut active_item: ResMut<ActiveItem>,
    chests: Res<Chests>,
    requested_item: Res<RequestedItem>,
    assets: Res<AssetList>,
)
{
    let Some(Instruction::PresentItem) = instructions.0.front() else { return };

    if let Some((_, entity)) = active_item.0 {
        bevy::log::warn!("This should never happen! Proceeding nevertheless");
        commands.entity(entity).despawn_recursive();
    }

    let new_item = if requested_item.0.is_some() { chests.0[&(0, 0)].1 } else { Item::Gun };
    let scene = match new_item {
        Item::Barrel => assets.barrel.clone(),
        Item::Burger => assets.burger.clone(),
        Item::Gun => assets.gun.clone(),
        Item::Pill => assets.pill.clone(),
        Item::Screwdriver => assets.screwdriver.clone(),
    };
    let init_pos = Vec3::new(-2.4, -1.8, 0.0);

    let id = commands.spawn(SceneBundle {
            scene,
            transform: Transform::from_xyz(-2.4, -1.8, 0.0),
            ..default()
        })
        .insert(Animator::new(utils::tween::lift(init_pos, 1.2, 2000)))
        .insert(Rotate)
        .id();

    active_item.0 = Some((new_item, id));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(2.0);
}

fn hide_item(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    active_item: Res<ActiveItem>
)
{
    let Some(Instruction::HideItem) = instructions.0.front() else { return };

    let Some((_, entity)) = active_item.0 else { bevy::log::error!("Unreachable point reached!"); return };

    let init_pos = Vec3::new(-2.4, -1.8, 1.2);

    commands.entity(entity).insert(Animator::new(utils::tween::lift(init_pos, 0.0, 2000)));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(2.0);
}

fn handle_effects(
    mut instructions: ResMut<Instructions>,
    active_item: Res<ActiveItem>,
    requested_item: Res<RequestedItem>,
    mut ev_news: EventWriter<NewsFeedUpdate>,
    mut money: ResMut<Money>,
    mut status_effects: ResMut<StatusEffects>
)
{
    let Some(Instruction::HandleEffects) = instructions.0.front() else { return };

    let Some((item, _)) = active_item.0 else { bevy::log::error!("Unreachable point reached!"); return };

    let Some(req_item) = &requested_item.0 else {
        ev_news.send(NewsFeedUpdate(NewsLevel::WRONG, "Oh no, the firearm discharged in your hands and T. Utorial lies dead inside a pool of blood...".to_string()));
        ev_news.send(NewsFeedUpdate(NewsLevel::WRONG, "After cleaning up, the realization hits you like that bullet hit Mr Utorial - you are on your own!".to_string()));
        ev_news.send(NewsFeedUpdate(NewsLevel::WRONG, "Unfortunately, you had to reshuffle the boxes in order to clean up, so don't expect a gun in the lower left box... Just saying.".to_string()));
        *instructions.0.front_mut().unwrap() = Instruction::Wait(2.0);

        return;
    };

    let mut response = String::new();

    let level = if req_item.1 == item {
        let gain = item.gain();
        *money += gain;

        response += &format!("Success! You found a box of {}, as the customer requested! He paid you {}!", item.found(), gain);
        NewsLevel::CORRECT
    } else {
        let (text, side_effect) = item.side_effect();

        response += &format!("Customer requested {}, but you found {} instead! ", req_item.0, item.found());
        response += &text;

        match side_effect {
            utils::SideEffect::NoEffect => (),
            utils::SideEffect::MoneyLoss(sum) => *money -= sum,
            utils::SideEffect::StatusEffectEnable(effect, turns) => { status_effects.0.insert(effect, turns); },
            utils::SideEffect::CureDiarrhea => if status_effects.0.remove(&StatusEffect::Diarrhea).is_some() {
                response += " Your diarrhea was cured!";
            },
            utils::SideEffect::ToggleCancer => {
                if status_effects.0.remove(&StatusEffect::Cancer).is_some() {
                    response += "The radiation cured your cancer!";
                }
                else {
                    status_effects.0.insert(StatusEffect::Cancer, i32::MAX);
                    response += "You got cancer! You probably won't find out before 5 years pass, though.";
                }

            }
            utils::SideEffect::CustomerKill => (),
            utils::SideEffect::Reshuffle => (),
        }
        NewsLevel::WRONG
    };

    response += &format!(" Your new balance is {}.", *money);

    ev_news.send(NewsFeedUpdate(level, response));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(2.0);
}

fn end_of_turn(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    mut active_item: ResMut<ActiveItem>,
    mut requested_item: ResMut<RequestedItem>,
    mut ev_news: EventWriter<NewsFeedUpdate>
)
{
    let Some(Instruction::EndOfTurn) = instructions.0.front() else { return };

    let Some((_, entity)) = active_item.0 else { bevy::log::error!("Unreachable point reached!"); return };

    commands.entity(entity).despawn_recursive();
    active_item.0 = None;

    let new_item = Item::new_random();
    let request_str = new_item.request();
    requested_item.0 = Some((request_str.to_string(), new_item));

    ev_news.send(NewsFeedUpdate(NewsLevel::EVENT, format!("A customer just arrived! They requested {}.", request_str)));

    instructions.0.pop_front();
}
