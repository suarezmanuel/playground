#[derive(Clone)]
pub struct Pin {
    pub input_gate: usize,
    pub output_gate: usize,
    pub input_index: usize, 
    pub output_index: usize, 
    pub wire_index: usize
}

// enum a {
//     Input(usize),
//     Output(usize)
// }

pub type Pins = Vec<Pin>;