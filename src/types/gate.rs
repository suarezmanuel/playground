use crate::types::gate_type::*;
use crate::types::pin_type::*;
use crate::types::pins::*;
use crate::types::spatial_pin_index::*;
use macroquad::prelude::*;

const GATE_SIZE: u16 = 64;
const PIN_SIZE: u16 = 6;

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
            GateType::PWR => Vec::new(),
            GateType::NOT | GateType::GND => {
                vec![Pin {
                    index: 0,
                    pin_type: PinType::Input,
                    other_pin_index: None,
                    other_pin_type: None,
                    other_gate_index: None,
                    wire_index: None,
                }]
            }
            // the gates for pins need to be assigned by circuit
            _ => {
                vec![
                    Pin {
                        index: 0,
                        pin_type: PinType::Input,
                        other_pin_index: None,
                        other_pin_type: None,
                        other_gate_index: None,
                        wire_index: None,
                    },
                    Pin {
                        index: 1,
                        pin_type: PinType::Input,
                        other_pin_index: None,
                        other_pin_type: None,
                        other_gate_index: None,
                        wire_index: None,
                    },
                ]
            }
        };

        let output = match gate_type {
            GateType::GND => Vec::new(),
            _ => vec![Pin {
                index: 0,
                pin_type: PinType::Output,
                other_pin_index: None,
                other_pin_type: None,
                other_gate_index: None,
                wire_index: None,
            }],
        };

        return Gate {
            rect: rect,
            input: input,
            output: output,
            gate_type: gate_type,
        };
    }

    pub fn get_pin(&self, pin_index: usize, pin_type: PinType) -> Pin {
        return match pin_type {
            PinType::Input => self.input[pin_index].clone(),
            PinType::Output => self.output[pin_index].clone(),
        };
    }

    pub fn get_pin_block(&self, pin: Pin) -> SpatialPinIndex {
        let pin_count = match pin.pin_type {
            PinType::Input => self.input.len(),
            PinType::Output => self.output.len(),
        };

        let pin_pixel_side_len = PIN_SIZE as f32;
        let spaces_count = (pin_count + 1) as f32;
        let space_pixel_len = (GATE_SIZE as f32 - (pin_count as f32) * pin_pixel_side_len) / spaces_count;

        let (tl_x, tl_y) = match pin.pin_type {
            PinType::Input => (self.rect.x, self.rect.y),
            PinType::Output => (self.rect.x + self.rect.w - pin_pixel_side_len, self.rect.y),
        };

        return SpatialPinIndex {
            rect: Rect {
                x: tl_x,
                // camera is upside down
                y: tl_y + GATE_SIZE as f32
                    - space_pixel_len * (pin.index as f32 + 1.0)
                    - pin_pixel_side_len * (pin.index as f32 + 1.0),
                w: pin_pixel_side_len,
                h: pin_pixel_side_len,
            },
            index: pin.index,
            pin_type: pin.pin_type,
        };
    }

    pub fn get_side_pins_blocks(&self, pin_type: PinType) -> Vec<SpatialPinIndex> {
        let tl_x;
        let tl_y;
        let pin_count: usize;

        match pin_type {
            PinType::Input => {
                tl_x = self.rect.x;
                tl_y = self.rect.y;
                pin_count = self.input.len();
            }
            PinType::Output => {
                tl_x = self.rect.x + self.rect.w - PIN_SIZE as f32;
                tl_y = self.rect.y;
                pin_count = self.output.len();
            }
        }

        if pin_count > 8 {
            println!("WARNING: i hope youre not using gates that are 64x64");
        }

        let mut rects: Vec<SpatialPinIndex> = Vec::new();
        let pin_pixel_side_len = PIN_SIZE as f32;
        let spaces_count = (pin_count + 1) as f32;
        let space_pixel_len = (GATE_SIZE as f32 - (pin_count as f32) * pin_pixel_side_len) / spaces_count;

        for i in 1..=pin_count {
            rects.push(SpatialPinIndex {
                rect: Rect {
                    x: tl_x,
                    // camera is upside down
                    y: tl_y + GATE_SIZE as f32
                        - space_pixel_len * (i as f32)
                        - pin_pixel_side_len * (i as f32),
                    w: pin_pixel_side_len,
                    h: pin_pixel_side_len,
                },
                index: i - 1,
                pin_type: pin_type,
            });
        }

        return rects;
    }

    pub fn get_pins_blocks(self) -> Vec<SpatialPinIndex> {
        let mut rects: Vec<SpatialPinIndex> = Vec::new();
        // input pins
        rects.extend(self.get_side_pins_blocks(PinType::Input));

        // output pins
        rects.extend(self.get_side_pins_blocks(PinType::Output));

        return rects;
    }
}
