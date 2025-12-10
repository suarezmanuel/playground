use crate::types::keys::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    pub pin_index: usize,
    pub gate_index: GateKey, // all connections are input types
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wire {
    pub source: Connection, // output type connection
    pub connections: Vec<Connection>,
}

impl Wire {
    pub fn new(source: Connection, connections: Vec<Connection>) -> Wire {
        return Wire{source: source, connections: connections};
    }
}

pub trait ConnectionUtils {
    fn find_pin_index(&self, gate_index: GateKey, pin_index: usize) -> Option<usize>;
}

impl ConnectionUtils for [Connection] {
    fn find_pin_index(&self, gate_index: GateKey, pin_index: usize) -> Option<usize> {
        let mut result_index: Option<usize> = None;
        for i in 0..self.len() {
            if self[i].gate_index == gate_index && self[i].pin_index == pin_index {
                result_index = Some(i);
            }
        }
        return result_index;
    }
}
