use macroquad::{input, prelude::*};

use crate::SpatialBlockIndex;

#[derive(Copy, Clone)] // so it can be used inside a loop
pub enum GateType {
    NOT,
    OR,
    XOR,
    NOR,
    XNOR,
    AND,
    NAND,
    PWR,
    GND,
}

impl GateType {
    pub fn color(&self) -> Color {
        return match self {
            GateType::NOT => RED,
            GateType::OR => PINK,
            GateType::XOR => BLUE,
            GateType::XNOR => GRAY,
            GateType::NOR => ORANGE,
            GateType::AND => PURPLE,
            GateType::NAND => BROWN,
            GateType::PWR => YELLOW,
            GateType::GND => DARKGRAY,
        };
    }

    pub fn text(&self) -> &str {
        return match self {
            GateType::NOT => "not",
            GateType::OR => "or",
            GateType::XOR => "xor",
            GateType::XNOR => "xnor",
            GateType::NOR => "nor",
            GateType::AND => "and",
            GateType::NAND => "nand",
            GateType::PWR => "pwr",
            GateType::GND => "gnd",
        };
    }
}

#[derive(Clone)]
pub enum Pins {
    // Input and Output pins for gates
    Empty,
    Single(usize),
    Dual(usize, usize),
    Triple(usize, usize, usize),
    Variadic(Vec<usize>),
}

#[derive(Debug, Clone)]
pub enum PinType {
    Input,
    Output,
}

