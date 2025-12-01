use crate::types::pin_type::*;

#[derive(Clone)]
pub struct Pin {
    pub index: usize,
    pub pin_type: PinType,
    pub other_pin_index: Option<usize>,
    pub other_pin_type: Option<PinType>,
    pub other_gate_index: Option<usize>,
    pub wire_index: Option<usize>
}

pub type Pins = Vec<Pin>;