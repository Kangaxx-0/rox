#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}
