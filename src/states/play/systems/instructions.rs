use bevy::{prelude::*, utils::HashMap};
use bevy_kira_audio::prelude::*;
use bevy_tweening::Animator;
use rand::{prelude::*, seq::SliceRandom};

use crate::{
    plugins::post_process::{PostProcessConfig, PostProcessingMaterial},
    states::{
        play::{
            components::{PointerLight, Rotate},
            events::{NewsFeedUpdate, NewsLevel},
            resources::{
                ActiveItem, AssetList, Chests, CustomerNumber, GlobalNews, Instructions, Money,
                PrevRequestedItem, RequestedItem, Sound, SoundList, StatusEffects, War, WarNews,
                Win,
            },
            utils::{self, item::Item, StatusEffect},
        },
        GameState,
    },
};

#[derive(Debug)]
pub enum Instruction {
    Wait(f32),
    SwapWithFirst((i32, i32)),
    CameraToFirstChest,
    CameraToRest,
    PresentItem,
    HideItem,
    HandleEffects,
    HandleStatusEffects,
    EndOfTurn,
}

pub fn add_instruction_systems(app: &mut App) {
    app.add_system(wait.in_set(OnUpdate(GameState::Play)))
        .add_system(swap_with_first.in_set(OnUpdate(GameState::Play)))
        .add_system(camera_to_first.in_set(OnUpdate(GameState::Play)))
        .add_system(camera_to_rest.in_set(OnUpdate(GameState::Play)))
        .add_system(present_item.in_set(OnUpdate(GameState::Play)))
        .add_system(hide_item.in_set(OnUpdate(GameState::Play)))
        .add_system(handle_status_effects.in_set(OnUpdate(GameState::Play)))
        .add_system(handle_effects.in_set(OnUpdate(GameState::Play)))
        .add_system(end_of_turn.in_set(OnUpdate(GameState::Play)));
}

fn wait(mut instructions: ResMut<Instructions>, time: Res<Time>) {
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
    query: Query<&Transform>,
) {
    let Some(Instruction::SwapWithFirst(pos)) = instructions.0.front() else { return };

    let chest1 = chests.0[&(0, 0)];
    let chest2 = chests.0[pos];

    let pos1 = query.get(chest1.0).unwrap().translation;
    let pos2 = query.get(chest2.0).unwrap().translation;

    commands
        .entity(chest1.0)
        .insert(Animator::new(utils::tween::move_between(
            pos1,
            Vec3 { z: 0.0, ..pos2 },
            2.4,
        )));
    commands
        .entity(chest2.0)
        .insert(Animator::new(utils::tween::move_between(
            pos2,
            Vec3 { z: 0.0, ..pos1 },
            1.2,
        )));

    *chests.0.get_mut(&(0, 0)).unwrap() = chest2;
    *chests.0.get_mut(pos).unwrap() = chest1;

    *instructions.0.front_mut().unwrap() = Instruction::Wait(1.0);
}

fn camera_to_first(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    cameras: Query<Entity, With<Camera3d>>,
) {
    let Some(Instruction::CameraToFirstChest) = instructions.0.front() else { return };
    let camera = cameras.single();

    commands
        .entity(camera)
        .insert(Animator::new(utils::tween::camera_to_first_chest()));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(1.0);
}

fn camera_to_rest(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    cameras: Query<Entity, With<Camera3d>>,
) {
    let Some(Instruction::CameraToRest) = instructions.0.front() else { return };
    let camera = cameras.single();

    commands
        .entity(camera)
        .insert(Animator::new(utils::tween::camera_to_rest()));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(1.0);
}

