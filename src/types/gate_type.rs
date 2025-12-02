use macroquad::prelude::*;

#[derive(Copy, Clone, Debug)] // so it can be used inside a loop
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

    pub fn input_count(&self) -> usize {
        return match self {
            GateType::NOT => 1,
            GateType::OR => 2,
            GateType::XOR => 2,
            GateType::XNOR => 2,
            GateType::NOR => 2,
            GateType::AND => 2,
            GateType::NAND => 2,
            GateType::PWR => 0,
            GateType::GND => 1,
        }
    }

    pub fn output_count(&self) -> usize {
        return match self {
            GateType::NOT => 1,
            GateType::OR => 1,
            GateType::XOR => 1,
            GateType::XNOR => 1,
            GateType::NOR => 1,
            GateType::AND => 1,
            GateType::NAND => 1,
            GateType::PWR => 1,
            GateType::GND => 0,
        }
    }
}
