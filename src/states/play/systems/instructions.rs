use std::iter;

use bevy::prelude::*;
use bevy_tweening::Animator;

use crate::states::{
    GameState,
    play::{
        components::Rotate,
        events::{NewsFeedUpdate, NewsLevel},
        resources::{Instructions, Chests, ActiveItem, AssetList, RequestedItem, Money, StatusEffects, Turn, PrevRequestedItem, GlobalNews, WarNews},
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
    mut requested_item: ResMut<RequestedItem>,
    mut prev_item: ResMut<PrevRequestedItem>,
    mut ev_news: EventWriter<NewsFeedUpdate>,
    mut money: ResMut<Money>,
    mut status_effects: ResMut<StatusEffects>,
    mut chests: ResMut<Chests>,
    mut turn: ResMut<Turn>,
    mut war_news: ResMut<WarNews>,
)
{
    let Some(Instruction::HandleEffects) = instructions.0.front() else { return };

    let Some((item, _)) = active_item.0 else { bevy::log::error!("Unreachable point reached!"); return };

    let Some(req_item) = &requested_item.0 else {
        let funeral_cost = Money::new(5000);
        *money -= funeral_cost;
        ev_news.send(NewsFeedUpdate(NewsLevel::WRONG, "Oh no, the firearm discharged in your hands and T. Utorial lies dead inside a pool of blood...".to_string()));
        ev_news.send(NewsFeedUpdate(NewsLevel::WRONG, "After cleaning up, the realization hits you like that bullet hit Mr Utorial - you are on your own!".to_string()));
        ev_news.send(NewsFeedUpdate(NewsLevel::WRONG, format!("Cleaning up messed the boxes, while the funeral cost {funeral_cost}. Your new balance is {}.", *money)));
        *instructions.0.front_mut().unwrap() = Instruction::Wait(2.0);

        return;
    };

    let mut response = String::new();

    let level = if req_item.1 == item {
        let gain = item.gain();
        *money += gain;

        response += &format!("Success! You found a box of {}, as the customer requested! They paid you {}!", item.found(), gain);
        response += &format!(" Your new balance is {}.", *money);
        prev_item.0 = Some(item);
        requested_item.0 = None;
        if turn.0 == 21 {
            war_news.0.push_back("Dirty bomb exploded in nearby large city, countries blame each other!".to_string());
            war_news.0.push_back("Our country retaliates with nukes!".to_string());
            war_news.0.push_back("World war! Every country launches nukes to everyone!".to_string());
            war_news.0.push_back("Seriously, stop playing. You destroyed the world. Mankind is not the same anymore. You are a millionaire in a world where money has no meaning. Sleep tight.".to_string());
 
            turn.0 += 1;
        }
        NewsLevel::CORRECT
    } else {
        let (text, side_effect) = item.side_effect();

        response += &format!("Customer requested {}, but you found {} instead! ", req_item.0, item.found());
        response += &text;

        match side_effect {
            utils::SideEffect::NoEffect => (),
            utils::SideEffect::MoneyLoss(sum) => {
                *money -= sum;
                response += &format!(" Your new balance is {}.", *money);
            }
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
            utils::SideEffect::CustomerKill => requested_item.0 = None,
            utils::SideEffect::Reshuffle => {
                let mut items: Vec<Item> =
                    iter::once(Item::Barrel).cycle().take(2)
                    .chain(iter::once(Item::Burger).cycle().take(6))
                    .chain(iter::once(Item::Gun).cycle().take(3))
                    .chain(iter::once(Item::Pill).cycle().take(3))
                    .chain(iter::once(Item::Screwdriver).cycle().take(6))
                    .collect();

                    for x in 0..5 {
                        for y in 0..4 {
                            chests.0.get_mut(&(x, y)).unwrap().1 = items.pop().unwrap();
                        }
                    }
            },
        }
        NewsLevel::WRONG
    };


    ev_news.send(NewsFeedUpdate(level, response));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(2.0);
}

fn end_of_turn(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    mut active_item: ResMut<ActiveItem>,
    mut requested_item: ResMut<RequestedItem>,
    mut prev_item: ResMut<PrevRequestedItem>,
    mut ev_news: EventWriter<NewsFeedUpdate>,
    mut global_news: ResMut<GlobalNews>,
    mut war_news: ResMut<WarNews>,
    mut turn: ResMut<Turn>
)
{
    let Some(Instruction::EndOfTurn) = instructions.0.front() else { return };

    let Some((_, entity)) = active_item.0 else { bevy::log::error!("Unreachable point reached!"); return };

    commands.entity(entity).despawn_recursive();
    active_item.0 = None;

    if let Some(gnews) = global_news.0.pop_front() {
        ev_news.send(NewsFeedUpdate(NewsLevel::EXTERNAL, gnews));
    }
    if let Some(gnews) = war_news.0.pop_front() {
        ev_news.send(NewsFeedUpdate(NewsLevel::EXTERNAL, gnews));
    }

    if let Some((request_str, _)) = &requested_item.0 {
        ev_news.send(NewsFeedUpdate(NewsLevel::EVENT, format!("The customer is still waiting for {}.", request_str)));
    }
    else {
        if turn.0 != 21 { // Turn 21 advances only on success
            turn.0 += 1;
        }

        let new_item = Item::new_random(turn.0, prev_item.0);

        if let Some(gnews) = prev_item.0.and_then(|item| item.global_side_effect()) {
            global_news.0.push_back(gnews.to_string());
        }

        prev_item.0 = None;
        let request_str = new_item.request();
        requested_item.0 = Some((request_str.to_string(), new_item));

        if turn.0 == 6 {
            ev_news.send(NewsFeedUpdate(NewsLevel::EVENT, format!(r#""I won't cover the debt by just selling the legal stuff". "I should probably advertise other stuff"."#)));
        }

        if turn.0 == 21 {
            ev_news.send(NewsFeedUpdate(NewsLevel::EVENT, format!("Turn {}/20: A shady figure just arrived! They requested {}.", turn.0, request_str)));
        }
        else {
            ev_news.send(NewsFeedUpdate(NewsLevel::EVENT, format!("Turn {}/20: A new customer just arrived! They requested {}.", turn.0, request_str)));
        }
    }

    instructions.0.pop_front();
}
