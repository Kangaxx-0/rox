pub fn convert_arrayslice_to_string(source: &[u8], start: usize, end: usize) -> String {
    String::from_utf8(source[start..end].to_vec()).expect("cannot get string value")
}
