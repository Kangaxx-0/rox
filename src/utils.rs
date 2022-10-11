use crate::value::Value;

pub fn convert_slice_to_string(source: &[u8], start: usize, end: usize) -> String {
    String::from_utf8(source[start..end].to_vec()).expect("cannot get string value")
}

pub fn is_falsey(value: &Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Bool(b) => !b,
        _ => false,
    }
}

pub fn hash(key: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325;

    for c in key.as_bytes() {
        hash ^= *c as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
