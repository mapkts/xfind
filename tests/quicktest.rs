//! We test `xfind` against `memchr` with high frequency words to see if their results match.
use std::fs::File;
use std::io::{prelude::*, SeekFrom};

#[test]
fn test_find_iter_ch1() {
    let needle = b"a";

    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("test file not found");
    let mut buf = Vec::with_capacity(1000000);
    f.read_to_end(&mut buf).unwrap();

    f.seek(SeekFrom::Start(0)).expect("I/O operation failed");
    let matches: Vec<usize> = xfind::find_iter(needle, &mut f)
        .map(|x| x.expect("I/O operation failed"))
        .collect();
    let expected: Vec<usize> =
        memchr::memmem::find_iter(&buf, needle).collect();

    assert_eq!(matches, expected);
}

#[test]
fn test_find_iter_ch2() {
    let needle = b"of";

    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("test file not found");
    let mut buf = Vec::with_capacity(1000000);
    f.read_to_end(&mut buf).unwrap();

    f.seek(SeekFrom::Start(0)).expect("I/O operation failed");
    let matches: Vec<usize> = xfind::find_iter(needle, &mut f)
        .map(|x| x.expect("I/O operation failed"))
        .collect();
    let expected: Vec<usize> =
        memchr::memmem::find_iter(&buf, needle).collect();

    assert_eq!(matches, expected);
}

#[test]
fn test_find_iter_ch3() {
    let needle = b"the";

    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("test file not found");
    let mut buf = Vec::with_capacity(1000000);
    f.read_to_end(&mut buf).unwrap();

    f.seek(SeekFrom::Start(0)).expect("I/O operation failed");
    let matches: Vec<usize> = xfind::find_iter(needle, &mut f)
        .map(|x| x.expect("I/O operation failed"))
        .collect();
    let expected: Vec<usize> =
        memchr::memmem::find_iter(&buf, needle).collect();

    assert_eq!(matches, expected);
}

#[test]
fn test_find_iter_ch4() {
    let needle = b"dear";

    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("test file not found");
    let mut buf = Vec::with_capacity(1000000);
    f.read_to_end(&mut buf).unwrap();

    f.seek(SeekFrom::Start(0)).expect("I/O operation failed");
    let matches: Vec<usize> = xfind::find_iter(needle, &mut f)
        .map(|x| x.expect("I/O operation failed"))
        .collect();
    let expected: Vec<usize> =
        memchr::memmem::find_iter(&buf, needle).collect();

    assert_eq!(matches, expected);
}

#[test]
fn test_rfind_iter_ch1() {
    let needle = b"a";

    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("test file not found");
    let mut buf = Vec::with_capacity(1000000);
    f.read_to_end(&mut buf).unwrap();

    f.seek(SeekFrom::Start(0)).expect("I/O operation failed");
    let matches: Vec<usize> = xfind::rfind_iter(needle, &mut f)
        .expect("I/O operation failed")
        .map(|x| x.expect("I/O operation failed"))
        .collect();
    let expected: Vec<usize> =
        memchr::memmem::find_iter(&buf, needle).collect();
    let expected: Vec<usize> = expected.into_iter().rev().collect();

    assert_eq!(matches, expected);
}

#[test]
fn test_rfind_iter_ch2() {
    let needle = b"of";

    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("test file not found");
    let mut buf = Vec::with_capacity(1000000);
    f.read_to_end(&mut buf).unwrap();

    f.seek(SeekFrom::Start(0)).expect("I/O operation failed");
    let matches: Vec<usize> = xfind::rfind_iter(needle, &mut f)
        .expect("I/O operation failed")
        .map(|x| x.expect("I/O operation failed"))
        .collect();
    let expected: Vec<usize> =
        memchr::memmem::find_iter(&buf, needle).collect();
    let expected: Vec<usize> = expected.into_iter().rev().collect();

    assert_eq!(matches, expected);
}

#[test]
fn test_rfind_iter_ch3() {
    let needle = b"the";

    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("test file not found");
    let mut buf = Vec::with_capacity(1000000);
    f.read_to_end(&mut buf).unwrap();

    f.seek(SeekFrom::Start(0)).expect("I/O operation failed");
    let matches: Vec<usize> = xfind::rfind_iter(needle, &mut f)
        .expect("I/O operation failed")
        .map(|x| x.expect("I/O operation failed"))
        .collect();
    let expected: Vec<usize> =
        memchr::memmem::find_iter(&buf, needle).collect();
    let expected: Vec<usize> = expected.into_iter().rev().collect();

    assert_eq!(matches, expected);
}

#[test]
fn test_rfind_iter_ch4() {
    let needle = b"dear";

    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("test file not found");
    let mut buf = Vec::with_capacity(1000000);
    f.read_to_end(&mut buf).unwrap();

    f.seek(SeekFrom::Start(0)).expect("I/O operation failed");
    let matches: Vec<usize> = xfind::rfind_iter(needle, &mut f)
        .expect("I/O operation failed")
        .map(|x| x.expect("I/O operation failed"))
        .collect();
    let expected: Vec<usize> =
        memchr::memmem::find_iter(&buf, needle).collect();
    let expected: Vec<usize> = expected.into_iter().rev().collect();

    assert_eq!(matches, expected);
}
