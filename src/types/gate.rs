use macroquad::prelude::*;
use crate::types::pins::*;
use crate::types::pin_type::*;
use crate::types::spatial_pin_index::*;
use crate::types::gate_type::*;

#[derive(Clone)]
pub struct Gate {
    pub rect: Rect,
    pub input: Pins,
    pub output: Pins,
    pub gate_type: GateType,
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