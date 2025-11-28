use macroquad::prelude::*;
pub enum GateType {
    NOT,
    OR,
    XOR,
    NOR,
    XNOR,
    AND,
    NAND,
}

pub enum Input {
    Single(u32),
    Dual(u32, u32),
    Triple(u32, u32, u32),
    Variadic(Vec<u32>),
}

pub struct Gate {
    pub rect: Rect,
    pub input: Input,
    pub output: u32,
    pub gate_type: GateType,
}

pub struct Circuit {
    pub wires: Vec<bool>,
    pub gates: Vec<Gate>, // make sure this array is ordered topologically
}

impl Gate {
    pub fn evaluate(&self, circuit: &Circuit) -> bool {
        let wires = &circuit.wires;
        match (&self.gate_type, &self.input) {
            (GateType::NOT, Input::Single(input1)) => {
                let a = wires[*input1 as usize];
                !a
            }
            (GateType::OR, Input::Dual(input1, input2)) => {
                let a = wires[*input1 as usize];
                let b = wires[*input2 as usize];
                a | b
            }
            (GateType::XOR, Input::Dual(input1, input2)) => {
                let a = wires[*input1 as usize];
                let b = wires[*input2 as usize];
                a ^ b
            }
            (GateType::XNOR, Input::Dual(input1, input2)) => {
                let a = wires[*input1 as usize];
                let b = wires[*input2 as usize];
                !(a ^ b)
            }
            (GateType::NOR, Input::Dual(input1, input2)) => {
                let a = wires[*input1 as usize];
                let b = wires[*input2 as usize];
                !(a | b)
            }
            (GateType::AND, Input::Dual(input1, input2)) => {
                let a = wires[*input1 as usize];
                let b = wires[*input2 as usize];
                a & b
            }
            (GateType::NAND, Input::Dual(input1, input2)) => {
                let a = wires[*input1 as usize];
                let b = wires[*input2 as usize];
                !(a & b)
            }
            _ => panic!("Unsupported gate type or input configuration"),
        }
    }
}

// input1, input2, memory
// [true,  false,  false, ...] read
// [true,  false,  false, ...] write

// output
// [false, false, true, true]
