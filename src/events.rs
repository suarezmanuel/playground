use macroquad::prelude::*;
use crate::types::*;

struct game_state {
    pub dragging_wire : bool,
    pub current_wire_start : u32,
    pub current_wire_end : u32,
}

pub fn startWireDragVisual(mut game_state : game_state, gateOut : gate) {
    // can just create a wire in the air, not necessarily on gates
    game_state.dragging_wire = true;
}

pub fn finalizeWireVisual(mut game_state : game_state, gateOut : gate) {
    // if linked to gate 
   game_state.dragging_wire = false
}

// // look at all the 
// pub fn removeWire(gateOut : gate, gateIn : gate) {

// }

pub fn renderCurrentWire() {
    // use manhattan thing
}