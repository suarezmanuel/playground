use crate::types::gate::*;
use macroquad::prelude::*;

struct GameState {
    pub dragging_wire: bool,
    pub current_wire_start: u32,
    pub current_wire_end: u32,
}

// pub fn startWireDragVisual(mut game_state: GameState, gateOut: Gate) {
//     // can just create a wire in the air, not necessarily on gates
//     game_state.dragging_wire = true;
// }

// pub fn finalizeWireVisual(mut game_state: GameState, gateOut: Gate) {
//     // if linked to gate
//     game_state.dragging_wire = false
// }

// // // look at all the
// // pub fn removeWire(gateOut : gate, gateIn : gate) {

// // }

// pub fn renderCurrentWire() {
//     // use manhattan thing
// }
