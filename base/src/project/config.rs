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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Units {
    length: Length,
    time: Time,
    mass: Mass,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Domain {
    /// Rect centered on origin with the given dimensions
    Rect { width: f64, height: f64, depth: f64 },
}

impl Default for Domain {
    fn default() -> Self {
        Self::Rect {
            width: 1.0,
            height: 1.0,
            depth: 1.0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub name: String,
    pub units: Units,
    // pub domain: Domain,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::from("default-name"),
            units: Units::default(),
            // domain: Default::default(),
        }
    }
}