impl PinType {
    pub fn to_string(self) -> String {
        return match self {
            PinType::Input => {"input".to_owned()}
            PinType::Output => {"output".to_owned()}
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpatialPinIndex {
    pub rect: Rect,
    pub index: usize,
    pub pin_type: PinType,
}

#[derive(Clone)]
pub struct Gate {
    pub rect: Rect,
    pub input: Pins,
    pub output: Pins,
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
        return Circuit {
            emulation_done: false,
            wires_read: vec![],
            wires_write: vec![],
            gates: Vec::new(),
        };
    }

    // wires always start as false.
    pub fn new_wire(&mut self) -> usize {
        self.wires_read.push(false);
        self.wires_write.push(false); // to make them equal in length so no problems when swapping
        self.wires_read.len() - 1
    }

    // make it so a 'NOT' gate automatically makes the wire yellow.
    pub fn set_wire(&mut self, wire_index: usize, value: bool) {
        if !(0..self.wires_read.len()).contains(&(wire_index)) {
            panic!("invalid wire_index {} for wires_read", wire_index)
        }
        self.wires_read[wire_index] = value;
        self.wires_write[wire_index] = value;
    }

    pub fn get_wire(&mut self, wire_index: usize) -> bool {
        if !(0..self.wires_read.len()).contains(&(wire_index)) {
            panic!("invalid wire_index {} for wires_read", wire_index)
        }
        return self.wires_read[wire_index];
    }

    pub fn evaluate(&self, gate: &Gate) -> bool {
        // make sure somehow that the input is always set-up
        match (&gate.gate_type, &gate.input) {
            (GateType::NOT, Pins::Single(input1)) => {
                let a = self.wires_read[*input1];
                !a
            }
            (GateType::OR, Pins::Dual(input1, input2)) => {
                let a = self.wires_read[*input1];
                let b = self.wires_read[*input2];
                a | b
            }
            (GateType::XOR, Pins::Dual(input1, input2)) => {
                let a = self.wires_read[*input1];
                let b = self.wires_read[*input2];
                a ^ b
            }
            (GateType::XNOR, Pins::Dual(input1, input2)) => {
                let a = self.wires_read[*input1];
                let b = self.wires_read[*input2];
                !(a ^ b)
            }
            (GateType::NOR, Pins::Dual(input1, input2)) => {
                let a = self.wires_read[*input1];
                let b = self.wires_read[*input2];
                !(a | b)
            }
            (GateType::AND, Pins::Dual(input1, input2)) => {
                let a = self.wires_read[*input1];
                let b = self.wires_read[*input2];
                a & b
            }
            (GateType::NAND, Pins::Dual(input1, input2)) => {
                let a = self.wires_read[*input1];
                let b = self.wires_read[*input2];
                !(a & b)
            }
            (GateType::PWR, Pins::Empty) => true,
            (GateType::GND, Pins::Single(_)) => false,
            _ => panic!("Unsupported gate type or input configuration"),
        }
    }

    pub fn connect_wire(
        &mut self,
        wire: usize,
        out_gate_index: usize,
        in_gate_index: usize,
        outpin_index: u32,
        inpin_index: u32,
    ) {
        // the terms input / output are confusing here, they are from the pov of each gate

        {
            let out_gate = &mut self.gates[out_gate_index];
            // wire input coming from gate, allow for input from an 'electricity source' (not a gate)
            out_gate.output = Pins::Single(wire);
        }

        {
            let in_gate = &mut self.gates[in_gate_index];
            // wire output going to gate pin, allow for output to "air"
            match &mut in_gate.input {
                Pins::Empty => {
                    panic!("cannot connect wire to gate with no input pins");
                }
                Pins::Single(input1) => {
                    if inpin_index != 0 {
                        panic!("invalid inpin_index {} for gate", inpin_index)
                    }
                    *input1 = wire;
                }
                Pins::Dual(input1, input2) => {
                    if !(0..=1).contains(&(inpin_index as usize)) {
                        panic!("invalid inpin_index {} for gate", inpin_index)
                    };
                    match inpin_index {
                        0 => {
                            *input1 = wire;
                        }
                        1 => *input2 = wire,
                        _ => {}
                    };
                }
                Pins::Triple(input1, input2, input3) => {
                    if !(0..=2).contains(&(inpin_index as usize)) {
                        panic!("invalid inpin_index {} for gate", inpin_index)
                    }
                    *input1 = wire;
                    match inpin_index {
                        0 => {
                            *input1 = wire;
                        }
                        1 => *input2 = wire,
                        2 => *input3 = wire,
                        _ => {}
                    };
                }
                Pins::Variadic(vec) => {
                    if !(0..vec.len()).contains(&(inpin_index as usize)) {
                        panic!("invalid inpin_index {} for gate", inpin_index)
                    }
                    vec[inpin_index as usize] = wire;
                }
            }
        }
        {
            let out_gate = &mut self.gates[out_gate_index];
            match &mut out_gate.output {
                Pins::Empty => {
                    panic!("cannot connect wire to gate with no output pins");
                }
                Pins::Single(output1) => {
                    if outpin_index != 0 {
                        panic!("invalid outpin_index {} for gate", outpin_index)
                    }
                    *output1 = wire;
                }
                _ => {
                    panic!("only single output pins supported for now");
                }
            }
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
            let output_bit = match &gate.output {
                Pins::Single(output1) => *output1,
                _ => panic!("only single output pins supported for now"),
            };
            if changed_wires[output_bit] && self.wires_write[output_bit] == !result {
                panic!("short circuit on wire {}", output_bit);
            }
            self.wires_write[output_bit] = result;
            changed_wires[output_bit] = true;
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
            GateType::PWR => Pins::Empty,
            GateType::NOT | GateType::GND => Pins::Single(0),
            _ => Pins::Dual(0, 0),
        };

        let output = match gate_type {
            GateType::GND => Pins::Empty,
            _ => Pins::Single(0),
        };

        return Gate {
            rect: rect,
            input: input,
            output: output,
            gate_type: gate_type,
        };
    }

    pub fn get_pin_count(pins: Pins) -> usize {
        return match pins {
            Pins::Single(_) => 1,
            Pins::Dual(_, _) => 2,
            Pins::Triple(_, _, _) => 3,
            Pins::Variadic(vec) => vec.len(),
            _ => 0,
        };
    }

    pub fn get_side_pins_rects(
        pins: Pins,
        pin_type: PinType,
        tl_x: f32,
        tl_y: f32,
    ) -> Vec<SpatialPinIndex> {
        let mut rects: Vec<SpatialPinIndex> = Vec::new();

        let pin_count = Self::get_pin_count(pins);
        if pin_count > 8 {
            println!("WARNING: i hope youre not using gates that are 64x64");
        }
        let pin_pixel_side_len = 6.0;
        let spaces_count = (pin_count + 1) as f32;
        let space_pixel_len = (64.0 - (pin_count as f32) * pin_pixel_side_len) / spaces_count;

        for i in 1..=pin_count {
            rects.push(SpatialPinIndex {
                rect: Rect {
                    x: tl_x,
                    // camera is upside down
                    y: tl_y + 64.0 - space_pixel_len * (i as f32) - pin_pixel_side_len * ((i - 1) as f32),
                    w: pin_pixel_side_len,
                    h: pin_pixel_side_len,
                },
                index: i-1,
                pin_type: pin_type.clone(),
            });
        }

        return rects;
    }

    pub fn get_pins_blocks(self) -> Vec<SpatialPinIndex> {
        let mut rects: Vec<SpatialPinIndex> = Vec::new();
        // input pins
        rects.extend(Self::get_side_pins_rects(
            self.input.clone(),
            PinType::Input,
            self.rect.x,
            self.rect.y,
        ));

        // output pins
        rects.extend(Self::get_side_pins_rects(
            self.output.clone(),
            PinType::Output,
            self.rect.x + self.rect.w - 6.0,
            self.rect.y,
        ));

        return rects;
    }
}
