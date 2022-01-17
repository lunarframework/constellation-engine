use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Length {
    Meter,
    Kilometer,
    AstronomicalUnit,
    LightYear,
    Parsec,
}

impl Default for Length {
    fn default() -> Self {
        Self::Meter
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Time {
    Second,
    Day,
    Year,
}

impl Default for Time {
    fn default() -> Self {
        Self::Second
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Mass {
    Kilogram,
    SolarMass,
}

impl Default for Mass {
    fn default() -> Self {
        Self::Kilogram
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub name: String,
    pub length: Length,
    pub time: Time,
    pub mass: Mass,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::from("default-name"),
            length: Default::default(),
            time: Default::default(),
            mass: Default::default(),
        }
    }
}
