use serde::{Deserialize, Serialize};

use crate::Star;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Data {
    pub stars: Vec<Star>,
}
