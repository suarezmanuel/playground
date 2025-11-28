use macroquad::prelude::*;
pub enum gate_type {
    NOT,
    OR,
    XOR,
    NOR,
    XNOR,
    AND,
    NAND
}

pub struct gate {
    pub rect : Rect,
    // can index up to 4,294,967,295 wires.
    pub input1 : u32,
    // for 'NOT' gate this is ignored
    pub input2 : u32,
    pub output : u32,
    pub gate_type : gate_type 
}

pub struct circuit {
    pub wires : Vec<bool>,
    pub gates : Vec<gate>, // make sure this array is ordered topologically
}

// input1, input2, memory
// [true,  false,  false, ...] read
// [true,  false,  false, ...] write

// output
// [false, false, true, true]