use macroquad::prelude::*;

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
