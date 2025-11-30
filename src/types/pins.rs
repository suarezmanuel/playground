#[derive(Clone)]
pub struct Pin {
    pub other_pin_index: Option<usize>,
    pub other_gate_index: Option<usize>,
    pub wire_index: Option<usize>
}

pub type Pins = Vec<Pin>;