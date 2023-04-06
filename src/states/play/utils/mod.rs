use std::f32::consts::FRAC_PI_2;
use bevy::prelude::*;
use lazy_static::lazy_static;

use super::resources::Money;

pub mod item;
pub mod tween;

lazy_static! {
    pub static ref CAMERA_REST_POS: Transform = Transform::from_xyz(0.0, -8.0, 17.0).looking_at(Vec3::ZERO, Vec3::Y);
    pub static ref CAMERA_FIRST_CHEST_POS: Transform = Transform::from_xyz(-2.4, -5.0, 1.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2));
    pub static ref POINT_LIGHT: PointLight = PointLight {
        color: Color::WHITE,
        intensity: 40.0,
        range: 10.0,
        radius: 0.5,
        shadows_enabled: true,
        ..default()
    };
    pub static ref POINTER_LIGHT: PointLight = PointLight {
        color: Color::BEIGE,
        intensity: 10.0,
        range: 10.0,
        radius: 0.5,
        shadows_enabled: true,
        ..default()
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect
{
    LightsOut,
    Trippy,
    Diarrhea,
    Cancer
}

pub enum SideEffect
{
    NoEffect,
    MoneyLoss(Money),
    StatusEffectEnable(StatusEffect, i32),
    CureDiarrhea,
    ToggleCancer,
    CustomerKill,
    Reshuffle
}
