use std::{fs::remove_file, path::Path};

use image::io::Reader;

use bpg::process::process_and_save_local;

#[test]
fn test_process_and_save_local() {
    process_and_save_local(Path::new("tests/emoji.png"), 100).unwrap();
    let result = Reader::open("tests/emoji_bordered.png")
        .unwrap()
        .decode()
        .unwrap();
    let expected = Reader::open("tests/emoji_test.png")
        .unwrap()
        .decode()
        .unwrap();
    assert_eq!(result, expected);
    remove_file("tests/emoji_bordered.png").unwrap();
}
