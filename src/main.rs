#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_variables)]
extern crate specs;
use big_number::BigNumber;
use big_number::BigVec2;

use macroquad::camera::Camera2D;
use macroquad::color;
use macroquad::color::BLACK;
use macroquad::math::Vec2;
use macroquad::prelude::*;
use macroquad::window::clear_background;
use macroquad::window::next_frame;
use macroquad::window::request_new_screen_size;
use macroquad::window::Conf;
use num::traits::real::Real;
use specs::prelude::*;
mod big_number;
// Hierarchy: Sun => Planet => Moon
// Planets are unscaled for the sake of visualization purposes
trait SpaceObject {
    fn get_position(&self) -> BigVec2;
    fn get_radius(&self) -> f32;
    fn get_color(&self) -> Vec<Color>;
    fn get_current_color(&self) -> Color;
    fn set_current_color(&mut self, color: Color) -> Color;
    fn get_color_elapsed_time(&self) -> f32;
    fn set_color_elapsed_time(&mut self, value: f32) -> f32;
}
#[derive(Debug)]
struct Sun {
    position: BigVec2,
    radius: f32,
    color: Vec<Color>,
    current_color: Color,
    color_elapsed_time: f32,
}
impl SpaceObject for Sun {
    fn get_position(&self) -> BigVec2 {
        self.position.clone()
    }
    fn get_radius(&self) -> f32 {
        self.radius
    }
    fn get_color(&self) -> Vec<Color> {
        self.color.clone()
    }
    fn get_current_color(&self) -> Color {
        self.current_color
    }
    fn set_current_color(&mut self, color: Color) -> Color {
        self.current_color = color;
        color
    }
    fn get_color_elapsed_time(&self) -> f32 {
        self.color_elapsed_time
    }
    fn set_color_elapsed_time(&mut self, value: f32) -> f32 {
        self.color_elapsed_time = value;
        self.color_elapsed_time
    }
}
impl Component for Sun {
    type Storage = VecStorage<Self>;
}
#[derive(Debug)]
struct Planet {
    position: BigVec2,
    radius: f32,
    color: Vec<Color>,
    current_color: Color,
    color_elapsed_time: f32,
}
impl SpaceObject for Planet {
    fn get_position(&self) -> BigVec2 {
        self.position.clone()
    }
    fn get_radius(&self) -> f32 {
        self.radius
    }
    fn get_color(&self) -> Vec<Color> {
        self.color.clone()
    }
    fn get_current_color(&self) -> Color {
        self.current_color
    }
    fn set_current_color(&mut self, color: Color) -> Color {
        self.current_color = color;
        color
    }
    fn get_color_elapsed_time(&self) -> f32 {
        self.color_elapsed_time
    }
    fn set_color_elapsed_time(&mut self, value: f32) -> f32 {
        self.color_elapsed_time = value;
        self.color_elapsed_time
    }
}
impl Component for Planet {
    type Storage = VecStorage<Self>;
}
struct Moon {
    position: BigVec2,
    radius: f32,
    color: Vec<Color>,
    current_color: Color,
    color_elapsed_time: f32,
}
struct IsOrbital {
    moons: Vec<Moon>,
}
impl SpaceObject for Moon {
    fn get_position(&self) -> BigVec2 {
        self.position.clone()
    }
    fn get_radius(&self) -> f32 {
        self.radius
    }
    fn get_color(&self) -> Vec<Color> {
        self.color.clone()
    }
    fn get_current_color(&self) -> Color {
        self.current_color
    }
    fn set_current_color(&mut self, color: Color) -> Color {
        self.current_color = color;
        color
    }
    fn get_color_elapsed_time(&self) -> f32 {
        self.color_elapsed_time
    }
    fn set_color_elapsed_time(&mut self, value: f32) -> f32 {
        self.color_elapsed_time = value;
        self.color_elapsed_time
    }
}
impl Component for IsOrbital {
    type Storage = VecStorage<Self>;
}
fn map_ranges(number: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    c + ((number - a) / (b - a)) * (d - c)
}
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}
mod easing_styles {
    use crate::lerp;
    use std::f32::consts::PI;
    fn flip_across_midline(point: f32, mid_point: f32) -> f32 {
        let difference = mid_point - point;
        let sign = f32::signum(difference);
        mid_point + (sign * difference.abs())
    }
    pub mod in_direction {
        use crate::easing_styles::flip_across_midline;

