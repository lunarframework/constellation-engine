use crate::base::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Length {
    Meter,
    Kilometer,
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

impl Units {
    pub fn speed_of_light(&self) -> f64 {
        let meters = 299792458.0
            * match self.length {
                Length::Meter => 1.0,
                Length::Kilometer => 1000.0,
            };
        let seconds = match self.time {
            Time::Second => 1.0,
            Time::Day => 3600.0 * 24.0,
            Time::Year => 3600.0 * 24.0 * 365.0,
        };

        meters / seconds
    }

    pub fn gravitational_constant(&self) -> f64 {
        let mut base = 6.67408e-11;

        match self.length {
            Length::Meter => (),
            Length::Kilometer => {
                base /= 1000000000.0;
            }
        }

        match self.mass {
            Mass::Kilogram => (),
            Mass::SolarMass => {
                base *= 1.989e30;
            }
        }

        match self.time {
            Time::Second => (),
            Time::Day => {
                base *= 3600.0 * 24.0 * 3600.0 * 24.0;
            }
            Time::Year => {
                base *= 3600.0 * 24.0 * 365.0 * 3600.0 * 24.0 * 365.0;
            }
        }

        base
    }
}

impl Config for Units {}
