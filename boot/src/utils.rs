use alloc::vec::Vec;

fn pattern_to_hex(pattern: &str) -> Vec<Option<u8>> {
    let mut result = Vec::new();
    pattern
        .split_ascii_whitespace()
        .for_each(|char| match char {
            "?" => result.push(None),
            other => result.push(Some(
                u8::from_str_radix(other, 16).expect("Invalid signature"),
            )),
        });
    result
}

pub fn find_pattern(pattern: &str, data: &[u8]) -> Option<usize> {
    let pattern = pattern_to_hex(pattern);
    data.windows(pattern.len()).position(|window| {
        window
            .iter()
            .zip(pattern.iter())
            .all(|(byte, pattern_byte)| pattern_byte.map_or(true, |b| *byte == b))
    })
}