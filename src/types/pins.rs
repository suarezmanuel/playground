use crate::types::keys::*;
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Pin {
    // this will be the new spatialPinIndex
    pub rect: Rect,
    pub index: usize,
    pub wire_index: Option<WireKey>,
}

pub type Pins = Vec<Pin>;
