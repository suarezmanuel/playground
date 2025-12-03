use crate::types::gate_type::*;
use crate::types::pin_type::*;
use crate::types::pins::*;
use macroquad::prelude::*;

const GATE_SIZE: u16 = 64;
const PIN_SIZE: u16 = 6;
const PIN_PIXEL_SIDE_LEN: f32 = PIN_SIZE as f32;

#[derive(Clone, Debug)]
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
}

#[derive(Clone, Debug)]
pub struct Gate {
    pub rotation: Rotation,
    pub rect: Rect,
    pub input: Pins,
    pub output: Pins,
    pub gate_type: GateType,
}

impl Gate {
    pub fn new(rect: Rect, rotation: Rotation, gate_type: GateType) -> Gate {
        let (input, output) = Self::get_pins(rect, Rotation::Up, gate_type);
        println!("rect x: {} y: {}", rect.x, rect.y);
        return Gate {
            rotation: rotation,
            rect: rect,
            input: input,
            output: output,
            gate_type: gate_type,
        };
    }

    pub fn get_pins(
        gate_rect: Rect,
        rotation: Rotation,
        gate_type: GateType,
    ) -> (Vec<Pin>, Vec<Pin>) {
        fn get_pin_rect(tl_x: f32, tl_y: f32, pin_index: usize, pin_count: usize) -> Rect {
            let spaces_count = (pin_count + 1) as f32;
            let space_pixel_len =
                (GATE_SIZE as f32 - (pin_count as f32) * PIN_PIXEL_SIDE_LEN) / spaces_count;

            return Rect {
                x: tl_x,
                y: tl_y
                    + space_pixel_len * ((pin_index + 1) as f32)
                    + PIN_PIXEL_SIDE_LEN * (pin_index as f32),
                w: PIN_PIXEL_SIDE_LEN,
                h: PIN_PIXEL_SIDE_LEN,
            };
        }

        let mut input: Vec<Pin> = vec![];
        let input_count = gate_type.input_count();

        for index in 0..input_count {
            let pin_rect = get_pin_rect(gate_rect.x, gate_rect.y, index, input_count);
            input.push(Pin {
                rect: pin_rect,
                index: index,
                pin_type: PinType::Input,
                wire_index: None,
            });
        }

        let mut output: Vec<Pin> = vec![];
        let output_count = gate_type.output_count();

        for index in 0..output_count {
            let pin_rect = get_pin_rect(
                gate_rect.x + gate_rect.w - PIN_PIXEL_SIDE_LEN,
                gate_rect.y,
                index,
                output_count,
            );
            output.push(Pin {
                rect: pin_rect,
                index: index,
                pin_type: PinType::Output,
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

    pub fn get_pin_rect(&self, pin_index: usize, pin_type: PinType) -> Rect {
        let pin_count = match pin_type {
            PinType::Input => self.input.len(),
            PinType::Output => self.output.len(),
        };

        let (tl_x, tl_y) = match pin_type {
            PinType::Input => (self.rect.x, self.rect.y),
            PinType::Output => (self.rect.x + self.rect.w - PIN_PIXEL_SIDE_LEN, self.rect.y),
        };

        let spaces_count = (pin_count + 1) as f32;
        let space_pixel_len =
            (GATE_SIZE as f32 - (pin_count as f32) * PIN_PIXEL_SIDE_LEN) / spaces_count;

        return Rect {
            x: tl_x,
            y: tl_y
                + space_pixel_len * ((pin_index + 1) as f32)
                + PIN_PIXEL_SIDE_LEN * (pin_index as f32),
            w: PIN_PIXEL_SIDE_LEN,
            h: PIN_PIXEL_SIDE_LEN,
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
}