fn present_item(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    mut active_item: ResMut<ActiveItem>,
    chests: Res<Chests>,
    requested_item: Res<RequestedItem>,
    assets: Res<AssetList>,
) {
    let Some(Instruction::PresentItem) = instructions.0.front() else { return };

    if let Some((_, entity)) = active_item.0 {
        bevy::log::warn!("This should never happen! Proceeding nevertheless");
        commands.entity(entity).despawn_recursive();
    }

    let new_item = if requested_item.0.is_some() {
        chests.0[&(0, 0)].1
    } else {
        Item::Gun
    };
    let scene = match new_item {
        Item::Barrel => assets.barrel.clone(),
        Item::Burger => assets.burger.clone(),
        Item::Gun => assets.gun.clone(),
        Item::Pill => assets.pill.clone(),
        Item::Screwdriver => assets.screwdriver.clone(),
    };
    let init_pos = Vec3::new(-2.4, -1.8, 0.0);

    let id = commands
        .spawn(SceneBundle {
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
    active_item: Res<ActiveItem>,
) {
    let Some(Instruction::HideItem) = instructions.0.front() else { return };

    let Some((_, entity)) = active_item.0 else { bevy::log::error!("Unreachable point reached!"); return };

    let init_pos = Vec3::new(-2.4, -1.8, 1.2);

    commands
        .entity(entity)
        .insert(Animator::new(utils::tween::lift(init_pos, 0.0, 2000)));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(2.0);
}

#[allow(clippy::too_many_arguments)]
fn handle_effects(
    mut instructions: ResMut<Instructions>,
    active_item: Res<ActiveItem>,
    mut requested_item: ResMut<RequestedItem>,
    mut prev_item: ResMut<PrevRequestedItem>,
    mut ev_news: EventWriter<NewsFeedUpdate>,
    mut money: ResMut<Money>,
    mut status_effects: ResMut<StatusEffects>,
    mut war_news: ResMut<WarNews>,
    mut war: ResMut<War>,
    audio: Res<Audio>,
    sounds: Res<SoundList>,
) {
    let Some(Instruction::HandleEffects) = instructions.0.front() else { return };

    let Some((item, _)) = active_item.0 else { bevy::log::error!("Unreachable point reached!"); return };

    let Some(req_item) = &requested_item.0 else {
        let funeral_cost = Money::new(5000);
        *money -= funeral_cost;
        audio.play(sounds.gunshot.clone());
        audio.play(sounds.large_hit.clone());
        ev_news.send(NewsFeedUpdate(NewsLevel::Wrong, "Oh no, the firearm discharged in your hands and T. Utorial lies dead inside a pool of blood...".to_string()));
        ev_news.send(NewsFeedUpdate(NewsLevel::Wrong, "After cleaning up, the realization hits you like that bullet hit Mr Utorial - you are on your own!".to_string()));
        ev_news.send(NewsFeedUpdate(NewsLevel::Wrong, format!("Cleaning up messed the boxes, while the funeral cost {funeral_cost}. Your new balance is {}.", *money)));
        status_effects.0.insert(StatusEffect::Reshuffle, 1);
        *instructions.0.front_mut().unwrap() = Instruction::Wait(2.0);

        return;
    };

    let mut response = String::new();

    let level = if req_item.1 == item {
        let gain = item.gain();
        *money += gain;
        audio.play(sounds.correct.clone());

        response += &format!(
            "Success! You found a box of {}, as the customer requested! They paid you {}!",
            item.found(),
            gain
        );
        response += &format!(" Your new balance is {}.", *money);
        prev_item.0 = Some(item);
        if item == Item::Barrel {
            war_news.0.push_back(
                "Dirty bomb exploded in the capital of neigboring country, they blame our army!"
                    .to_string(),
            );
            war_news
                .0
                .push_back("Our country retaliates with nukes! For the motherland!".to_string());
            war_news
                .0
                .push_back("World war! Every country launches nukes to everyone!".to_string());
            war_news.0.push_back("Seriously, stop playing. You won, but you destroyed the world in the process. Mankind is not the same anymore. You are a millionaire in a world where money has no meaning. Sleep tight.".to_string());

            war.0 = true;
        }
        requested_item.0 = None;
        NewsLevel::Correct
    } else {
        let (text, side_effect, sound) = item.side_effect();
        for s in sound {
            match s {
                Sound::Death => {
                    audio.play(sounds.death.clone());
                }
                Sound::Eat => {
                    audio.play(sounds.eat.clone());
                }
                Sound::Energized => {
                    audio.play(sounds.energized.clone());
                }
                Sound::Fart => {
                    audio.play(sounds.fart.clone());
                }
                Sound::Flush => {
                    audio.play(sounds.flush.clone());
                }
                Sound::Gunshot => {
                    audio.play(sounds.gunshot.clone());
                }
                Sound::LargeHit => {
                    audio.play(sounds.large_hit.clone());
                }
                Sound::SadTrombone => {
                    audio.play(sounds.sad_trombone.clone());
                }
                Sound::Siren => {
                    audio.play(sounds.siren.clone());
                }
                Sound::SmallHit => {
                    audio.play(sounds.small_hit.clone());
                }
                Sound::Strange => {
                    audio.play(sounds.strange.clone());
                }
            }
        }

        response += &format!(
            "Customer requested {}, but you found {} instead! ",
            req_item.0,
            item.found()
        );
        response += &text;

        match side_effect {
            utils::SideEffect::NoEffect => (),
            utils::SideEffect::MoneyLoss(sum) => {
                *money -= sum;
                response += &format!(" Your new balance is {}.", *money);
            }
            utils::SideEffect::StatusEffectEnable(effect, turns) => {
                status_effects.0.insert(effect, turns);
            }
            utils::SideEffect::CureDiarrhea => {
                if status_effects.0.remove(&StatusEffect::Diarrhea).is_some() {
                    audio.play(sounds.correct.clone());
                    response += " Your diarrhea was cured! The power of Imodium will turn the hands of fate!";
                }
            }
            utils::SideEffect::ToggleCancer => {
                if status_effects.0.remove(&StatusEffect::Cancer).is_some() {
                    audio.play(sounds.correct.clone());
                    response += "The radiation cured your cancer!";
                } else {
                    status_effects.0.insert(StatusEffect::Cancer, i32::MAX);
                    response +=
                        "You got cancer! You probably won't find out before 5 years pass, though.";
                }
            }
            utils::SideEffect::CustomerKill => requested_item.0 = None,
        }
        NewsLevel::Wrong
    };

    ev_news.send(NewsFeedUpdate(level, response));

    *instructions.0.front_mut().unwrap() = Instruction::Wait(2.0);
}

#[allow(clippy::too_many_arguments)]
fn handle_status_effects(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    mut status_effects: ResMut<StatusEffects>,
    mut active_item: ResMut<ActiveItem>,
    mut ev_news: EventWriter<NewsFeedUpdate>,
    mut requested_item: ResMut<RequestedItem>,
    mut lights: Query<&mut PointLight, (Without<Camera>, Without<PointerLight>)>,
    mut pointer_light: Query<&mut PointLight, With<PointerLight>>,
    mut post_processing_materials: ResMut<Assets<PostProcessingMaterial>>,
    mut chests: ResMut<Chests>,
    post_process_config: Res<PostProcessConfig>,
    audio: Res<Audio>,
    sounds: Res<SoundList>,
) {
    let Some(Instruction::HandleStatusEffects) = instructions.0.front() else { return };

    let Some((_, entity)) = active_item.0 else { bevy::log::error!("Unreachable point reached!"); return };

    commands.entity(entity).despawn_recursive();
    active_item.0 = None;

    let mut wait = 0.0;

    let mut rng = thread_rng();

    let mut deletions = Vec::new();
    for (status_effect, turns) in status_effects.0.iter_mut() {
        *turns -= 1;
        if *turns < 0 {
            deletions.push(*status_effect);
            continue;
        }

        match status_effect {
            StatusEffect::LightsOut => {
                for mut light in lights.iter_mut() {
                    light.intensity = 0.0;
                }
                pointer_light.single_mut().intensity = utils::POINTER_LIGHT.intensity;
            }
            StatusEffect::Trippy => {
                let post_process_mat = post_processing_materials
                    .get_mut(&post_process_config.material_handle)
                    .unwrap();
                post_process_mat.is_trippy = true;
            }
            StatusEffect::Reshuffle => {
                let positions: Vec<(i32, i32)> = (0..5)
                    .flat_map(move |x| (0..4).map(move |y| (x, y)))
                    .collect();
                let mut positions2 = positions.clone();
                positions2.shuffle(&mut rng);
                let mut new_chests = HashMap::new();

                for ((ox, oy), (nx, ny)) in positions.into_iter().zip(positions2) {
                    let chest = chests.0[&(ox, oy)];
                    new_chests.insert((nx, ny), chest);

                    let from = Vec3::new(-2.4 + ox as f32 * 1.2, -1.8 + oy as f32 * 1.2, 0.0);
                    let to = Vec3::new(-2.4 + nx as f32 * 1.2, -1.8 + ny as f32 * 1.2, 0.0);
                    let height = rng.gen_range(0.0..=2.4);

                    commands
                        .entity(chest.0)
                        .insert(Animator::new(utils::tween::move_between(from, to, height)));
                }

                chests.0 = new_chests;

                wait += 1.0;
            }
            _ => (),
        }
    }

    for del in deletions {
        status_effects.0.remove(&del);
        match del {
            StatusEffect::LightsOut => {
                for mut light in lights.iter_mut() {
                    light.intensity = utils::POINT_LIGHT.intensity;
                }
                pointer_light.single_mut().intensity = 0.0;
                ev_news.send(NewsFeedUpdate(
                    NewsLevel::Event,
                    "Finally, the power is back!".to_string(),
                ));
            }
            StatusEffect::Trippy => {
                let post_process_mat = post_processing_materials
                    .get_mut(&post_process_config.material_handle)
                    .unwrap();
                post_process_mat.is_trippy = false;
                ev_news.send(NewsFeedUpdate(
                    NewsLevel::Event,
                    "Your vision is back to normal!".to_string(),
                ));
            }
            StatusEffect::Diarrhea => {
                ev_news.send(NewsFeedUpdate(
                    NewsLevel::Event,
                    "Your stomach feels better!".to_string(),
                ));
            }
            StatusEffect::Cancer => {
                ev_news.send(NewsFeedUpdate(NewsLevel::Event, format!("You mean to tell me that you played the game for {} turns. Suuuure buddy, sure you did. I'm not mad though, it means one of three things: a)  You scripted the game for {} turns (lol), b) cheated or c) read the source code. In all cases, thank you for giving my little game such interest. You are the real winner of this game, and you may screenshot this text as proof of your achievement!", i32::MAX, i32::MAX).to_string()));
            }
            _ => (),
        }
    }

    if status_effects.0.contains_key(&StatusEffect::Diarrhea) {
        audio.play(sounds.flush.clone());

        if requested_item.0.is_some() {
            if rng.gen_range(0..=10) > 7 {
                requested_item.0 = None;
                ev_news.send(NewsFeedUpdate(NewsLevel::Event,
                    [
                        "The customer is leaving, but the burger needs to return to its people. To the toilet!",
                        "A disgusted customer leaves as you have to rush to the toilet. Again."
                    ].choose(&mut rng).unwrap().to_string()
                ));
            } else {
                ev_news.send(NewsFeedUpdate(
                    NewsLevel::Event,
                    [
                        "You had to go to the toilet! Thankfully, the customer is waiting.",
                        "Emergency toilet run! The customer will listen to all kinds of sounds...",
                    ]
                    .choose(&mut rng)
                    .unwrap()
                    .to_string(),
                ));
            }
        } else {
            ev_news.send(NewsFeedUpdate(
                NewsLevel::Event,
                "The customer left just in time for the toilet instruments to start playing!"
                    .to_string(),
            ));
        }

        wait += 5.0;
    }

    if wait > 0.0 {
        *instructions.0.front_mut().unwrap() = Instruction::Wait(wait);
    } else {
        instructions.0.pop_front();
    }
}

#[allow(clippy::too_many_arguments)]
fn end_of_turn(
    mut instructions: ResMut<Instructions>,
    mut requested_item: ResMut<RequestedItem>,
    mut prev_item: ResMut<PrevRequestedItem>,
    mut ev_news: EventWriter<NewsFeedUpdate>,
    mut global_news: ResMut<GlobalNews>,
    mut war_news: ResMut<WarNews>,
    war: Res<War>,
    mut win: ResMut<Win>,
    mut customer_no: ResMut<CustomerNumber>,
    audio: Res<Audio>,
    sounds: Res<SoundList>,
) {
    let Some(Instruction::EndOfTurn) = instructions.0.front() else { return };

    if let Some(gnews) = global_news.0.pop_front() {
        ev_news.send(NewsFeedUpdate(NewsLevel::External, gnews));
    }
    if let Some(gnews) = war_news.0.pop_front() {
        if war_news.0.len() == 3 {
            audio.play(sounds.gunshot.clone());
        }
        if !war_news.0.is_empty() {
            audio.play(sounds.nuke_siren.clone());
        }
        ev_news.send(NewsFeedUpdate(NewsLevel::External, gnews));
    }

    if !win.0 && war.0 && war_news.0.is_empty() {
        win.0 = true;
        audio.play(sounds.win_music.clone());
    }

    if let Some((request_str, _)) = &requested_item.0 {
        ev_news.send(NewsFeedUpdate(
            NewsLevel::Event,
            format!("The customer is still waiting for {}.", request_str),
        ));
    } else {
        if customer_no.0 != 21 || war.0 {
            // Customer 21 advances only on success
            customer_no.0 += 1;
        }

        let new_item = Item::new_random(customer_no.0, prev_item.0);

        if let Some(gnews) = prev_item.0.and_then(|item| item.global_side_effect()) {
            global_news.0.push_back(gnews.to_string());
        }

        prev_item.0 = None;
        let request_str = new_item.request();
        requested_item.0 = Some((request_str.to_string(), new_item));

        if customer_no.0 == 6 {
            ev_news.send(NewsFeedUpdate(NewsLevel::Event, r#""I won't cover the debt by just selling the legal stuff". "I should probably advertise other stuff"."#.to_string()));
        }

        if customer_no.0 == 21 {
            ev_news.send(NewsFeedUpdate(
                NewsLevel::Event,
                format!(
                    "Turn {}/20: A shady figure just arrived! They requested {}.",
                    customer_no.0, request_str
                ),
            ));
        } else {
            ev_news.send(NewsFeedUpdate(
                NewsLevel::Event,
                format!(
                    "Turn {}/20: A new customer just arrived! They requested {}.",
                    customer_no.0, request_str
                ),
            ));
        }
    }

    instructions.0.pop_front();
}
