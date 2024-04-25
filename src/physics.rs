use macroquad::{
    math::{vec2, Vec2},
    time::get_frame_time,
};
use specs::{Component, VecStorage};
use std::ops::{Add, Mul};
#[derive(Clone)]
pub enum AccelerationType {
    Constant,
    Linear,
}
#[derive(Clone)]
pub struct RigidBody {
    pub acceleration: Vec2,
    pub velocity: Vec2,
    pub position: Vec2,
    pub acceleration_type: AccelerationType,
    pub update: i32,
}
impl Component for RigidBody {
    type Storage = VecStorage<Self>;
}
impl Default for RigidBody {
    fn default() -> Self {
        RigidBody {
            acceleration: vec2(0.0, 0.0),
            velocity: vec2(0.0, 0.0),
            position: vec2(0.0, 0.0),
            acceleration_type: AccelerationType::Constant,
            update: 1,
        }
    }
}

pub fn update_bodies(mut bodies: Vec<RigidBody>) -> Vec<RigidBody> {
    let mut owned_vec = Vec::new();
    for body in bodies.iter_mut() {
        match body.acceleration_type {
            AccelerationType::Linear => {
                body.acceleration += (body.acceleration.mul(1.0 / body.update as f32));
            }
            _ => {}
        }
        body.velocity = body.velocity.add(body.acceleration * get_frame_time());
        body.position = body.position.add(body.velocity * get_frame_time());
        body.update += 1;
        owned_vec.push(body.clone());
    }
    owned_vec
}
