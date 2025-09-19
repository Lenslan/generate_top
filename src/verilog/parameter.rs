

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub value: usize,
}

impl Param {
    pub fn new(name: String, value: usize) -> Param {
        Param { name, value }
    }
}