use crate::value::Value;

#[derive(Default, Debug, Clone)]
pub struct Stack {
    pub values: Vec<Value>,
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
        if distance >= self.values.len() {
            None
        } else {
            self.values.get(self.values.len() - 1 - distance)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut stack = Stack::new();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));
        assert_eq!(stack.len(), 3);
    }

    #[test]
    fn test_pop() {
        let mut stack = Stack::new();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));
        assert_eq!(stack.pop(), Some(Value::Number(3.0)));
        assert_eq!(stack.pop(), Some(Value::Number(2.0)));
        assert_eq!(stack.pop(), Some(Value::Number(1.0)));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_peek() {
        let mut stack = Stack::new();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));
        assert_eq!(stack.peek(0), Some(&Value::Number(3.0)));
        assert_eq!(stack.peek(1), Some(&Value::Number(2.0)));
        assert_eq!(stack.peek(2), Some(&Value::Number(1.0)));
        assert_eq!(stack.peek(3), None);
    }

    #[test]
    fn test_len() {
        let mut stack = Stack::new();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));
        assert_eq!(stack.len(), 3);
    }

    #[test]
    fn test_reset() {
        let mut stack = Stack::new();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));
        stack.reset();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let mut stack = Stack::new();
        assert!(stack.is_empty());
        stack.push(Value::Number(1.0));
        assert!(!stack.is_empty());
        stack.pop();
        assert!(stack.is_empty());
    }
}
