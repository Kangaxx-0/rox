#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Deault,
    Bool(bool),
    Nil,
    Number(f64),
    String(String),
}
