use std::collections::HashMap;
// (arduino pin, cc)S
#[derive(Debug)]
pub struct Faders {
    pub pins: HashMap<u8, u8>
}

impl Faders {
    pub fn default() -> Self {
        Self {
            pins: HashMap::from([(18, 1), (19, 11), (20, 2), (21, 3)])
        }
    }
}
