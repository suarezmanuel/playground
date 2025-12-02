use crate::types::pin_type::*;
use macroquad::prelude::*;

#[derive(Clone)]
pub struct Pin { // this will be the new spatialPinIndex
    pub rect: Rect,
    pub index: usize,
    pub pin_type: PinType,
    pub wire_index: Option<usize>,
}

pub type Pins = Vec<Pin>;
