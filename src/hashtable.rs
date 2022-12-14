#![allow(dead_code)]

use std::fmt::Display;

use crate::objects::HashKeyString;
use crate::value::Value;

const TABLE_MAX_LOAD: f32 = 0.75;

#[derive(PartialEq, Clone)]
pub struct Entry {
    key: HashKeyString,
    value: Value,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entry {{ key: {:?}, value: {:?} }}",
            self.key, self.value
        )
    }
}

#[derive(PartialEq, Clone)]
pub struct HashTable {
    entries: Vec<Entry>,
    count: usize,
    capacity: usize,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            count: 0,
            capacity: 0,
        }
    }

    pub fn insert(&mut self, key: HashKeyString, value: Value) {
        let threshold = (self.capacity as f32 * TABLE_MAX_LOAD) as usize;
        if self.count + 1 > threshold {
            let capaicty = self.grow_capacity();
            self.resize(capaicty);
        }
        match self.find_entry(&key) {
            (Some(_), index) => {
                self.entries[index].value = value;
            }
            (None, index) => {
                let mut element = Entry { key, value };
                // We want to replace the value, but keep the vec capacity the same.
                std::mem::swap(&mut self.entries[index], &mut element);
                self.count += 1;
            }
        }
    }

    fn find_entry(&self, key: &HashKeyString) -> (Option<()>, usize) {
        let mut index = key.hash as usize % (self.capacity - 1);

        while index < self.capacity {
            if self.entries[index].value == Value::Nil {
                return (None, index);
            } else {
                let entry = &self.entries[index];

                if entry.key == *key {
                    return (Some(()), index);
                }
                index = (index + 1) % self.capacity;
            }
        }

        (None, index)
    }

    pub fn get(&self, key: &HashKeyString) -> Option<&Value> {
        if self.count == 0 {
            return None;
        }
        let (found, index) = self.find_entry(key);
        if found.is_some() {
            Some(&self.entries[index].value)
        } else {
            None
        }
    }

    fn remove(&mut self, key: &HashKeyString) -> Option<Value> {
        if self.count == 0 {
            return None;
        }
        let (found, index) = self.find_entry(key);
        if found.is_some() {
            let value = self.entries[index].value.clone();
            self.entries[index].value = Value::Nil;
            self.count -= 1;
            Some(value)
        } else {
            None
        }
    }

    fn grow_capacity(&self) -> usize {
        if self.capacity < 8 {
            8
        } else {
            self.capacity * 2
        }
    }

    fn resize(&mut self, capacity: usize) {
        let mut entries = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            entries.push(Entry {
                key: HashKeyString {
                    value: String::new(),
                    hash: 0,
                },
                value: Value::Nil,
            });
        }

        for entry in self.entries.iter() {
            if entry.value != Value::Nil {
                let index = entry.key.hash as usize % (capacity - 1);
                entries[index] = entry.clone();
            }
        }

        self.entries = entries;
        self.capacity = capacity;
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn remove_all(&mut self) {
        self.entries.clear();
        self.count = 0;
        self.capacity = 0;
    }

    fn print(&self) {
        for entry in self.entries.iter() {
            if entry.value != Value::Nil {
                println!("{}", entry);
            }
        }
    }
}

impl Default for HashTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::hash;

    use super::*;

    #[test]
    fn test_hash_table() {
        let mut table = HashTable::new();
        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);
    }

    #[test]
    fn test_hash() {
        let hash = hash("hello");
        assert_eq!(hash, 11831194018420276491);
    }

    #[test]
    fn test_hash_table_insert_duplicate() {
        let mut table = HashTable::new();
        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);

        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        table.insert(key, Value::Number(2.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);
    }

    #[test]
    fn test_hash_table_insert_resize() {
        let mut table = HashTable::new();
        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);

        let key = HashKeyString {
            value: "hello2".to_string(),
            hash: hash("hello2"),
        };
        table.insert(key, Value::Number(2.0));
        let key = HashKeyString {
            value: "hello3".to_string(),
            hash: hash("hello3"),
        };
        table.insert(key, Value::Number(3.0));
        let key = HashKeyString {
            value: "hello4".to_string(),
            hash: hash("hello4"),
        };
        table.insert(key, Value::Number(4.0));
        let key = HashKeyString {
            value: "hello5".to_string(),
            hash: hash("hello5"),
        };
        table.insert(key, Value::Number(5.0));
        let key = HashKeyString {
            value: "hello6".to_string(),
            hash: hash("hello6"),
        };
        table.insert(key, Value::Number(6.0));
        let key = HashKeyString {
            value: "hello7".to_string(),
            hash: hash("hello7"),
        };
        table.insert(key, Value::Number(7.0));
        let key = HashKeyString {
            value: "hello8".to_string(),
            hash: hash("hello8"),
        };
        table.insert(key, Value::Number(8.0));
        assert_eq!(table.count, 8);
        assert_eq!(table.capacity, 16);
    }

    #[test]
    fn test_hash_table_get() {
        let mut table = HashTable::new();
        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);

        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        let value = table.get(&key);
        assert_eq!(value, Some(&Value::Number(1.0)));
    }

    #[test]
    fn test_hash_table_get_not_found() {
        let mut table = HashTable::new();
        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);

        let key = HashKeyString {
            value: "hello2".to_string(),
            hash: hash("hello2"),
        };
        let value = table.get(&key);
        assert_eq!(value, None);
    }

    #[test]
    fn test_hash_table_remove() {
        let mut table = HashTable::new();
        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);

        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        let value = table.remove(&key);
        assert_eq!(value, Some(Value::Number(1.0)));
        assert_eq!(table.count, 0);
        assert_eq!(table.capacity, 8);
    }

    #[test]
    fn test_hash_table_remove_not_found() {
        let mut table = HashTable::new();
        let key = HashKeyString {
            value: "hello".to_string(),
            hash: hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);

        let key = HashKeyString {
            value: "hello2".to_string(),
            hash: hash("hello2"),
        };
        let value = table.remove(&key);
        assert_eq!(value, None);
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);
    }
}
