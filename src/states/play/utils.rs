use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{*, lens::*};

pub fn lift(start: Vec3, height: f32) -> impl Tweenable<Transform>
{
    let end = Vec3 { z: height, ..start };

    Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(200),
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
