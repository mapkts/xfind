//! Provides forward and backward substring searchers for stream searches.
//!
//! This crate is built on top of [`memchr`], a heavily optimized routines for string searches.
//! But unlike [`memchr`], utilties provided by this crate operate directly on stream (i.e.,
//! [`Read`] instances) rather than in-memory buffers.
//!
//! Note that this crate provides no advantage when searching substring in a source that is already
//! in memory, in this case considering [`memchr`] instead. Besides, if you want to search multiple
//! substrings at once, take a look at [`aho-corasick`].
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
//!     // For simplicity, we just assume the contents is UTF-8 valid and unwrap here.
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
