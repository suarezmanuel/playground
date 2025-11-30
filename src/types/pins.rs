#[derive(Clone)]
pub enum Pins {
    // Input and Output pins for gates
    Empty,
    Single(usize),
    Dual(usize, usize),
    Triple(usize, usize, usize),
    Variadic(Vec<usize>),
}
