use crate::value::Value;

#[derive(Default, Debug, Clone)]
pub struct Stack {
    values: Vec<Value>,
}

impl Iterator for Stack {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.pop()
    }
}

impl Stack {
    pub fn new() -> Stack {
        Stack { values: Vec::new() }
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.values.pop()
    }

    pub fn peek(&self, distance: usize) -> Option<&Value> {
        self.values.get(self.values.len() - distance - 1)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn reset(&mut self) {
        self.values.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
