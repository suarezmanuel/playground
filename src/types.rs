use macroquad::{input, prelude::*};
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
    pub emulation_done: bool,
    pub wires_read: Vec<bool>,
    pub wires_write: Vec<bool>,
    pub gates: Vec<Gate>, // make sure this array is ordered topologically
}

impl Circuit {

    pub fn new() -> Circuit {
        // first wire acts as input (later generalize to handle multiple inputs e.g. binary numbers)
        return Circuit{emulation_done: false, wires_read: vec![], wires_write: vec![], gates: Vec::new()};
    }

    // wires always start as false.
    pub fn new_wire(&mut self) -> u32 {
        self.wires_read.push(false);
        self.wires_write.push(false); // to make them equal in length so no problems when swapping
        return (self.wires_read.len() - 1) as u32;
    }

    // make it so a 'NOT' gate automatically makes the wire yellow.
    pub fn set_wire(&mut self, wire_index: u32, value: bool) {
        if !(0..self.wires_read.len()).contains(&(wire_index as usize)) {
            panic!("invalid wire_index {} for wires_read", wire_index)
        }
        self.wires_read[wire_index as usize]  = value;
        self.wires_write[wire_index as usize] = value;
    }

    pub fn get_wire(&mut self, wire_index: u32) -> bool {
        if !(0..self.wires_read.len()).contains(&(wire_index as usize)) {
            panic!("invalid wire_index {} for wires_read", wire_index)
        }
        return self.wires_read[wire_index as usize];
    }

    pub fn evaluate(&self, gate: &Gate) -> bool {
        // make sure somehow that the input is always set-up
        match (&gate.gate_type, &gate.input) {
            (GateType::NOT, Input::Single(input1)) => {
                let a = self.wires_read[*input1 as usize];
                !a
            }
            (GateType::OR, Input::Dual(input1, input2)) => {
                let a = self.wires_read[*input1 as usize];
                let b = self.wires_read[*input2 as usize];
                a | b
            }
            (GateType::XOR, Input::Dual(input1, input2)) => {
                let a = self.wires_read[*input1 as usize];
                let b = self.wires_read[*input2 as usize];
                a ^ b
            }
            (GateType::XNOR, Input::Dual(input1, input2)) => {
                let a = self.wires_read[*input1 as usize];
                let b = self.wires_read[*input2 as usize];
                !(a ^ b)
            }
            (GateType::NOR, Input::Dual(input1, input2)) => {
                let a = self.wires_read[*input1 as usize];
                let b = self.wires_read[*input2 as usize];
                !(a | b)
            }
            (GateType::AND, Input::Dual(input1, input2)) => {
                let a = self.wires_read[*input1 as usize];
                let b = self.wires_read[*input2 as usize];
                a & b
            }
            (GateType::NAND, Input::Dual(input1, input2)) => {
                let a = self.wires_read[*input1 as usize];
                let b = self.wires_read[*input2 as usize];
                !(a & b)
            }
            _ => panic!("Unsupported gate type or input configuration"),
        }
    }
    
    pub fn connect_wire(&mut self, wire: u32, out_gate_index: Option<usize>, in_gate_index: Option<usize>, inpin_index: u32) { // the terms input / output are confusing here, they are from the pov of each gate
        
        {
            let out_gate = match out_gate_index { Some(out_gate_index) => {Some(&mut self.gates[out_gate_index])} _ => {None} };
            // wire input coming from gate, allow for input from an 'electricity source' (not a gate)
            match out_gate {
                Some(value) => {value.output = wire;}
                _ => {}
            };
        }

        {
            match in_gate_index { 
                Some(in_gate_index) => {
                    let in_gate = &mut self.gates[in_gate_index];
                    // wire output going to gate pin, allow for output to "air"
                    match &mut in_gate.input {
                        Input::Single(input1)                                     => {if inpin_index != 0 {panic!("invalid inpin_index {} for gate", inpin_index)} *input1 = wire; }
                        Input::Dual(input1, input2)                     => {if !(0..=1).contains(&(inpin_index as usize)) {panic!("invalid inpin_index {} for gate", inpin_index)}; match inpin_index { 0 => {*input1 = wire;} 1 => {*input2 = wire} _ => {}};}
                        Input::Triple(input1, input2, input3) => {if !(0..=2).contains(&(inpin_index as usize)) {panic!("invalid inpin_index {} for gate", inpin_index)} *input1 = wire; match inpin_index { 0 => {*input1 = wire;} 1 => {*input2 = wire} 2 => {*input3 = wire} _ => {}};}
                        Input::Variadic(vec)                                 => {if !(0..vec.len()).contains(&(inpin_index as usize)) {panic!("invalid inpin_index {} for gate", inpin_index)} vec[inpin_index as usize] = wire; }
                    }
                }
                _ => {} 
            };
        }
    }
    
    pub fn tick(&mut self) {
        // check for same length so no problems when swapping
        if self.wires_read.len() != self.wires_write.len() {
            panic!("wire buffers are not of the same length");
        }

        let mut changed_wires = vec![false; self.wires_read.len()];
        // read
        for gate in &self.gates {
            let result = self.evaluate(gate);
            if changed_wires[gate.output as usize] && self.wires_write[gate.output as usize] == !result {
                panic!("short circuit on wire {}", gate.output);
            }
            self.wires_write[gate.output as usize] = result;
            changed_wires[gate.output as usize] = true;
        }

        // check if emulation is done (add output test in future)
        self.emulation_done = true;
        for (index, value) in self.wires_read.iter().enumerate() {
            if *value != self.wires_write[index] {
                self.emulation_done = false;
            }
        }
        // write
        std::mem::swap(&mut self.wires_read, &mut self.wires_write);
    }
}

impl Gate {
    pub fn new(rect: Rect, gate_type: GateType) -> Gate {
        let input = match gate_type {
            GateType::NOT => {Input::Single(0)}
            _ => {Input::Dual(0, 0)}
        };
        return Gate{rect: rect, input: input, output: 0, gate_type: gate_type};
    }
}