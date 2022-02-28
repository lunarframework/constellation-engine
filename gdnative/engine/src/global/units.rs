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
