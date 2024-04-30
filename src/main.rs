#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_variables)]
extern crate specs;
use std::collections::HashMap;
use std::default;
use std::env;
use std::f32::consts::PI;
use std::fmt::format;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::ops::Mul;
use std::path::Path;

use big_number::BigNumber;
use big_number::BigVec2;

use ::rand::random;
use big_number::Format;
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
use physics::AccelerationType;
use physics::RigidBody;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use serde_json::Value;
use specs::prelude::*;
mod big_number;
mod physics;

const SUN_MASS: f32 = 1.0;
const ASTRONOMICAL_UNIT: f32 = 149597871.0;
const FONT_SIZE: f32 = 32.0;
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
#[derive(Clone, Debug)]
struct Planet {
    position: BigVec2,
    focus: Vec2,
    radius: f32,
    color: Vec<Color>,
    current_color: Color,
    color_elapsed_time: f32,
    name: String,
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
struct DrawTextAbovePlanets;
struct DisplayPlanetInformation;

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
                        .mul(rand::gen_range(200, 1000) as f32 / 10.0),
                        acceleration_type: AccelerationType::Linear,
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
#[derive(Deserialize, Serialize, Clone, Debug)]
struct PlaceholderColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
#[derive(Clone, Debug)]
struct OrbitMetadata {
    gravitational_constant: f32,
    eccentricity: f32,
    major_axis: f32,
    theta: f32,
    color: Vec<Color>,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
struct OrbitMetadataSave {
    gravitational_constant: f32,
    eccentricity: f32,
    major_axis: f32,
    name: String,
}
impl OrbitMetadataSave {
    fn to_unsavable(&self) -> OrbitMetadata {
        let mut color_vector = Vec::new();
        color_vector.push(Color::new(
            rand::gen_range(0.0, 1.0),
            rand::gen_range(0.0, 1.0),
            rand::gen_range(0.0, 1.0),
            1.0,
        ));
        OrbitMetadata::new(
            self.gravitational_constant,
            self.eccentricity,
            map_world_to_screen_space(BigNumber::new_d(ASTRONOMICAL_UNIT) * self.major_axis),
            color_vector,
        )
    }
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
impl<'a> System<'a> for DrawTextAbovePlanets {
    type SystemData = (ReadStorage<'a, Planet>);
    fn run(&mut self, (planet): Self::SystemData) {
        for object in planet.join() {
            draw_text(
                &object.name,
                map_world_to_screen_space(object.position.clone().x),
                map_world_to_screen_space(object.position.clone().y) - 50.0,
                32.0,
                RED,
            );
        }
    }
}
impl<'a> System<'a> for DisplayPlanetInformation {
    type SystemData = (Write<'a, SelectedPlanet>, ReadStorage<'a, Planet>);
    fn run(&mut self, (mut selected_planet, planet): Self::SystemData) {
        let (mouse_x, mouse_y) = mouse_position();
        let mut identical = None;
        for object in planet.join() {
            let object_position = object.get_position();
            if ((((mouse_x - (map_world_to_screen_space(object_position.x))).abs()
                < object.radius)
                && ((mouse_y - map_world_to_screen_space(object_position.y)).abs()
                    < object.radius))
                && is_mouse_button_pressed(MouseButton::Left))
            {
                identical = Some((*object).clone());
            }
            match selected_planet.0 {
                Some(ref x) => {
                    if (x.name == object.name) {
                        identical = Some(object.clone());
                    }
                }
                None => {}
            }
        }
        selected_planet.0 = identical;
        match selected_planet.0 {
            Some(ref x) => {
                let position = x.get_position();
                draw_text(
                    format!(
                        "Coordinates: ({}, {})",
                        match position.x.serialized {
                            Format::Haven(x) => x,
                            Format::Scientific(x) => x,
                        },
                        match position.y.serialized {
                            Format::Haven(x) => x,
                            Format::Scientific(x) => x,
                        }
                    )
                    .as_str(),
                    (0.8 * 1920.0) - FONT_SIZE * 15.0,
                    FONT_SIZE * 1.0,
                    FONT_SIZE,
                    GREEN,
                )
            }
            _ => {}
        }
        ()
    }
}
#[derive(Default)]
struct SelectedPlanet(Option<Planet>);
fn window_conf() -> Conf {
    Conf {
        window_title: "ORBITAL_SYSTEM".to_string(),
        window_resizable: false,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() -> io::Result<()> {
    env::set_var("RUST_BACKTRACE", "full");
    request_new_screen_size(0.8 * 1920.0, 0.8 * 1080.0);
    let mut world = World::new();
    world.register::<Sun>();
    world.register::<Planet>();
    world.register::<IsOrbital>();
    world.register::<BackgroundStars>();

    world.register::<Renderable>();
    world.register::<RigidBody>();
    world.insert(SelectedPlanet(None));
    // Initialize Simulation
    let mut first_iteration = true;
    let mut color_lerp = ColorLerp;
    let mut draw_object = DrawObject;
    let mut destroy_background_stars = DestroyBackgroundStars;
    let mut add_background_stars = AddBackgroundStars;
    let mut update_background_stars = UpdateBackgroundStars;
    let mut update_planet_positions = UpdatePlanetPositions;
    let mut draw_text_above_planets = DrawTextAbovePlanets;
    let mut display_planet_information = DisplayPlanetInformation;
    let mut orbit_metadata = HashMap::new();
    orbit_metadata.insert(
        "Lubaitis".to_string(),
        OrbitMetadata::new(
            3.7,
            0.0206,
            map_world_to_screen_space(BigNumber::new_d(ASTRONOMICAL_UNIT) * 1.4),
            vec![GRAY, WHITE],
        ),
    );
    orbit_metadata.insert(
        "Nora U3".to_string(),
        OrbitMetadata::new(
            23.1,
            0.8,
            map_world_to_screen_space(BigNumber::new_d(ASTRONOMICAL_UNIT) * 2.5),
            vec![BROWN, ORANGE, WHITE],
        ),
    );
    orbit_metadata.insert(
        "Zerth RM8F".to_string(),
        OrbitMetadata::new(
            8.7,
            0.37,
            map_world_to_screen_space(BigNumber::new_d(ASTRONOMICAL_UNIT) * 4.0),
            vec![BLUE, PURPLE],
        ),
    );
    let data_base_path = Path::new("data_base");
    let written_file = File::open(&data_base_path.join("written.json"))?;
    let written_file_reader = BufReader::new(&written_file);
    let mut metadata = Vec::new();
    metadata.push(String::new());
    let mut indent_sizes = Vec::new();
    for line in written_file_reader.lines() {
        let mut unwrapped_line = line.unwrap();
        let indent_size = unwrapped_line.len() - unwrapped_line.trim_start().len();
        unwrapped_line = unwrapped_line.trim().to_string();
        let mut vector_indent_index = 0;
        if (indent_sizes.get(indent_size).is_some()) {
            vector_indent_index = *indent_sizes.get(indent_size).unwrap();
        } else {
            vector_indent_index = indent_sizes.len();
            indent_sizes.push(vector_indent_index);
        }
        if (vector_indent_index == 1) {
            let mut last = metadata.last().unwrap().clone();
            metadata.pop();
            if (!unwrapped_line.contains("}")) {
                last.push_str("{");
                metadata.push(last);
                continue;
            }
            last.push_str("}");
            metadata.push(last);
            metadata.push(String::new());
        } else if (vector_indent_index == 2) {
            let mut last = metadata.last().unwrap().clone();
            last.push_str(unwrapped_line.as_str());
            metadata.pop();
            metadata.push(last);
        }
    }
    for string_data in metadata {
        let orbit_object = Deserializer::from_str(string_data.trim()).into_iter::<Value>();
        for value in orbit_object {
            let unwrapped_value = value.unwrap();
            let gravitational_constant = match (unwrapped_value.get("gravitational_constant")) {
                Some(value) => value.as_f64().unwrap(),
                None => 0.0,
            } as f32;
            let eccentricity = match (unwrapped_value.get("eccentricity")) {
                Some(value) => value.as_f64().unwrap(),
                None => 0.0,
            } as f32;
            let major_axis = match (unwrapped_value.get("major_axis")) {
                Some(value) => value.as_f64().unwrap(),
                None => 0.0,
            } as f32;
            let name = match (unwrapped_value.get("name")) {
                Some(value) => value.as_str().unwrap(),
                None => "L",
            }
            .to_string();
            orbit_metadata.insert(
                name.clone(),
                (OrbitMetadataSave {
                    gravitational_constant,
                    eccentricity,
                    major_axis,
                    name,
                })
                .to_unsavable(),
            );
        }
    }
    loop {
        clear_background(BLACK);
        if screen_width() != 0.8 * 1920.0 || screen_height() != 0.8 * 1080.0 {
            next_frame().await;
            continue;
        }
        if first_iteration {
            first_iteration = false;
            for (key, individual) in orbit_metadata.iter_mut() {
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
                        name: key.to_string(),
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
        draw_text_above_planets.run_now(&world);
        display_planet_information.run_now(&world);
        world.maintain();
        next_frame().await;
    }
}
