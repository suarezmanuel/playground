use macroquad::prelude::*;
use serde::{Serialize, Deserialize};
use crate::types::keys::*;
use crate::utils::color_serde;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)] // so it can be used inside a loop
pub enum GateType {
    NOT,
    OR,
    XOR,
    NOR,
    XNOR,
    AND,
    NAND,
    IN,
    OUT,
    CUSTOM{
    gates: Vec<GateKey>, // all gates except input / output
    #[serde(with = "color_serde")] 
    color: Color, 
    text: String,
    inputs: Vec<WireKey>, // all wires that have input / output as source, should be generated top-down
    outputs: Vec<WireKey>
    },
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
            GateType::IN => YELLOW,
            GateType::OUT => DARKGRAY,
            GateType::CUSTOM{color, ..} => *color,
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
            GateType::IN => "in",
            GateType::OUT => "out",
            GateType::CUSTOM {text, .. } => text,
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
            GateType::IN => 0,
            GateType::OUT => 1,
            GateType::CUSTOM {inputs, .. } => inputs.len(),
        };
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
            GateType::IN => 1,
            GateType::OUT => 0,
            GateType::CUSTOM { outputs, .. } => outputs.len(),
        };
    }
}
