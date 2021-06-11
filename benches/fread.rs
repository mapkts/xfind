#![feature(test)]
extern crate test;

use std::fs::File;
use std::io::{self, Read};
use test::Bencher;

use aho_corasick::AhoCorasick;
use memchr::memmem;

#[bench]
fn stream_find_iter_xfind(b: &mut Bencher) {
    b.iter(|| {
        let mut f = File::open("data/pride-and-prejudice.txt")
            .expect("testing file is not existed");

        let _matches: Vec<io::Result<usize>> =
            xfind::find_iter(b"dear", &mut f).collect();
    });
}

#[bench]
fn stream_rfind_iter_xfind(b: &mut Bencher) {
    b.iter(|| {
        let mut f = File::open("data/pride-and-prejudice.txt")
            .expect("testing file is not existed");

        let _matches: Vec<io::Result<usize>> =
            xfind::rfind_iter(b"dear", &mut f).unwrap().collect();
    });
}

#[bench]
fn stream_find_iter_memchr(b: &mut Bencher) {
    b.iter(|| {
        let mut f = File::open("data/pride-and-prejudice.txt")
            .expect("testing file is not existed");
        let mut haystack = Vec::with_capacity(1000000);
        f.read_to_end(&mut haystack).unwrap();

        let _matches: Vec<usize> =
            memmem::find_iter(&haystack, b"dear").collect();
    });
}

#[bench]
fn stream_rfind_iter_memchr(b: &mut Bencher) {
    b.iter(|| {
        let mut f = File::open("data/pride-and-prejudice.txt")
            .expect("testing file is not existed");
        let mut haystack = Vec::with_capacity(1000000);
        f.read_to_end(&mut haystack).unwrap();

        let _matches: Vec<usize> =
            memmem::rfind_iter(&haystack, b"dear").collect();
    });
}

#[bench]
fn stream_find_iter_aho_corasick(b: &mut Bencher) {
    b.iter(|| {
        let f = File::open("data/pride-and-prejudice.txt")
            .expect("testing file is not existed");
        let patterns = &["dear"];
        let ac = AhoCorasick::new(patterns);

        let _matches: Vec<io::Result<aho_corasick::Match>> =
            ac.stream_find_iter(f).collect();
    });
}

#[bench]
fn memory_find_iter_xfind(b: &mut Bencher) {
    use std::io::{Seek, SeekFrom};

    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("testing file is not existed");
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let mut haystack = io::Cursor::new(buf);

    b.iter(|| {
        // We must move the cursor to the start before searching.
        haystack.seek(SeekFrom::Start(0)).unwrap();
        let _matches: Vec<io::Result<usize>> =
            xfind::find_iter(b"dear", &mut haystack).collect();
    });
}

#[bench]
fn memory_rfind_iter_xfind(b: &mut Bencher) {
    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("testing file is not existed");
    let mut haystack = Vec::new();
    f.read_to_end(&mut haystack).unwrap();
    let mut haystack = io::Cursor::new(haystack);

    b.iter(|| {
        let _matches: Vec<io::Result<usize>> =
            xfind::rfind_iter(b"dear", &mut haystack).unwrap().collect();
    });
}

#[bench]
fn memory_find_iter_memchr(b: &mut Bencher) {
    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("testing file is not existed");
    let mut haystack = Vec::new();
    f.read_to_end(&mut haystack).unwrap();

    b.iter(|| {
        let _matches: Vec<usize> =
            memmem::find_iter(&haystack, b"dear").collect();
    });
}

#[bench]
fn memory_rfind_iter_memchr(b: &mut Bencher) {
    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("testing file is not existed");
    let mut haystack = Vec::new();
    f.read_to_end(&mut haystack).unwrap();

    b.iter(|| {
        let _matches: Vec<usize> =
            memmem::rfind_iter(&haystack, b"dear").collect();
    });
}

#[bench]
fn memory_find_iter_aho_corasick(b: &mut Bencher) {
    let mut f = File::open("data/pride-and-prejudice.txt")
        .expect("testing file is not existed");
    let mut haystack = Vec::new();
    f.read_to_end(&mut haystack).unwrap();

    let patterns = &["dear"];
    let ac = AhoCorasick::new(patterns);

    b.iter(|| {
        let _matches: Vec<aho_corasick::Match> =
            ac.find_iter(&haystack).collect();
    });
}
