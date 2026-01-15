use astra_stack::{process_stream, Deduplicator, TransformProfile, transform_line};

#[test]
fn upper_trim_and_drop() {
    let profile = TransformProfile {
        trim: true,
        to_upper: true,
        drop_empty: true,
        deduplicate: false,
    };
    let mut dedup = Deduplicator::new();
    let out = transform_line("  hola  ", &profile, &mut dedup).unwrap();
    assert_eq!(out, "HOLA");
}

#[test]
fn stream_deduplication() {
    let input = "uno\ndos\nuno\n".as_bytes();
    let mut output = Vec::new();
    let profile = TransformProfile {
        trim: true,
        to_upper: false,
        drop_empty: true,
        deduplicate: true,
    };
    let stats = process_stream(input, &mut output, profile).unwrap();
    assert_eq!(stats.read, 3);
    assert_eq!(stats.written, 2);
    assert_eq!(stats.skipped, 1);
    let result = String::from_utf8(output).unwrap();
    assert_eq!(result, "uno\ndos\n");
}
