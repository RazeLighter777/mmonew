use serde::{Deserialize, Serialize};

use crate::component;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x : i32,
    pub y : i32,
}

impl component::ComponentType for Position {
}