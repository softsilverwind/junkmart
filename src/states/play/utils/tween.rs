use std::{
    f32::consts::FRAC_PI_2,
    iter,
    time::Duration
};

use bevy::prelude::*;
use bevy_tweening::{*, lens::*};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CAMERA_REST_POS: Transform = Transform::from_xyz(0.0, -8.0, 17.0).looking_at(Vec3::ZERO, Vec3::Y);
    pub static ref CAMERA_FIRST_CHEST_POS: Transform = Transform::from_xyz(-2.4, -5.0, 1.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2));
}

pub fn lift(start: Vec3, height: f32, duration: u64) -> impl Tweenable<Transform>
{
    let end = Vec3 { z: height, ..start };

    Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(duration),
        TransformPositionLens {
            start,
            end
        }
    )
}

pub fn move_between(start: Vec3, end: Vec3, max_height: f32) -> impl Tweenable<Transform>
{
    let above_start = Vec3 { z: max_height, ..start };
    let above_end = Vec3 { z: max_height, ..end };

    Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(200),
        TransformPositionLens {
            start,
            end: above_start
        }
    )
    .then(Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(500),
        TransformPositionLens {
            start: above_start,
            end: above_end
        }
    ))
    .then(Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(200),
        TransformPositionLens {
            start: above_end,
            end
        }
    ))
}

pub fn camera_to_first_chest() -> impl Tweenable<Transform>
{
    let trans = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(1000),
        TransformPositionLens {
            start: CAMERA_REST_POS.translation,
            end: CAMERA_FIRST_CHEST_POS.translation
        }
    );

    let rot = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(1000),
        TransformRotationLens {
            start: CAMERA_REST_POS.rotation,
            end: CAMERA_FIRST_CHEST_POS.rotation
        }
    );

    Tracks::new( iter::once(trans).chain(iter::once(rot)) )
}

pub fn camera_to_rest() -> impl Tweenable<Transform>
{
    let trans = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(1000),
        TransformPositionLens {
            start: CAMERA_FIRST_CHEST_POS.translation,
            end: CAMERA_REST_POS.translation
        }
    );

    let rot = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(1000),
        TransformRotationLens {
            start: CAMERA_FIRST_CHEST_POS.rotation,
            end: CAMERA_REST_POS.rotation
        }
    );

    Tracks::new( iter::once(trans).chain(iter::once(rot)) )
}

