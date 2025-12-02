use crate::types::pin_type::*;
use macroquad::prelude::*;

<<<<<<< HEAD
#[derive(Clone)]
pub struct Pin { // this will be the new spatialPinIndex
    pub rect: Rect,
=======
#[derive(Clone, Debug)]
pub struct Pin {
>>>>>>> d2ac70ceef18a91f9fd21856ee5c4390df45bb17
    pub index: usize,
    pub pin_type: PinType,
    pub wire_index: Option<usize>,
}

pub type Pins = Vec<Pin>;
