pub struct Connection {
    pub pin_index: usize,
    pub gate_index: usize
    // all connections are input types
}

pub struct Wire {
    pub source: Connection, // output type connection
    pub connections: Vec<Connection>,
    pub wire_index: usize,
}

pub trait ConnectionUtils {
    fn find_pin_index (&self, gate_index: usize, pin_index: usize) -> Option<usize>;
}

impl ConnectionUtils for [Connection] {
    fn find_pin_index (&self, gate_index: usize, pin_index: usize) -> Option<usize> {
        let mut result_index: Option<usize> = None;
        for i in 0..self.len() {
            if self[i].gate_index == gate_index && self[i].pin_index == pin_index {
                result_index = Some(i);
            }
        }
        return result_index;
    }
}