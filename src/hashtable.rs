#![allow(dead_code)]

use crate::value::Value;

const TABLE_MAX_LOAD: f32 = 0.75;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct HashKeyString {
    value: String,
    hash: u64,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Entry {
    key: HashKeyString,
    value: Value,
}

#[derive(PartialEq, Debug, Clone)]
pub struct HashTable {
    entries: Vec<Entry>,
    count: usize,
    capacity: usize,
}

impl HashTable {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            count: 0,
            capacity: 0,
        }
    }

    fn insert(&mut self, key: HashKeyString, value: Value) {
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
                let element = Entry { key, value };
                self.entries.insert(index, element);
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
            let mut index = entry.key.hash as usize % capacity;

            while index < capacity {
                if entries.get(index).unwrap().value != Value::Nil {
                    entries[index] = entry.clone();
                }
                index += 1;
            }
        }

        self.entries = entries;
        self.capacity = capacity;
    }

    fn hash(key: &str) -> u64 {
        let mut hash = 0xcbf29ce484222325;

        for c in key.as_bytes() {
            hash ^= *c as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_table() {
        let mut table = HashTable::new();
        let key = HashKeyString {
            value: "hello".to_string(),
            hash: HashTable::hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);
    }

    #[test]
    fn test_hash() {
        let hash = HashTable::hash("hello");
        assert_eq!(hash, 11831194018420276491);
    }

    #[test]
    fn test_hash_table_insert_duplicate() {
        let mut table = HashTable::new();
        let key = HashKeyString {
            value: "hello".to_string(),
            hash: HashTable::hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);

        let key = HashKeyString {
            value: "hello".to_string(),
            hash: HashTable::hash("hello"),
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
            hash: HashTable::hash("hello"),
        };
        table.insert(key, Value::Number(1.0));
        assert_eq!(table.count, 1);
        assert_eq!(table.capacity, 8);

        let key = HashKeyString {
            value: "hello2".to_string(),
            hash: HashTable::hash("hello2"),
        };
        table.insert(key, Value::Number(2.0));
        let key = HashKeyString {
            value: "hello3".to_string(),
            hash: HashTable::hash("hello3"),
        };
        table.insert(key, Value::Number(3.0));
        let key = HashKeyString {
            value: "hello4".to_string(),
            hash: HashTable::hash("hello4"),
        };
        table.insert(key, Value::Number(4.0));
        let key = HashKeyString {
            value: "hello5".to_string(),
            hash: HashTable::hash("hello5"),
        };
        table.insert(key, Value::Number(5.0));
        let key = HashKeyString {
            value: "hello6".to_string(),
            hash: HashTable::hash("hello6"),
        };
        table.insert(key, Value::Number(6.0));
        let key = HashKeyString {
            value: "hello7".to_string(),
            hash: HashTable::hash("hello7"),
        };
        table.insert(key, Value::Number(7.0));
        let key = HashKeyString {
            value: "hello8".to_string(),
            hash: HashTable::hash("hello8"),
        };
        table.insert(key, Value::Number(8.0));
        assert_eq!(table.count, 8);
        assert_eq!(table.capacity, 16);
    }
}
