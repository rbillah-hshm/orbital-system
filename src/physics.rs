use macroquad::{
    math::{vec2, Vec2},
    time::get_frame_time,
};
use specs::{Component, VecStorage};
use std::ops::{Add, Mul};
#[derive(Clone)]
pub struct RigidBody {
    pub mass: Option<f32>,
    pub forces: Vec<Vec2>,

    pub acceleration: Vec2,
    pub velocity: Vec2,
    pub position: Vec2,
}
impl Component for RigidBody {
    type Storage = VecStorage<Self>;
}
impl Default for RigidBody {
    fn default() -> Self {
        RigidBody {
            mass: None,
            forces: Vec::new(),

            acceleration: vec2(0.0, 0.0),
            velocity: vec2(0.0, 0.0),
            position: vec2(0.0, 0.0),
        }
    }
}
pub fn update_bodies(mut bodies: Vec<RigidBody>) -> Vec<RigidBody> {
    let mut owned_vec = Vec::new();
    for body in bodies.iter_mut() {
        let mut sum_force = vec2(0.0, 0.0);
        for force in body.forces.iter() {
            sum_force = sum_force.add(*force);
        }
        let mass = match body.mass {
            Some(x) => x,
            None => 1.0,
        };
        body.acceleration = body.acceleration.add(sum_force.mul((1.0 / mass)));
        body.velocity = body.velocity.add(body.acceleration * get_frame_time());
        body.position = body.position.add(body.velocity * get_frame_time());
        owned_vec.push(body.clone());
    }
    owned_vec
}
