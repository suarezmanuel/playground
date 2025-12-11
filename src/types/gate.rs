use crate::types::circuit::*;
use crate::types::gate_type::*;
use crate::types::pin_type::*;
use crate::types::pins::*;
use crate::utils::*;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
const GATE_SIZE: u16 = 64;
const PIN_SIZE: u16 = 6;
const PIN_PIXEL_SIDE_LEN: f32 = PIN_SIZE as f32;
use crate::utils::rect_serde;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
}

impl Rotation {
    pub fn as_degrees(&self) -> f32 {
        match self {
            Rotation::Up => 0.0,
            Rotation::Right => 90.0,
            Rotation::Down => 180.0,
            Rotation::Left => 270.0,
        }
    }

    pub fn as_radians(&self) -> f32 {
        self.as_degrees().to_radians()
    }

    pub fn to_string(&self) -> &str {
        match self {
            Rotation::Up => "up",
            Rotation::Right => "right",
            Rotation::Down => "down",
            Rotation::Left => "left",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gate {
    pub rotation: Rotation,
    #[serde(with = "rect_serde")] 
    pub rect: Rect, //switch to usize for gates (x,y), make lerp
    pub input: Pins,
    pub output: Pins,
    pub gate_type: GateType,
    pub active: bool,
}

impl Gate {
    pub fn new(rect: Rect, rotation: Rotation, gate_type: GateType) -> Gate {
        let (input, output) = Self::get_pins(rect, gate_type.clone(), rotation.clone());
        // println!("rect x: {} y: {}", rect.x, rect.y);
        return Gate {
            rotation: rotation.clone(),
            rect: rect,
            input: input,
            output: output,
            gate_type: gate_type.clone(),
            active: true,
        };
    }

    pub fn get_pins(
        gate_rect: Rect,
        gate_type: GateType,
        rotation: Rotation,
    ) -> (Vec<Pin>, Vec<Pin>) {
        fn get_pin_rect(
            gate_rect: Rect,
            pin_type: PinType,
            rotation: Rotation,
            pin_index: usize,
            pin_count: usize,
        ) -> Rect {
            let spaces_count = (pin_count + 1) as f32;
            let space_len =
                (GATE_SIZE as f32 - (pin_count as f32) * PIN_PIXEL_SIDE_LEN) / spaces_count;

            // Calculate offset from the "start" of the edge
            let offset =
                (space_len * ((pin_index + 1) as f32)) + (PIN_PIXEL_SIDE_LEN * (pin_index as f32));

            // 2. Define the Pin Size (Swap W/H if pins weren't squares, but yours are)
            let w = PIN_PIXEL_SIDE_LEN;
            let h = PIN_PIXEL_SIDE_LEN;

            let (x, y) = match (rotation, pin_type) {
                // --- UP (Standard) ---
                // Input on Left edge, Output on Right edge
                (Rotation::Up, PinType::Input) => (gate_rect.x, gate_rect.y + offset),
                (Rotation::Up, PinType::Output) => (gate_rect.right() - w, gate_rect.y + offset),

                // --- RIGHT (90 deg CW) ---
                // Input on Top edge, Output on Bottom edge
                (Rotation::Right, PinType::Input) => (gate_rect.x + offset, gate_rect.y),
                (Rotation::Right, PinType::Output) => {
                    (gate_rect.x + offset, gate_rect.bottom() - h)
                }

                // --- DOWN (180 deg) ---
                // Input on Right edge, Output on Left edge
                (Rotation::Down, PinType::Input) => (gate_rect.right() - w, gate_rect.y + offset),
                (Rotation::Down, PinType::Output) => (gate_rect.x, gate_rect.y + offset),

                // --- LEFT (270 deg) ---
                // Input on Bottom edge, Output on Top edge
                (Rotation::Left, PinType::Input) => (gate_rect.x + offset, gate_rect.bottom() - h),
                (Rotation::Left, PinType::Output) => (gate_rect.x + offset, gate_rect.y),
            };

            Rect { x, y, w, h }
        }

        let mut input: Vec<Pin> = vec![];
        let input_count = gate_type.input_count();

        for index in 0..input_count {
            let pin_rect = get_pin_rect(
                gate_rect,
                PinType::Input,
                rotation.clone(),
                index,
                input_count,
            );
            input.push(Pin {
                rect: pin_rect,
                index: index,
                wire_index: None,
            });
        }

        let mut output: Vec<Pin> = vec![];
        let output_count = gate_type.output_count();

        for index in 0..output_count {
            let pin_rect = get_pin_rect(
                gate_rect,
                PinType::Output,
                rotation.clone(),
                index,
                output_count,
            );
            output.push(Pin {
                rect: pin_rect,
                index: index,
                wire_index: None,
            });
        }

        return (input, output);
    }

    pub fn offset(&mut self, offset: Vec2) {
        self.rect = self.rect.offset(offset);

        for pin in &mut self.input {
            pin.rect = pin.rect.offset(offset);
        }

        for pin in &mut self.output {
            pin.rect = pin.rect.offset(offset);
        }
    }

    pub fn change_rotation(&mut self, new_rotation: Rotation) {
        self.rotation = new_rotation.clone();
        let (new_inputs, new_outputs) = Self::get_pins(self.rect, self.gate_type.clone(), self.rotation.clone());
        for (index, pin) in self.input.iter_mut().enumerate() {
            pin.rect = new_inputs[index].rect.clone();
        }
        for (index, pin) in self.output.iter_mut().enumerate() {
            pin.rect = new_outputs[index].rect.clone();
        }
    }

    pub fn get_pin_rect(&self, pin_index: usize, pin_type: PinType) -> Rect {
        return match pin_type {
            PinType::Input => self.input[pin_index].rect,
            PinType::Output => self.output[pin_index].rect,
        };
    }
    // maybe not needed
    pub fn get_pin(&self, pin_index: usize, pin_type: PinType) -> Pin {
        // if not in bounds return None??? FIX
        return match pin_type {
            PinType::Input => self.input[pin_index].clone(),
            PinType::Output => self.output[pin_index].clone(),
        };
    }
    // doesnt include the pins for some custom z handling
    pub fn draw(&self, camera_view_rect: Rect, color: Color) {
        if intersects(self.rect, camera_view_rect) {

            let text = GateType::text(&self.gate_type);

            draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, color);
            let dims = measure_text(text, None, FONT_SIZE, 1.0);
            let tx = self.rect.x + self.rect.w * 0.5 - dims.width * 0.5;
            let ty = self.rect.y + self.rect.h * 0.5 + dims.offset_y * 0.5 as f32;

            draw_text_ex(
                text,
                tx,
                ty,
                TextParams {
                    font_size: FONT_SIZE,
                    color: BLACK,
                    ..Default::default()
                },
            );
        }
    }

    pub fn draw_pins(&self, circuit: &Circuit, camera_view_rect: Rect, base_color: Color) {
        let pins = self.input.iter().chain(self.output.iter());

        for pin in pins {
            let pin_rect = pin.rect;
            if intersects(pin_rect, camera_view_rect) {
                let mut color = base_color;
                if let Some(idx) = pin.wire_index {
                    if *circuit.wires_read.get(idx).unwrap() == true {
                        color = BLACK.lerp(YELLOW, 0.4);
                    }
                }
                draw_rectangle(pin_rect.x, pin_rect.y, pin_rect.w, pin_rect.h, color);
            }
        }
    }

    pub fn draw_wires(&self, circuit: &Circuit, camera_view_rect: Rect) {
        for pin in &self.output {
            let center = pin.rect.center();
            if let Some(wire_index) = pin.wire_index {
                for conn in &circuit.wires.get(wire_index).unwrap().connections {
                    let out_gate = circuit.gates.get(conn.gate_index).unwrap();
                    let out_pin_center = out_gate
                        .get_pin(conn.pin_index, PinType::Input)
                        .rect
                        .center();
                    // draw line between out_pin_center, center
                    let color = match circuit.wires_read.get(wire_index).unwrap() {
                        true => YELLOW,
                        false => BLACK,
                    };

                    draw_line(
                        center.x,
                        center.y,
                        out_pin_center.x,
                        out_pin_center.y,
                        3.0,
                        color,
                    );
                }
            }
        }

        for pin in &self.input {
            let center = pin.rect.center();
            if let Some(wire_index) = pin.wire_index {
                let conn = &circuit.wires.get(wire_index).unwrap().source;
                let in_gate = circuit.gates.get(conn.gate_index).unwrap();
                let in_pin_center = in_gate
                    .get_pin(conn.pin_index, PinType::Output)
                    .rect
                    .center();
                // draw line between in_pin_center, center
                let color = match circuit.wires_read.get(wire_index).unwrap() {
                    true => YELLOW,
                    false => BLACK,
                };

                draw_line(
                    center.x,
                    center.y,
                    in_pin_center.x,
                    in_pin_center.y,
                    3.0,
                    color,
                );
            }
        }
    }
}
