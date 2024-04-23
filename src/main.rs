#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_variables)]
extern crate specs;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::Mul;

use big_number::BigNumber;
use big_number::BigVec2;

use ::rand::random;
use macroquad::color;
use macroquad::color::BLACK;
use macroquad::math::Vec2;
use macroquad::prelude::*;
use macroquad::rand;
use macroquad::window::clear_background;
use macroquad::window::next_frame;
use macroquad::window::request_new_screen_size;
use macroquad::window::Conf;
use num::traits::real::Real;
use physics::update_bodies;
use physics::RigidBody;
use specs::prelude::*;
use specs::shred::Fetch;
use specs::shred::FetchMut;
mod big_number;
mod physics;

const SUN_MASS: f32 = 1.0;
const ASTRONOMICAL_UNIT: f32 = 149597871.0;
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
    focus: Vec2,
    radius: f32,
    color: Vec<Color>,
    current_color: Color,
    color_elapsed_time: f32,
    orbit_data: OrbitMetadata,
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
fn get_first_significant_figure(number: f32) -> f32 {
    number / Real::powf(10.0, number.log10().floor())
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
            - (map_ranges(number_clone.exponent as f32, 7.0, 8.0, 1.0, 2.0) as i32),
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
// Elliptical Functions
// I reverse-engineered these formulae into code myself
fn radius_of_ellipse_from_theta(theta: f32, eccentricity: f32, major_axis: f32) -> f32 {
    let semi_major_axis = major_axis / 2.0;
    let semi_lactus_rect = semi_major_axis * (1.0 - Real::powi(eccentricity, 2));
    semi_lactus_rect / (1.0 + (eccentricity * f32::cos(theta)))
}
fn get_ellispe_period(major_axis: f32, gravitational_constant: f32) -> f32 {
    f32::sqrt(
        (4.0 * Real::powi(PI, 2) * Real::powi(major_axis, 3)) / (gravitational_constant * SUN_MASS),
    )
}
fn get_delta_theta(
    current_theta: f32,
    eccentricity: f32,
    major_axis: f32,
    gravitational_constant: f32,
) -> f32 {
    let semi_major_axis = major_axis / 2.0;
    let semi_minor_axis = semi_major_axis * f32::sqrt(1.0 - Real::powi(eccentricity, 2));
    let period = get_ellispe_period(major_axis, gravitational_constant);
    let mean_motion = (2.0 * PI) / period;
    let radius = radius_of_ellipse_from_theta(current_theta, eccentricity, major_axis);
    (semi_minor_axis * semi_major_axis * mean_motion * get_frame_time() * 1000.0)
        / (Real::powi(radius, 2))
}
//
struct DrawObject;
struct ColorLerp;
struct AddBackgroundStars;
struct UpdateBackgroundStars;
struct DestroyBackgroundStars;
struct UpdatePlanetPositions;

struct Renderable;
impl Component for Renderable {
    type Storage = VecStorage<Self>;
}
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
impl<'a> System<'a> for AddBackgroundStars {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, BackgroundStars>,
        Read<'a, LazyUpdate>,
    );
    fn run(&mut self, (entities, background_star, updater): Self::SystemData) {
        let size = background_star.count();
        if (size < 200) {
            for i in (size..200) {
                let star = entities.create();
                updater.insert(
                    star,
                    BackgroundStars {
                        offset_from_center: vec2(0.0, 0.0),
                        speed: 0.0,
                    },
                );
                updater.insert(
                    star,
                    RigidBody {
                        acceleration: vec2(
                            (rand::gen_range(100, 1000) as f32 / 100.0)
                                * (match (rand::gen_range(0.0, 1.0) < 0.5) {
                                    true => -1.0,
                                    false => 1.0,
                                }),
                            (rand::gen_range(100, 1000) as f32 / 100.0)
                                * (match (rand::gen_range(0.0, 1.0) < 0.5) {
                                    true => -1.0,
                                    false => 1.0,
                                }),
                        )
                        .normalize()
                        .mul(rand::gen_range(1000, 10000) as f32 / 10.0),
                        ..Default::default()
                    },
                );
            }
        }
    }
}
impl<'a> System<'a> for UpdateBackgroundStars {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, BackgroundStars>,
        ReadStorage<'a, RigidBody>,
        Read<'a, LazyUpdate>,
    );
    fn run(&mut self, (entities, mut background_star, rigid_body, updater): Self::SystemData) {
        let mut body_vec = Vec::new();
        for (star, body) in (&background_star, &rigid_body).join() {
            body_vec.push(body.clone());
        }
        body_vec = update_bodies(body_vec);
        for (i, (entity, star, body)) in (&entities, &mut background_star, &rigid_body)
            .join()
            .enumerate()
        {
            updater.remove::<RigidBody>(entity);
            updater.insert(entity, body_vec.get(i).unwrap().clone());
            star.offset_from_center = vec2(
                body_vec.get(i).unwrap().position.x,
                body_vec.get(i).unwrap().position.y,
            );
            draw_circle(
                body_vec.get(i).unwrap().position.x + screen_width() / 2.0,
                body_vec.get(i).unwrap().position.y + screen_height() / 2.0,
                map_ranges(star.offset_from_center.length(), 1.0, 1000.0, 1.0, 30.0),
                WHITE,
            );
        }
    }
}
impl<'a> System<'a> for DestroyBackgroundStars {
    type SystemData = (Entities<'a>, ReadStorage<'a, BackgroundStars>);
    fn run(&mut self, (entities, background_star): Self::SystemData) {
        for (entity, star) in (&entities, &background_star).join() {
            let offset = star.offset_from_center;
            if ((f32::abs(offset.x) > (screen_width() / 2.0))
                || f32::abs(offset.y) > (screen_height() / 2.0))
            {
                let _ = entities.delete(entity);
            }
        }
    }
}
#[derive(Clone, Debug)]
struct OrbitMetadata {
    gravitational_constant: f32,
    eccentricity: f32,
    major_axis: f32,
    theta: f32,
    color: Vec<Color>,
}
impl OrbitMetadata {
    fn new(
        gravitational_constant: f32,
        eccentricity: f32,
        major_axis: f32,
        color: Vec<Color>,
    ) -> Self {
        OrbitMetadata {
            gravitational_constant,
            eccentricity,
            major_axis,
            color,
            theta: 0.0,
        }
    }
}
impl<'a> System<'a> for UpdatePlanetPositions {
    type SystemData = (WriteStorage<'a, Planet>);
    fn run(&mut self, (mut planet): Self::SystemData) {
        for object in (&mut planet).join() {
            let mut orbit_data = object.orbit_data.clone();
            orbit_data.theta = (orbit_data.theta
                + get_delta_theta(
                    orbit_data.theta,
                    orbit_data.eccentricity,
                    orbit_data.major_axis,
                    orbit_data.gravitational_constant,
                ))
                % (2.0 * PI);
            let angled_vector = vec2(f32::cos(orbit_data.theta), -f32::sin(orbit_data.theta)).mul(
                radius_of_ellipse_from_theta(
                    orbit_data.theta,
                    orbit_data.eccentricity,
                    orbit_data.major_axis,
                ),
            );
            object.position = BigVec2 {
                x: map_screen_to_world_space(object.focus.x + angled_vector.x),
                y: map_screen_to_world_space(object.focus.y + angled_vector.y),
            };
            object.orbit_data = orbit_data;
        }
    }
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
    world.register::<BackgroundStars>();

    world.register::<Renderable>();
    world.register::<RigidBody>();
    // Initialize Simulation
    let mut first_iteration = true;
    let mut color_lerp = ColorLerp;
    let mut draw_object = DrawObject;
    let mut destroy_background_stars = DestroyBackgroundStars;
    let mut add_background_stars = AddBackgroundStars;
    let mut update_background_stars = UpdateBackgroundStars;
    let mut update_planet_positions = UpdatePlanetPositions;
    let mut orbit_metadata = HashMap::new();
    orbit_metadata.insert(
        "Lubaitis",
        OrbitMetadata::new(
            3.7,
            0.0206,
            map_world_to_screen_space(BigNumber::new_d(ASTRONOMICAL_UNIT) * 1.4),
            vec![GRAY, WHITE],
        ),
    );
    orbit_metadata.insert(
        "Nora U3",
        OrbitMetadata::new(
            23.1,
            0.8,
            map_world_to_screen_space(BigNumber::new_d(ASTRONOMICAL_UNIT) * 2.5),
            vec![BROWN, ORANGE, WHITE],
        ),
    );
    orbit_metadata.insert(
        "Zerth RM8F",
        OrbitMetadata::new(
            8.7,
            0.37,
            map_world_to_screen_space(BigNumber::new_d(ASTRONOMICAL_UNIT) * 4.0),
            vec![BLUE, PURPLE],
        ),
    );
    loop {
        clear_background(BLACK);
        if screen_width() != 0.8 * 1920.0 || screen_height() != 0.8 * 1080.0 {
            next_frame().await;
            continue;
        }
        if first_iteration {
            first_iteration = false;
            for (key, individual) in orbit_metadata.clone().iter_mut() {
                let semi_major_axis = individual.major_axis / 2.0;
                let semi_minor_axis =
                    semi_major_axis * f32::sqrt(1.0 - Real::powi(individual.eccentricity, 2));
                let foci =
                    f32::sqrt(Real::powi(semi_minor_axis, 2) + Real::powi(semi_major_axis, 2));
                world
                    .create_entity()
                    .with(Planet {
                        position: BigVec2 {
                            x: map_screen_to_world_space(
                                (screen_width() / 2.0) + (semi_major_axis - foci),
                            ),
                            y: map_screen_to_world_space(screen_height() / 2.0),
                        },
                        focus: Vec2 {
                            x: (screen_width() / 2.0),
                            y: screen_height() / 2.0,
                        },
                        radius: 12.0,
                        color: individual.color.clone(),
                        current_color: *individual.color.get(0).unwrap(),
                        color_elapsed_time: 0.0,
                        orbit_data: individual.clone(),
                    })
                    .build();
            }
            world
                .create_entity()
                .with(Sun {
                    position: BigVec2 {
                        x: map_screen_to_world_space(screen_width() / 2.0),
                        y: map_screen_to_world_space(screen_height() / 2.0),
                    },
                    radius: 30.0,
                    color: vec![YELLOW, ORANGE],
                    current_color: YELLOW,
                    color_elapsed_time: 0.0,
                })
                .build();
            continue;
        }
        destroy_background_stars.run_now(&mut world);
        add_background_stars.run_now(&mut world);

        update_background_stars.run_now(&world);
        update_planet_positions.run_now(&world);
        color_lerp.run_now(&world);
        draw_object.run_now(&world);
        world.maintain();
        next_frame().await;
    }
}
