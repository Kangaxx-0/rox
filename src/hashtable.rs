#![allow(dead_code)]

use crate::value::Value;

const TABLE_MAX_LOAD: f32 = 0.75;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct HashKeyString {
    value: String,
    hash: usize,
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
            Some(index) => {
                self.entries[index].value = value;
            }
            None => {
                self.entries.push(Entry { key, value });
                self.count += 1;
            }
        }
    }

    fn find_entry(&self, key: &HashKeyString) -> Option<usize> {
        let mut index = key.hash % self.capacity;

        while index < self.capacity {
            let entry = &self.entries[index];

            if entry.key == *key {
                return Some(index);
            }

            index = (index + 1) % self.capacity;
        }

        None
    }

    fn grow_capacity(&self) -> usize {
        if self.capacity < 8 {
            8
        } else {
            self.capacity * 2
        }
    }

    fn resize(&mut self, capaicty: usize) {
        let mut entries: Vec<Entry> = Vec::with_capacity(capaicty);
        for entry in self.entries.iter() {
            let mut index = entry.key.hash % capaicty;

            while index < capaicty {
                entries[index] = entry.clone();

                index = (index + 1) % capaicty;
            }
        }

        self.entries = entries;
        self.capacity = capaicty;
    }
}
