use crate::types::keys::*;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use crate::utils::rect_serde;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pin {
    // this will be the new spatialPinIndex
    #[serde(with = "rect_serde")] 
    pub rect: Rect,
    pub index: usize,
    pub wire_index: Option<WireKey>,
}

pub type Pins = Vec<Pin>;
