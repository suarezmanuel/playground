use macroquad::prelude::*;
use crate::types::pin_type::*;

#[derive(Debug, Clone)]
pub struct SpatialPinIndex {
    pub rect: Rect,
    pub index: usize,
    pub pin_type: PinType,
}