#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Deault,
    Bool(bool),
    Nil,
    Number(f64),
}