        pub fn sine(a: f32, b: f32, t: f32) -> f32 {
            let alpha = super::flip_across_midline(
                f32::sin(super::flip_across_midline(
                    t * (super::PI / 2.0),
                    super::PI / 4.0,
                )),
                0.5,
            );
            super::lerp(a, b, alpha)
        }
        pub fn circular(a: f32, b: f32, t: f32) -> f32 {
            let half_pi = super::PI / 2.0;
            let angle = super::lerp(half_pi * 3.0, half_pi * 4.0, t);
            let alpha = flip_across_midline(-f32::sin(angle), 0.5);
            println!("{alpha}");
            super::lerp(a, b, alpha)
        }
    }
    pub mod out_direction {
        pub fn sine(a: f32, b: f32, t: f32) -> f32 {
            let alpha = f32::sin(t * (super::PI / 2.0));
            super::lerp(a, b, alpha)
        }
        pub fn circular(a: f32, b: f32, t: f32) -> f32 {
            let half_pi = super::PI / 2.0;
            let angle = super::lerp(half_pi * 2.0, half_pi * 1.0, t);
            let alpha = f32::sin(angle);
            super::lerp(a, b, alpha)
        }
    }
}
fn map_world_to_screen_space(number: BigNumber) -> f32 {
    let mut number_clone = number.clone();
    number_clone.decrease_power(
        number_clone.exponent
            - (map_ranges(number_clone.exponent as f32, 7.0, 9.0, 1.0, 3.0) as i32),
    );
    number_clone.base * Real::powf(10.0, number_clone.exponent as f32)
}
fn map_screen_to_world_space(number: f32) -> BigNumber {
    BigNumber::new_d(number) * Real::powf(10.0, 6.0)
}
fn draw_object<T: SpaceObject>(object: &T) {
    let position = object.get_position();
    let radius = object.get_radius();
    let (x, y) = (
        map_world_to_screen_space(position.x),
        map_world_to_screen_space(position.y),
    );
    draw_circle(x, y, radius, object.get_current_color());
}
fn lerp_color<T: SpaceObject>(object: &mut T) {
    let color_vector = object.get_color();
    let new_elapsed_time =
        object.set_color_elapsed_time(object.get_color_elapsed_time() + get_frame_time());
    let current_alpha = (new_elapsed_time % 2.0) / 2.0;
    let current_index = ((new_elapsed_time / 2.0) % (color_vector.len() as f32)).floor();
    object.set_current_color(Color::new(
        easing_styles::out_direction::circular(
            color_vector.get(current_index as usize).unwrap().r,
            color_vector
                .get(((current_index + 1.0) % (color_vector.len() as f32)) as usize)
                .unwrap()
                .r,
            current_alpha,
        ),
        easing_styles::out_direction::circular(
            color_vector.get(current_index as usize).unwrap().g,
            color_vector
                .get(((current_index + 1.0) % (color_vector.len() as f32)) as usize)
                .unwrap()
                .g,
            current_alpha,
        ),
        easing_styles::out_direction::circular(
            color_vector.get(current_index as usize).unwrap().b,
            color_vector
                .get(((current_index + 1.0) % (color_vector.len() as f32)) as usize)
                .unwrap()
                .b,
            current_alpha,
        ),
        1.0,
    ));
    ()
}
struct DrawObject;
struct ColorLerp;
struct UpdateBackgroundStars;
impl<'a> System<'a> for ColorLerp {
    type SystemData = (
        WriteStorage<'a, Sun>,
        WriteStorage<'a, Planet>,
        WriteStorage<'a, IsOrbital>,
    );
    fn run(&mut self, (mut sun, mut planet, mut is_orbital): Self::SystemData) {
        (&mut sun).join().for_each(lerp_color);
        (&mut planet).join().for_each(lerp_color);
        (&mut is_orbital).join().for_each(|object| {
            for moon in object.moons.iter_mut() {
                lerp_color(moon);
            }
        });
        ()
    }
}
impl<'a> System<'a> for DrawObject {
    type SystemData = (
        ReadStorage<'a, Sun>,
        ReadStorage<'a, Planet>,
        ReadStorage<'a, IsOrbital>,
    );
    fn run(&mut self, (sun, planet, is_orbital): Self::SystemData) {
        sun.join().for_each(draw_object);
        planet.join().for_each(draw_object);
        is_orbital.join().for_each(|object| {
            for moon in object.moons.iter() {
                draw_object(moon);
            }
        });
    }
}
#[derive(Debug)]
struct BackgroundStars {
    offset_from_center: Vec2,
    speed: f32,
}
impl Component for BackgroundStars {
    type Storage = VecStorage<Self>;
}
impl<'a> System<'a> for UpdateBackgroundStars {
    type SystemData = (ReadStorage<'a, BackgroundStars>);
    fn run(&mut self, (background_star): Self::SystemData) {}
}
fn window_conf() -> Conf {
    Conf {
        window_title: "ORBITAL_SYSTEM".to_string(),
        window_resizable: false,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    request_new_screen_size(0.8 * 1920.0, 0.8 * 1080.0);
    let mut world = World::new();
    world.register::<Sun>();
    world.register::<Planet>();
    world.register::<IsOrbital>();
    // Initialize Simulation
    let mut first_iteration = true;
    let mut color_lerp = ColorLerp;
    let mut draw_object = DrawObject;
    loop {
        clear_background(BLACK);
        if screen_width() != 0.8 * 1920.0 || screen_height() != 0.8 * 1080.0 {
            next_frame().await;
            continue;
        }
        if first_iteration {
            first_iteration = false;
            world
                .create_entity()
                .with(Sun {
                    position: BigVec2 {
                        x: map_screen_to_world_space(screen_width() / 2.0),
                        y: map_screen_to_world_space(screen_height() / 2.0),
                    },
                    radius: 50.0,
                    color: vec![YELLOW, ORANGE],
                    current_color: YELLOW,
                    color_elapsed_time: 0.0,
                })
                .build();
            continue;
        }
        color_lerp.run_now(&world);
        draw_object.run_now(&world);
        next_frame().await;
    }
}
