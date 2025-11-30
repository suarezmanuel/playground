#[derive(Debug, Clone, Copy)]
pub enum PinType {
    Input,
    Output,
}

impl PinType {
    pub fn to_string(self) -> String {
        return match self {
            PinType::Input => {"input".to_owned()}
            PinType::Output => {"output".to_owned()}
        }
    }
}
