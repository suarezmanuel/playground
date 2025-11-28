use macroquad::prelude::*;
use crate::types::*;

impl circuit {

    pub fn add_gate(&mut self, input1 : u32, input2 : u32, gate_type : gate_type) {

        let newInput2 = match gate_type {
            gate_type::NOT => 0,
            _ => input2
        };
        // not necessarily in topological order
        self.gates.push(gate{input1: input1, input2: newInput2, output: 0 as u32, gate_type : gate_type});
    }

    // the wires result can only be trusted after a 'emulate'
    pub fn add_wire(&mut self, gateOut: gate, gateIn: u32, inputProbe: u32, wireStart : u32, wireEnd : u32) {
        // should remove the already existing
        // gateOut.output = wireStart;
        // updateWireVisual(gateOut);

        // add multiplexers in future
        // let mut probe = match inputProbe {
            
        // }
    }

    // assumes topological order
    pub fn tick(&mut self) {
        for gate in &self.gates {
            let a = self.wires[gate.input1 as usize];
            let b = self.wires[gate.input2 as usize]; // b should always have a value, even for 'NOT'

            let result: bool = match gate.gate_type {
                gate_type::NOT  => !a,
                gate_type::OR   => a | b,
                gate_type::XOR  => a ^ b,
                gate_type::XNOR => !(a ^ b),
                gate_type::NOR  => !(a | b),
                gate_type::AND  => a & b,
                gate_type::NAND => !(a & b),
            };

            self.wires[gate.output as usize] = result;
        }
    }
}