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