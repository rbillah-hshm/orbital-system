use num::{
    complex::ComplexFloat,
    traits::{
        float::{FloatCore, TotalOrder},
        real::Real,
        Pow,
    },
    Float,
};
use std::{
    cmp::{self, Ordering},
    ops::{self, BitAndAssign},
};
const HAVEN_ABBREVIATIONS: [Option<&str>; 9] = [
    None,
    Some("K"),
    Some("M"),
    Some("B"),
    Some("T"),
    Some("QD"),
    Some("QN"),
    Some("SX"),
    Some("SP"),
];
const ROUNDING_ORDER: f32 = 2.0;
fn bool_from_number(number: i32) -> bool {
    match number {
        -1 => false,
        0 => false,
        _ => true,
    }
}
fn sign_from_bool(boolean: bool) -> i32 {
    match boolean {
        false => -1,
        true => 1,
    }
}
fn reverse_number(a: i32, b: i32, c: i32) -> i32 {
    *(a..=b)
        .rev()
        .collect::<Vec<i32>>()
        .get((c - a) as usize)
        .unwrap_or(&0)
}
fn cyclic_wrap(a: i32, b: i32, max: i32) -> f32 {
    let modulo_of_sum = (a + b) % max;
    Real::powf(
        10.0,
        match (modulo_of_sum != 0) {
            true => modulo_of_sum,
            false => max,
        } as f32,
    )
}
fn exponential_modulo_ten(a: f32, b: f32) -> i32 {
    let modulus = (a.log10().floor() - b.log10().floor()) % 3.0;
    let map = match modulus as i32 {
        0 => 0,
        1 => 1,
        _ => -1,
    };
    map
}
fn get_first_significant_figure(number: f32) -> f32 {
    number / Real::powf(10.0, number.log10().floor())
}
fn round_to_order(number: f32) -> f32 {
    (number * Real::powf(10.0, ROUNDING_ORDER)).floor() / Real::powf(10.0, ROUNDING_ORDER)
}
fn is_between_integers(a: f32, b: i32, c: i32) -> bool {
    (a > b as f32) && (a < c as f32)
}
#[derive(Debug, Clone)]
pub enum Format {
    Haven(String),
    Scientific(String),
}
#[derive(Debug, Clone)]
pub struct BigNumber {
    pub serialized: Format,
    pub base: f32,
    pub exponent: i32,
    pub is_positive: bool,
}
macro_rules! handle_analysis_errors {
    ($condition:expr, $error:expr) => {
        if ($condition) {
            match $error.err().unwrap() {
                AnalysisErrors::InvalidPrefix => {
                    println!("BRO");
                }
                AnalysisErrors::InvalidSuffix => {
                    println!("BRUH");
                }
                AnalysisErrors::InvalidExponent => {
                    println!("DUDE");
                }
            }
            return None;
        }
    };
}
impl BigNumber {
    pub fn new(serialized: Format) -> Option<BigNumber> {
        let mut big_number = BigNumber::new_d(1.0);
        let inner = big_number.get_value();
        let exponent = match serialized {
            Format::Haven(_) => Haven::get_exponent(inner.clone()),
            Format::Scientific(_) => Scientific::get_exponent(inner.clone()),
        }
        .ok()
        .unwrap();
        let multiplier = match serialized {
            Format::Haven(_) => Haven::get_multiplier(inner, exponent),
            Format::Scientific(_) => Scientific::get_multiplier(inner, exponent),
        };
        handle_analysis_errors!(multiplier.is_err(), multiplier);
        big_number.increase_power(exponent, false);
        match serialized {
            Format::Haven(_) => {
                big_number.serialized =
                    Format::Haven(Haven::create(multiplier.ok().unwrap(), exponent, false));
            }
            Format::Scientific(_) => {
                big_number.serialized = Format::Scientific(Scientific::create(
                    multiplier.ok().unwrap(),
                    exponent,
                    false,
                ));
            }
        }
        Some(big_number)
    }
    pub fn new_d(deserialized: f32) -> BigNumber {
        let positive_deserialized = deserialized.abs();
        let mut temp = BigNumber {
            serialized: Format::Haven(("1.0").to_string()),
            base: 1.0,
            exponent: 0,
            is_positive: (deserialized >= 0.0),
        };
        if (deserialized == 0.0) {
            return BigNumber {
                serialized: Format::Haven(("0.0").to_string()),
                base: 0.0,
                exponent: 0,
                ..Default::default()
            };
        }
        if (is_between_integers(positive_deserialized, 0, 1)) {
            let base = get_first_significant_figure(positive_deserialized);
            let exponent = -((1.0 / positive_deserialized).log10().ceil()) as i32;
            return BigNumber {
                serialized: Format::Scientific(Scientific::create(base, exponent, false)),
                base,
                exponent,
                is_positive: (deserialized >= 0.0),
            };
        }
        if (positive_deserialized < 10.0) {
            return BigNumber {
                serialized: Format::Haven((positive_deserialized as f32).to_string()),
                base: positive_deserialized,
                exponent: 0,
                is_positive: (deserialized >= 0.0),
            };
        }
        temp.increase_power(positive_deserialized.log10().floor() as i32, false);
        match temp.serialized {
            Format::Haven(x) => {
                temp.base = get_first_significant_figure(positive_deserialized);
                temp.serialized =
                    Format::Haven(Haven::create(temp.base, temp.exponent as i32, false));
            }
            Format::Scientific(x) => {
                let current_multiplier =
                    Scientific::get_multiplier(x, positive_deserialized.log10().floor() as i32);
                temp.base = current_multiplier.ok().unwrap();
                temp.serialized = Format::Scientific(Scientific::create(
                    temp.base,
                    positive_deserialized.log10().floor() as i32,
                    false,
                ));
            }
        }
        temp
    }
    pub fn get_value(&self) -> String {
        match self.serialized.clone() {
            Format::Haven(x) => x,
            Format::Scientific(x) => x,
        }
    }
    pub fn increase_power(&mut self, increment: i32, is_bounce: bool) -> Option<bool> {
        if (increment == 0) {
            return Some(true);
        }
        let exponent = match self.serialized.clone() {
            Format::Haven(x) => Haven::get_exponent(x),
            Format::Scientific(x) => Scientific::get_exponent(x),
        };
        handle_analysis_errors!(exponent.is_err(), exponent);
        let mut flag = false;
        let new_power = match (is_bounce && (self.exponent + increment) < 0) {
            false => self.exponent + increment,
            true => {
                flag = true;
                (self.exponent + increment).abs()
            }
        };
        if ((new_power > (HAVEN_ABBREVIATIONS.len() * 3) as i32) || (flag)) {
            let multiplier = Scientific::get_multiplier(self.get_value(), increment);
            handle_analysis_errors!(multiplier.is_err(), multiplier);
            self.exponent = new_power;
            self.serialized = Format::Scientific(Scientific::create(self.base, new_power, false));
        } else {
            let multiplier = Haven::get_multiplier(self.get_value(), new_power);
            handle_analysis_errors!(multiplier.is_err(), multiplier);
            self.exponent = new_power;
            self.serialized = Format::Haven(Haven::create(self.base, new_power, false));
        }
        Some(true)
    }
    pub fn decrease_power(&mut self, increment: i32) -> Option<bool> {
        self.increase_power(-increment, false)
    }
}
impl Default for BigNumber {
    fn default() -> Self {
        BigNumber {
            serialized: Format::Haven("1.0".to_string()),
            base: 1.0,
            exponent: 0,
            is_positive: true,
        }
    }
}
impl PartialEq for BigNumber {
    fn eq(&self, other: &Self) -> bool {
        ((self.is_positive == other.is_positive)
            && (self.base == other.base)
            && (self.exponent == other.exponent))
    }
}
impl Eq for BigNumber {}
impl PartialOrd for BigNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut result = None;
        result = match self.exponent.partial_cmp(&other.exponent) {
            Some(Ordering::Equal) => self.base.partial_cmp(&other.base),
            Some(x) => Some(x),
            None => result,
        };
        result
    }
    fn lt(&self, other: &Self) -> bool {
        (self.partial_cmp(other).unwrap() == Ordering::Less)
    }
    fn le(&self, other: &Self) -> bool {
        ((self.lt(other)) || (self.partial_cmp(other).unwrap() == Ordering::Equal))
    }
    fn gt(&self, other: &Self) -> bool {
        (self.partial_cmp(other).unwrap() == Ordering::Greater)
    }
    fn ge(&self, other: &Self) -> bool {
        ((self.gt(other)) || (self.partial_cmp(other).unwrap() == Ordering::Equal))
    }
}
impl Ord for BigNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match (self > other) {
            false => other,
            true => self,
        }
    }
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match (self < other) {
            false => other,
            true => self,
        }
    }
    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
        Self: PartialOrd,
    {
        (self.min(max)).max(min)
    }
}
impl ops::Add<f32> for BigNumber {
    type Output = BigNumber;
    fn add(self, rhs: f32) -> BigNumber {
        let big_version = BigNumber::new_d(rhs);
        self + big_version
    }
}
impl ops::Add for BigNumber {
    type Output = BigNumber;
    fn add(self, other: BigNumber) -> BigNumber {
        let mut sum = BigNumber::max(self.clone(), other.clone());
        let mut other_min = BigNumber::min(self.clone(), other.clone());
        let base_delta =
            other_min.base / Real::powi(10.0, sum.exponent - other_min.exponent) as f32;
        let new_base = (sum.base * sign_from_bool(sum.is_positive) as f32)
            + (sign_from_bool(other_min.is_positive) as f32 * base_delta);
        let is_bounce = ((new_base < 0.0) && sum.is_positive);
        let exponent_difference = new_base.log10().floor()
            - match is_bounce {
                false => new_base.log10().floor(),
                true => new_base.log10().floor() - other_min.base.log10(),
            };
        if (is_bounce) {
            sum.is_positive = false;
        }
        sum.increase_power(
            match exponent_difference as i32 * sign_from_bool(other_min.is_positive) {
                x => {
                    let mut new_x = x;
                    if (is_bounce) {
                        if (x > 0) {
                            new_x = x + 1;
                        } else {
                            new_x = x - 1;
                        }
                    } else {
                        new_x += other_min.base.log10().floor() as i32;
                    }
                    new_x
                }
            },
            is_bounce,
        );
        sum.base = get_first_significant_figure(new_base.abs());
        match sum.serialized {
            Format::Haven(_) => {
                sum.serialized = Format::Haven(Haven::create(sum.base, sum.exponent, false));
            }
            Format::Scientific(_) => {
                sum.serialized =
                    Format::Scientific(Scientific::create(sum.base, sum.exponent, false));
            }
        }
        sum
    }
}
impl ops::Mul<f32> for BigNumber {
    type Output = BigNumber;
    fn mul(self, rhs: f32) -> Self::Output {
        let big_version = BigNumber::new_d(rhs);
        self * big_version
    }
}
impl ops::Mul for BigNumber {
    type Output = BigNumber;
    fn mul(self, other: BigNumber) -> Self::Output {
        let mut product = self.clone();
        let multiplier = match product.serialized {
            Format::Haven(ref x) => Haven::get_multiplier(x.to_string(), product.exponent),
            Format::Scientific(ref x) => Ok(product.base),
        }
        .ok()
        .unwrap()
            * match other.serialized {
                Format::Haven(ref x) => Haven::get_multiplier(x.to_string(), other.exponent),
                Format::Scientific(ref x) => Ok(other.base),
            }
            .ok()
            .unwrap();
        let mut new_multiplier: f32 = 0.0;
        match product.serialized {
            Format::Haven(ref x) => {
                let original_multiplier = Haven::get_multiplier(x.to_string(), product.exponent);
                let unwrapped_multiplier = original_multiplier.ok().unwrap();
                let difference = multiplier.log10().floor() - unwrapped_multiplier.log10().floor();
                let factor = (10 as i32).pow(match exponential_modulo_ten(
                    multiplier,
                    unwrapped_multiplier,
                ) {
                    x => {
                        let mut result = (unwrapped_multiplier).log10().floor() as i32 + x;
                        let is_less_than = result < 0;
                        let is_greater_than = result > 2;
                        if (is_greater_than) {
                            result = 0;
                        } else if (is_less_than) {
                            result = 2;
                        };
                        result
                    }
                } as u32);
                new_multiplier =
                    get_first_significant_figure(unwrapped_multiplier * other.base) * factor as f32;
                ()
            }
            Format::Scientific(ref x) => {
                new_multiplier = get_first_significant_figure(multiplier);
                let change = (get_first_significant_figure(self.base)
                    * get_first_significant_figure(other.base))
                .log10()
                .floor();
                if (change == 1.0) {
                    product.increase_power(change as i32, false);
                }
                ()
            }
        }
        product.base = get_first_significant_figure(new_multiplier);
        product.increase_power(other.exponent, false);
        match product.serialized {
            Format::Haven(_) => {
                product.serialized = Format::Haven(Haven::create(
                    round_to_order(new_multiplier),
                    product.exponent,
                    true,
                ));
            }
            Format::Scientific(_) => {
                product.serialized =
                    Format::Scientific(Scientific::create(new_multiplier, product.exponent, true));
            }
        }
        product.is_positive = bool_from_number(
            sign_from_bool(product.is_positive) * sign_from_bool(other.is_positive),
        );
        product
    }
}
enum AnalysisErrors {
    // Haven
    InvalidPrefix,
    InvalidSuffix,
    //
    InvalidExponent,
}
trait NumberMethods {
    fn get_exponent(x: String) -> Result<i32, AnalysisErrors>;
    fn get_multiplier(x: String, exponent: i32) -> Result<f32, AnalysisErrors>;
    fn create(a: f32, b: i32, is_product: bool) -> String;
}
struct Haven;
struct Scientific;
impl NumberMethods for Haven {
    fn get_exponent(x: String) -> Result<i32, AnalysisErrors> {
        let mut abbreviation = String::new();
        let mut rest = x
            .chars()
            .rev()
            .skip_while(|char| {
                let is_alphabetic = char.is_ascii_alphabetic();
                if (is_alphabetic) {
                    abbreviation.push(*char);
                }
                is_alphabetic
            })
            .collect::<String>();
        abbreviation = abbreviation.chars().rev().collect::<String>();
        rest = rest.chars().rev().collect::<String>();
        if (abbreviation.is_empty()) {
            let num = x.parse::<f32>();
            let unwrapped = match num {
                Ok(n) => n,
                Err(error) => -1.0,
            };
            if (!bool_from_number(unwrapped as i32)) {
                return Err(AnalysisErrors::InvalidPrefix);
            }
            return Ok(((unwrapped as f32).log10().floor()) as i32);
        } else {
            let mut position = None;
            for (index, suffix) in HAVEN_ABBREVIATIONS.iter().enumerate() {
                if (index == 0) {
                    continue;
                }
                if (*suffix.unwrap() == abbreviation) {
                    position = Some(index);
                    break;
                }
            }
            if (position.is_some()) {
                let parsed_rest = rest.parse::<f32>();
                if (parsed_rest.is_err()) {
                    return Err(AnalysisErrors::InvalidPrefix);
                }
                let result = parsed_rest.ok().unwrap().log10().floor();
                return Ok(((position.unwrap() as i32 * 3 as i32)
                    - reverse_number(1, 3, result as i32)
                    + 1));
            } else {
                return Err(AnalysisErrors::InvalidSuffix);
            }
        }
    }
    fn get_multiplier(x: String, exponent: i32) -> Result<f32, AnalysisErrors> {
        match x
            .chars()
            .take_while(|char| char.is_ascii_digit() || *char == '.')
            .collect::<String>()
            .parse::<f32>()
        {
            Ok(number) => match (number < 1000.0) {
                true => Ok(number),
                false => Ok(get_first_significant_figure(number)
                    * (cyclic_wrap((((number).log10()).floor() as i32), exponent, 3)) as f32),
            },
            Err(error) => Err(AnalysisErrors::InvalidPrefix),
        }
    }
    fn create(a: f32, b: i32, is_product: bool) -> String {
        let mut serialized = String::new();
        let abbreviation = HAVEN_ABBREVIATIONS[(b as f32 / 3.0).floor() as usize];
        if (is_product) {
            serialized.push_str(a.to_string().as_str());
        } else {
            serialized.push_str(
                (get_first_significant_figure(a) * Real::powi(10.0, b % 3))
                    .to_string()
                    .as_str(),
            );
        }
        match abbreviation {
            Some(x) => {
                serialized.push_str(abbreviation.unwrap());
            }
            _ => {}
        }
        serialized
    }
}
impl NumberMethods for Scientific {
    fn get_exponent(x: String) -> Result<i32, AnalysisErrors> {
        let mut exponent = x
            .chars()
            .rev()
            .take_while(|char| char.is_ascii_digit())
            .collect::<String>();
        exponent = exponent.chars().rev().collect::<String>();
        let parsed_exponent = exponent.parse::<i32>();
        match parsed_exponent {
            Ok(number) => Ok(number),
            Err(error) => Err(AnalysisErrors::InvalidExponent),
        }
    }
    fn get_multiplier(x: String, exponent: i32) -> Result<f32, AnalysisErrors> {
        Haven::get_multiplier(x, exponent)
    }
    fn create(a: f32, b: i32, is_product: bool) -> String {
        let mut serialized = String::new();
        serialized.push_str(get_first_significant_figure(a).to_string().as_str());
        serialized.push_str("x10^");
        serialized.push_str(b.to_string().as_str());
        serialized
    }
}
#[derive(Debug, Clone)]
pub struct BigVec2 {
    pub x: BigNumber,
    pub y: BigNumber,
}
