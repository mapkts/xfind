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
//! [`Pride and Prejudice`](https://www.gutenberg.org/files/1342/1342-0.txt). Feel free to run the
//! benchsuite using `cargo bench` in your terminal.
//!
//! ```text
//! test group_1::stream_find_iter::aho_corasick ... bench:     558,530 ns/iter (+/- 8,705)
//! test group_1::stream_find_iter::memchr       ... bench:      89,728 ns/iter (+/- 4,979)
//! test group_1::stream_find_iter::xfind        ... bench:     112,766 ns/iter (+/- 2,453)
//!
//! test group_2::stream_rfind_iter::memchr      ... bench:     613,183 ns/iter (+/- 10,610)
//! test group_2::stream_rfind_iter::xfind       ... bench:     681,210 ns/iter (+/- 6,990)
//!
//! test group_3::memory_find_iter::aho_corasick ... bench:     454,277 ns/iter (+/- 2,030)
//! test group_3::memory_find_iter::memchr       ... bench:      21,564 ns/iter (+/- 657)
//! test group_3::memory_find_iter::xfind        ... bench:      41,548 ns/iter (+/- 2,028)
//!
//! test group_4::memory_rfind_iter::memchr      ... bench:     543,737 ns/iter (+/- 4,420)
//! test group_4::memory_rfind_iter::xfind       ... bench:     590,744 ns/iter (+/- 14,684)
//! ```
//!
//! - When performing forward stream searches, `xfind` is about 1.3x slower than `memchr::memmem`
//! (group 1), which is actually quite fast because `memmem` itself operates on in-memory buffer
//! but `xfind` operates directly on stream. The great difference is memory usage, `xfind` done its
//! jobs by using a 8KB-only buffer, but `memmem` needed to read the contents of the file into a
//! file-sized buffer (767KB in this case).
//!
//! - `xfind` provides no advantage when searching through in-memory buffers (nearly 2x slower)
//! (group 3), so please don't use it for in-memory searches.
//!
//! - When searching only one substrings, `xfind` beats `aho-corasick` in all cases above
//! (group 1, 3), which is still fair because `aho-corasick` is mainly used for searching multiple
//! substrings at once.
//!
//! - Reverse stream searches are by its nature much slower than forward stream searches
//! (group 2, 4). The performances of `xfind` and `memmem` are pretty close, only memory usages
//! differ.
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
