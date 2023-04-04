use bevy::prelude::*;
use bevy_tweening::Animator;

use crate::states::play::{
    resources::{Instruction, Instructions, Chests},
    utils
};

pub fn swap_with_first(
    mut commands: Commands,
    mut instructions: ResMut<Instructions>,
    mut chests: ResMut<Chests>,
    query: Query<&Transform>
)
{
    if let Some(Instruction::SwapWithFirst(pos)) = instructions.0.front() {
        let chest1 = chests.0[&(0, 0)].clone();
        let chest2 = chests.0[&pos].clone();

        let pos1 = query.get(chest1.0).unwrap().translation;
        let pos2 = query.get(chest2.0).unwrap().translation;

        commands.entity(chest1.0).insert(Animator::new(utils::move_between(pos1, Vec3 { z: 0.0, ..pos2 }, 1.2)));
        commands.entity(chest2.0).insert(Animator::new(utils::move_between(pos2, Vec3 { z: 0.0, ..pos1 }, 2.4)));

        *chests.0.get_mut(&(0, 0)).unwrap() = chest2;
        *chests.0.get_mut(&pos).unwrap() = chest1;
        instructions.0.pop_front();
    }
}

pub fn wait(
    mut instructions: ResMut<Instructions>,
    time: Res<Time>
)
{
    if let Some(Instruction::Wait(remaining)) = instructions.0.front_mut() {
        *remaining -= time.delta_seconds();

        if *remaining <= 0.0 {
            instructions.0.pop_front();
        }
    }
}
