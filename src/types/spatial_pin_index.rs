use crate::types::pin_type::*;
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub struct SpatialPinIndex {
    pub rect: Rect,
    pub index: usize,
    pub pin_type: PinType,
}
