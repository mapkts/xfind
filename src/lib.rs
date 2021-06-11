//! Provides forward and backward substring searchers for stream searches.
//!
//! This crate is built on top of [`memchr`], a heavily optimized routines for string searches.
//! But unlike [`memchr`], utilties provided by this crate operate directly on stream (i.e.,
//! [`Read`] instances) rather than in-memory buffers.
//!
//! Note that this crate provides no advantage when searching substring in a source that is already
//! in memory, in this case consider using the [`memchr`] library instead. Besides, if you want to
//! search multiple substrings at once, take a look at [`aho-corasick`].
//!
//! # Complexity
//!
//! Forward and backward stream search routines provided by this crate is guaranteed to have worst
//! case linear time complexity with respect to both `needle.len()` and `stream.len()`, and worst
//! case constant space complexity with respect to `needle.len()`.
//!
//! # Performance
//!
//! Below is a collected benchmark result for searching all occurrences of `dear` in a 767KB book
//! [`Pride and Prejudice`](https://www.gutenberg.org/files/1342/1342-0.txt).
//!
//! ```text
//! test memory_find_iter_aho_corasick ... bench:     464,070 ns/iter (+/- 6,166)
//! test memory_find_iter_memmem       ... bench:      24,224 ns/iter (+/- 314)
//! test memory_find_iter_xfind        ... bench:      43,408 ns/iter (+/- 948)
//! test memory_rfind_iter_memmem      ... bench:     543,790 ns/iter (+/- 9,169)
//! test memory_rfind_iter_xfind       ... bench:     502,290 ns/iter (+/- 6,643)
//! test stream_find_iter_aho_corasick ... bench:     642,467 ns/iter (+/- 20,707)
//! test stream_find_iter_memmem       ... bench:      90,457 ns/iter (+/- 2,776)
//! test stream_find_iter_xfind        ... bench:     182,127 ns/iter (+/- 3,683)
//! test stream_rfind_iter_memmem      ... bench:     614,040 ns/iter (+/- 28,896)
//! test stream_rfind_iter_xfind       ... bench:     667,330 ns/iter (+/- 10,283)
//! ```
//!
//! - When performing forward stream searches, `xfind` is about 2.0x slower than `memchr::memmem`,
//! which is fair because `memmem` needs to read all contents of the file into a fairly large
//! pre-allocated buffer (>= 767KB) and operates over it, while `xfind` performs stream searches
//! using an 8KB-only buffer.
//!
//! - `xfind` provides no advantage when searching through in-memory buffers (nearly 1.8x slower),
//! so please don't use it for in-memory searches.
//!
//! - When searching only one substrings, `xfind` beats `aho-corasick` in all cases above, which is
//! still fair because `aho-corasick` is mainly used for searching multiple substrings at once.
//!
//! - Reverse stream searches are by its nature much slower than forward stream searches. The
//! performances of `xfind` and `memmem` are pretty close, only memory usages differ.
//!
//! [`memchr`]: https://crates.io/crates/memchr
//! [`aho-corasick`]: https://crates.io/crates/aho-corasick
//! [`Read`]: std::io::Read
//!
//! # Examples
//!
//! - Checks if a substring exists in a file.
//!
//! ```no_run
//! use std::fs::File;
//!
//! fn main() -> std::io::Result<()> {
//!     let mut rdr = File::open("foo.txt")?;
//!     let found = xfind::find(b"bar", &mut rdr).is_some();
//!
//!     Ok(())
//! }
//! ```
//!
//! - Gets the indexes of the first 10 occurrences of a substring in a file.
//!
//! ```no_run
//! use std::fs::File;
//! use std::io;
//!
//! fn main() -> io::Result<()> {
//!     let mut rdr = File::open("foo.txt")?;
//!     let indexes = xfind::find_iter(b"bar", &mut rdr)
//!         .take(10)
//!         .collect::<io::Result<Vec<usize>>>()?;
//!
//!     println!("{:?}", indexes);
//!     Ok(())
//! }
//! ```
//!
//! - Constructs a searcher once and searches for the same needle in multiple streams.
//!
//! ```no_run
//! use std::fs::File;
//! use std::io;
//! use xfind::StreamFinder;
//!
//! fn main() -> io::Result<()> {
//!     let mut f1 = File::open("foo.txt")?;
//!     let mut f2 = File::open("bar.txt")?;
//!
//!     let mut finder = StreamFinder::new(b"baz");
//!     let found_in_f1 = finder.find(&mut f1).is_some();
//!     let found_in_f2 = finder.find(&mut f2).is_some();
//!
//!     Ok(())
//! }
//!
//! ```
//!
//! - Reads the last line of a file, without loading the entire contents of the file into memory.
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::{self, Read};
//! use std::path::Path;
//!
//! fn main() -> io::Result<()> {
//!     let path = "foo.txt";
//!
//!     let mut buf = Vec::new();
//!     read_last_line(path, &mut buf)?;
//!     // For simplicity, we just assume the contents is valid UTF-8 and unwrap here.
//!     println!("{}", std::str::from_utf8(&buf).unwrap());
//!
//!     Ok(())
//! }
//!
//! fn read_last_line<P: AsRef<Path>>(
//!     path: P,
//!     buf: &mut Vec<u8>,
//! ) -> io::Result<usize> {
//!     let mut f = File::open(path)?;
//!     let mut iter = xfind::rfind_iter(b"\n", &mut f)?;
//!
//!     let read_pos = match iter.next().transpose()? {
//!         // if the file contains no newline, we read from the start.
//!         None => 0,
//!         // if the file ends with a newline, we need to perform another search.
//!         Some(pos) if pos + 1 == iter.stream_len() => {
//!             (iter.next().transpose()?.map(|x| x + 1)).unwrap_or(0)
//!         }
//!         // if the file doesn't end with a newline, then `pos + 1` is the `read_pos`.
//!         Some(pos) => pos + 1,
//!     };
//!
//!     iter.seek_to(read_pos)?;
//!     f.read_to_end(buf)
//! }
//! ```
#![deny(missing_docs)]

mod buffer;
mod finder;

pub use finder::*;
