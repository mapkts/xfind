//! Provides forward and backward substring searchers that operate on stream.
use crate::buffer::{Buffer, BufferRev};
use memchr::memmem;
use std::io::{self, Read, Seek, SeekFrom};

/// Returns the index of the first occurrence of the given needle in the stream.
///
/// # Examples
///
/// ```
/// use std::io::{self, Cursor};
///
/// fn main() -> io::Result<()> {
///     let mut stream = Cursor::new(b"rusty rust");
///
///     let pos = xfind::find(b"rust", &mut stream).transpose()?;
///     assert_eq!(pos, Some(0));
///
///     Ok(())
/// }
/// ```
pub fn find<R>(needle: &[u8], rdr: &mut R) -> Option<io::Result<usize>>
where
    R: Read,
{
    FindIter::new_with_needle(rdr, needle).next()
}

/// Returns the index of the last occurrence of the given needle in the stream.
///
/// # Examples
///
/// ```
/// use std::io::{self, Cursor};
///
/// fn main() -> io::Result<()> {
///     let mut stream = Cursor::new(b"rusty rust");
///
///     let pos = xfind::rfind(b"rust", &mut stream).transpose()?;
///     assert_eq!(pos, Some(6));
///
///     Ok(())
/// }
/// ```
pub fn rfind<R>(needle: &[u8], rdr: &mut R) -> Option<io::Result<usize>>
where
    R: Read + Seek,
{
    match FindRevIter::new_with_needle(rdr, needle) {
        Ok(mut iter) => iter.next(),
        Err(e) => Some(Err(e)),
    }
}

/// Returns an iterator over all occurrences of the given needle in the stream.
///
/// # Examples
///
/// ```
/// use std::io::{self, Cursor};
///
/// fn main() -> io::Result<()> {
///     let mut stream = Cursor::new(b"rusty rust");
///
///     let mut iter = xfind::find_iter(b"rust", &mut stream);
///     assert_eq!(iter.next().transpose()?, Some(0));
///     assert_eq!(iter.next().transpose()?, Some(6));
///     assert_eq!(iter.next().transpose()?, None);
///
///     Ok(())
/// }
/// ```
pub fn find_iter<'n, 's, R>(
    needle: &'n [u8],
    rdr: &'s mut R,
) -> FindIter<'n, 's, R>
where
    R: Read,
{
    FindIter::new_with_needle(rdr, needle)
}

/// Returns a reverse iterator over all occurrences of the given needle in the stream.
///
/// # Errors
///
/// Returns an I/O error if seeking to the end of the stream failed.
///
/// # Panics
///
/// Panics if the length of the stream is greater than `usize::MAX`.
///
/// # Examples
///
/// ```
/// use std::io::{self, Cursor};
///
/// fn main() -> io::Result<()> {
///     let mut stream = Cursor::new(b"rusty rust");
///
///     let mut iter = xfind::rfind_iter(b"rust", &mut stream)?;
///     assert_eq!(iter.next().transpose()?, Some(6));
///     assert_eq!(iter.next().transpose()?, Some(0));
///     assert_eq!(iter.next().transpose()?, None);
///
///     Ok(())
/// }
/// ```
pub fn rfind_iter<'n, 's, R>(
    needle: &'n [u8],
    rdr: &'s mut R,
) -> io::Result<FindRevIter<'n, 's, R>>
where
    R: Read + Seek,
{
    FindRevIter::new_with_needle(rdr, needle)
}

/// A substring searcher for stream searches.
#[derive(Clone, Debug)]
pub struct StreamFinder<'n> {
    /// The string we want to search.
    needle: &'n [u8],
}

impl<'n> StreamFinder<'n> {
    /// Creates a new `StreamFinder` for the given needle.
    ///
    /// # Examples
    ///
    /// ```
    /// use xfind::StreamFinder;
    ///
    /// let finder = StreamFinder::new(b"rust");
    /// ```
    pub fn new(needle: &'n [u8]) -> StreamFinder<'n> {
        StreamFinder { needle }
    }

    /// Returns the needle that this finder searches for.
    ///
    /// # Examples
    ///
    /// ```
    /// use xfind::StreamFinder;
    ///
    /// let finder = StreamFinder::new(b"rust");
    /// assert_eq!(finder.needle(), b"rust");
    /// ```
    pub fn needle(&self) -> &[u8] {
        self.needle
    }

    /// Returns the index of the first occurrence of the given needle in the stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{self, Cursor};
    /// use xfind::StreamFinder;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stream = Cursor::new(b"rusty rust");
    ///     let finder = StreamFinder::new(b"rust");
    ///
    ///     let pos = finder.find(&mut stream).transpose()?;
    ///     assert_eq!(pos, Some(0));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn find<R: Read>(&self, rdr: &mut R) -> Option<io::Result<usize>> {
        self.find_iter(rdr).next()
    }

    /// Returns the index of the last occurrence of the given needle in the stream.
    ///
    /// # Panics
    ///
    /// Panics if the length of the stream is greater than `usize::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{self, Cursor};
    /// use xfind::StreamFinder;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stream = Cursor::new(b"rusty rust");
    ///     let finder = StreamFinder::new(b"rust");
    ///
    ///     let pos = finder.rfind(&mut stream).transpose()?;
    ///     assert_eq!(pos, Some(6));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn rfind<R: Read + Seek>(
        &self,
        rdr: &mut R,
    ) -> Option<io::Result<usize>> {
        match self.rfind_iter(rdr) {
            Ok(mut iter) => iter.next(),
            Err(e) => Some(Err(e)),
        }
    }

    /// Returns an iterator over all occurrences of the given needle in the stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{self, Cursor};
    /// use xfind::StreamFinder;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stream = Cursor::new(b"rusty rust");
    ///     let finder = StreamFinder::new(b"rust");
    ///
    ///     let mut iter = finder.find_iter(&mut stream);
    ///     assert_eq!(iter.next().transpose()?, Some(0));
    ///     assert_eq!(iter.next().transpose()?, Some(6));
    ///     assert_eq!(iter.next().transpose()?, None);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn find_iter<'s, R: Read>(
        &'n self,
        rdr: &'s mut R,
    ) -> FindIter<'n, 's, R> {
        FindIter::new(rdr, self)
    }

    /// Returns a reverse iterator over all occurrences of the given needle in the stream.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if seeking to the end of the stream failed.
    ///
    /// # Panics
    ///
    /// Panics if the length of the stream is greater than `usize::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{self, Cursor};
    /// use xfind::StreamFinder;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stream = Cursor::new(b"rusty rust");
    ///     let finder = StreamFinder::new(b"rust");
    ///
    ///     let mut iter = finder.rfind_iter(&mut stream)?;
    ///     assert_eq!(iter.next().transpose()?, Some(6));
    ///     assert_eq!(iter.next().transpose()?, Some(0));
    ///     assert_eq!(iter.next().transpose()?, None);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn rfind_iter<'s, R: Read + Seek>(
        &'n self,
        rdr: &'s mut R,
    ) -> io::Result<FindRevIter<'n, 's, R>> {
        FindRevIter::new(rdr, self)
    }
}

/// A forward iterator over all non-overlapping occurrences of a substring in a stream.
///
/// Matches are reported by the byte offset at which they begin.
#[derive(Debug)]
pub struct FindIter<'n, 's, R: Read> {
    /// The stream source we read from.
    rdr: &'s mut R,
    /// The needle we search for.
    needle: &'n [u8],
    /// A fixed size buffer that we actually search for. It must be big enough to hold the needle.
    buf: Buffer,
    /// The current position at which to start the next search in `self.buf`.
    search_pos: usize,
    /// The absolute position of `search_pos` in the stream.
    stream_pos: usize,
    /// The position we report to the caller.
    report_pos: usize,
    /// If the match found was at the very end of the buffer.
    is_tail_match: bool,
}

/// A backward iterator over all non-overlapping occurrences of a substring in a stream.
///
/// Matches are reported by the byte offset at which they begin.
#[derive(Debug)]
pub struct FindRevIter<'n, 's, R: Read + Seek> {
    /// The stream source we read from.
    rdr: &'s mut R,
    /// The needle we search for.
    needle: &'n [u8],
    /// A fixed size buffer that we actually search for. It must be big enough to hold the needle.
    buf: BufferRev,
    /// The current position at which to start the next search in `self.buf`.
    search_pos: usize,
    /// The absolute position of `search_pos` in the stream.
    stream_pos: usize,
    /// The position we report to the caller.
    report_pos: usize,
    /// The current seek position.
    seek_pos: usize,
    /// The length of the stream.
    stream_len: usize,
}

impl<'n, 's, R: Read> FindIter<'n, 's, R> {
    pub(crate) fn new(rdr: &'s mut R, fdr: &'n StreamFinder<'n>) -> Self {
        let needle = fdr.needle();
        let buf = Buffer::new(needle.len());
        FindIter {
            rdr,
            needle,
            buf,
            search_pos: 0,
            stream_pos: 0,
            report_pos: 0,
            is_tail_match: false,
        }
    }

    pub(crate) fn new_with_needle(rdr: &'s mut R, needle: &'n [u8]) -> Self {
        let buf = Buffer::new(needle.len());
        FindIter {
            rdr,
            needle,
            buf,
            search_pos: 0,
            stream_pos: 0,
            report_pos: 0,
            is_tail_match: false,
        }
    }
}

impl<'n, 's, R: Read + Seek> FindRevIter<'n, 's, R> {
    pub(crate) fn new(
        rdr: &'s mut R,
        fdr: &'n StreamFinder<'n>,
    ) -> io::Result<Self> {
        let stream_len = rdr.seek(SeekFrom::End(0))?;
        assert!(stream_len <= usize::MAX as u64);
        let stream_len = stream_len as usize;

        let needle = fdr.needle();
        let buf = BufferRev::new(needle.len());
        Ok(FindRevIter {
            rdr,
            needle,
            buf,
            search_pos: 0,
            stream_pos: stream_len,
            report_pos: 0,
            seek_pos: stream_len,
            stream_len,
        })
    }

    pub(crate) fn new_with_needle(
        rdr: &'s mut R,
        needle: &'n [u8],
    ) -> io::Result<Self> {
        let stream_len = rdr.seek(SeekFrom::End(0))?;
        assert!(stream_len <= usize::MAX as u64);
        let stream_len = stream_len as usize;

        let buf = BufferRev::new(needle.len());
        Ok(FindRevIter {
            rdr,
            needle,
            buf,
            search_pos: 0,
            stream_pos: stream_len,
            report_pos: 0,
            seek_pos: stream_len,
            stream_len,
        })
    }

    /// Returns the length of the underlying stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{self, Read, Cursor};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stream = Cursor::new(b"hello rustaceans");
    ///     let mut iter = xfind::rfind_iter(b"rust", &mut stream)?;
    ///     assert_eq!(iter.stream_len(), 16);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn stream_len(&self) -> usize {
        self.stream_len
    }

    /// Moves the cursor of the underlying stream to the given position.
    ///
    /// This is equivalent to call `rdr.seek(SeekFrom::Start(pos))`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{self, Read, Cursor};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stream = Cursor::new(b"hello rustaceans");
    ///     let mut iter = xfind::rfind_iter(b"rust", &mut stream)?;
    ///     let mut buf = Vec::new();
    ///
    ///     let pos = iter.next().unwrap()?;
    ///     iter.seek_to(pos)?;
    ///     stream.read_to_end(&mut buf)?;
    ///     assert_eq!(buf, b"rustaceans");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn seek_to(&mut self, pos: usize) -> io::Result<()> {
        self.rdr.seek(SeekFrom::Start(pos as u64)).map(|_| ())
    }
}

impl<'n, 's, R: Read> Iterator for FindIter<'n, 's, R> {
    type Item = io::Result<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.search_pos < self.buf.len() {
                if let Some(mat) = memmem::find(
                    &self.buf.buffer()[self.search_pos..],
                    self.needle,
                ) {
                    self.report_pos = self.stream_pos + mat;
                    self.stream_pos += mat + self.needle.len();
                    self.search_pos += mat + self.needle.len();
                    return Some(Ok(self.report_pos));
                }

                self.stream_pos += self.buf.len() - self.search_pos;
                self.search_pos = self.buf.len();
            }

            // Roll our buffer if our buffer has at least the minimum amount of bytes in it.
            if self.buf.len() >= self.buf.min_buffer_len() {
                self.buf.roll();
                if &self.buf.buffer()[..self.buf.min_buffer_len()]
                    == self.needle
                {
                    self.search_pos = self.buf.min_buffer_len();
                } else {
                    self.stream_pos -= self.buf.min_buffer_len();
                    self.search_pos = 0;
                }
            }
            match self.buf.fill(&mut self.rdr) {
                // report any I/O errors.
                Err(err) => return Some(Err(err)),
                // we've reach EOF, return `None` now.
                Ok(false) => {
                    return None;
                }
                // fallthrough for another search.
                Ok(true) => {}
            }
        }
    }
}

impl<'n, 's, R: Read + Seek> Iterator for FindRevIter<'n, 's, R> {
    type Item = io::Result<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If the contents of the buffer have not been consumed yet.
            if self.search_pos < self.buf.len() {
                if let Some(mat) = memmem::rfind(
                    &self.buf.buffer()[..self.buf.len() - self.search_pos],
                    self.needle,
                ) {
                    self.report_pos = self.stream_pos
                        - (self.buf.len() - self.search_pos - mat);

                    // if [19827, 19716, 5838, 938, 544, 51]
                    if [7552, 7450, 6985, 6866, 6829, 6775]
                        .contains(&self.report_pos)
                    {
                        eprintln!(
                            "report: {}, search: {}, stream: {}, seek: {}",
                            self.report_pos,
                            self.search_pos,
                            self.stream_pos,
                            self.seek_pos,
                        );
                        // eprintln!(
                        //     "buffer: {}",
                        //     std::str::from_utf8(self.buf.buffer()).unwrap()
                        // );
                    }

                    self.stream_pos -= self.buf.len() - self.search_pos - mat;
                    self.search_pos += self.buf.len() - self.search_pos - mat;

                    // if [19827, 19716, 5838, 938, 544, 51]
                    if [7552, 7450, 6985, 6866, 6829, 6775]
                        .contains(&self.report_pos)
                    {
                        eprintln!(
                            "report: {}, search: {}, stream: {}, seek: {}, buflen: {}, buflen - search_pos: {}",
                            self.report_pos,
                            self.search_pos,
                            self.stream_pos,
                            self.seek_pos,
                            self.buf.len(),
                            self.buf.len() - self.search_pos,
                        );
                        // eprintln!(
                        //     "buffer: {}",
                        //     std::str::from_utf8(self.buf.buffer()).unwrap()
                        // );
                    }

                    // FIXME: This is a quick and dirty hack to fix end-of-stream roll issues. We
                    // should probably figure out a better way to handle this.
                    if self.stream_len > self.buf.capacity()
                        && self.seek_pos == 0
                    {
                        return Some(Ok(self.report_pos + self.needle.len()));
                    }

                    return Some(Ok(self.report_pos));
                }

                self.stream_pos = self
                    .stream_pos
                    .saturating_sub(self.buf.len() - self.search_pos);
                self.search_pos = self.buf.len();
            }

            // We have nothing left to search if seek position is 0.
            if self.seek_pos == 0 {
                return None;
            }

            // Roll our buffer if our buffer has at least the minimum amount of bytes in it.
            if self.buf.len() >= self.buf.min_buffer_len() {
                self.buf.roll_right();

                if &self.buf.buffer()
                    [self.buf.len() - self.buf.min_buffer_len()..]
                    == self.needle
                {
                    self.search_pos = self.buf.min_buffer_len();
                } else {
                    self.stream_pos += self.buf.min_buffer_len();
                    self.search_pos = 0;
                }
            }

            let free_buffer_len = self.buf.free_buffer().len();
            let amount = if self.stream_pos > free_buffer_len {
                self.seek_pos -= free_buffer_len;
                free_buffer_len
            } else {
                self.seek_pos = 0;
                self.stream_pos
            };
            match self.rdr.seek(SeekFrom::Start(self.seek_pos as u64)) {
                Ok(_) => {}
                Err(e) => return Some(Err(e)),
            }
            match self.buf.fill_exact(&mut self.rdr, amount) {
                // report any I/O errors.
                Err(err) => return Some(Err(err)),
                // we've reach EOF, return `None` now.
                Ok(false) => {
                    return None;
                }
                // fallthrough for another search.
                Ok(true) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::DEFAULT_BUFFER_CAPACITY;
    use std::io::Cursor;
    use std::iter::repeat;

    #[test]
    fn test_find_iter_n1s1() {
        let haystack = b"1";
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"1");
        let matches: Vec<usize> =
            finder.find_iter(&mut haystack).map(|x| x.unwrap()).collect();
        let expected: Vec<usize> = vec![0];
        assert_eq!(matches, expected);

        let finder = StreamFinder::new(b"0");
        let matches: Vec<usize> =
            finder.find_iter(&mut haystack).map(|x| x.unwrap()).collect();
        let expected: Vec<usize> = vec![];
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_rev_iter_n1s1() {
        let haystack = b"1";
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"1");
        let matches: Vec<usize> = finder
            .rfind_iter(&mut haystack)
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        let expected: Vec<usize> = vec![0];
        assert_eq!(matches, expected);

        let finder = StreamFinder::new(b"0");
        let matches: Vec<usize> = finder
            .rfind_iter(&mut haystack)
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        let expected: Vec<usize> = vec![];
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_iter_n2s1() {
        let haystack = b"1";
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"11");
        let matches: Vec<usize> =
            finder.find_iter(&mut haystack).map(|x| x.unwrap()).collect();
        let expected: Vec<usize> = vec![];
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_rev_iter_n2s1() {
        let haystack = b"1";
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"11");
        let matches: Vec<usize> = finder
            .rfind_iter(&mut haystack)
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        let expected: Vec<usize> = vec![];
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_iter_n1s21() {
        let haystack = b"11 - 1 - 11  - 1 - 11";
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"1");
        let matches: Vec<usize> =
            finder.find_iter(&mut haystack).map(|x| x.unwrap()).collect();
        let expected: Vec<usize> = vec![0, 1, 5, 9, 10, 15, 19, 20];
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_rev_iter_n1s21() {
        let haystack = b"11 - 1 - 11  - 1 - 11";
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"1");
        let matches: Vec<usize> = finder
            .rfind_iter(&mut haystack)
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        let expected: Vec<usize> =
            vec![0, 1, 5, 9, 10, 15, 19, 20].into_iter().rev().collect();
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_iter_n1s8213() {
        let haystack: Vec<u8> = repeat(&0u8)
            .take(DEFAULT_BUFFER_CAPACITY)
            .chain("42 0 42 42 0 42".as_bytes())
            .copied()
            .collect();
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"4");
        let matches: Vec<usize> =
            finder.find_iter(&mut haystack).map(|x| x.unwrap()).collect();
        let expected: Vec<usize> = vec![0, 5, 8, 13]
            .into_iter()
            .map(|x| x + DEFAULT_BUFFER_CAPACITY)
            .collect();
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_rev_iter_n1s8213() {
        let haystack: Vec<u8> = repeat(&0u8)
            .take(DEFAULT_BUFFER_CAPACITY)
            .chain("42 0 42 42 0 42".as_bytes())
            .copied()
            .collect();
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"4");
        let matches: Vec<usize> = finder
            .rfind_iter(&mut haystack)
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        let expected: Vec<usize> = vec![0, 5, 8, 13]
            .into_iter()
            .map(|x| x + DEFAULT_BUFFER_CAPACITY)
            .rev()
            .collect();
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_iter_n2s8213() {
        let haystack: Vec<u8> = repeat(&0u8)
            .take(DEFAULT_BUFFER_CAPACITY)
            .chain("42 0 42 42 0 42".as_bytes())
            .copied()
            .collect();
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"42");
        let matches: Vec<usize> =
            finder.find_iter(&mut haystack).map(|x| x.unwrap()).collect();
        let expected: Vec<usize> = vec![0, 5, 8, 13]
            .into_iter()
            .map(|x| x + DEFAULT_BUFFER_CAPACITY)
            .collect();
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_rev_iter_n2s8213() {
        let haystack: Vec<u8> = repeat(&0u8)
            .take(DEFAULT_BUFFER_CAPACITY)
            .chain("42 0 42 42 0 42".as_bytes())
            .copied()
            .collect();
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"42");
        let matches: Vec<usize> = finder
            .rfind_iter(&mut haystack)
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        let expected: Vec<usize> = vec![0, 5, 8, 13]
            .into_iter()
            .map(|x| x + DEFAULT_BUFFER_CAPACITY)
            .rev()
            .collect();
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_iter_n2s8212() {
        let haystack: Vec<u8> = repeat(&0u8)
            .take(DEFAULT_BUFFER_CAPACITY - 1)
            .chain("42 0 42 42 0 42".as_bytes())
            .copied()
            .collect();
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"42");
        let matches: Vec<usize> =
            finder.find_iter(&mut haystack).map(|x| x.unwrap()).collect();
        let expected: Vec<usize> = vec![0, 5, 8, 13]
            .into_iter()
            .map(|x| x + DEFAULT_BUFFER_CAPACITY - 1)
            .collect();
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_rev_iter_n2s8212() {
        let haystack: Vec<u8> = repeat(&0u8)
            .take(DEFAULT_BUFFER_CAPACITY - 1)
            .chain("42 0 42 42 0 42".as_bytes())
            .copied()
            .collect();
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"42");
        let matches: Vec<usize> = finder
            .rfind_iter(&mut haystack)
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        let expected: Vec<usize> = vec![0, 5, 8, 13]
            .into_iter()
            .map(|x| x + DEFAULT_BUFFER_CAPACITY - 1)
            .rev()
            .collect();
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_iter_n3s8212() {
        let haystack: Vec<u8> = repeat(&0u8)
            .take(DEFAULT_BUFFER_CAPACITY - 1)
            .chain("42 0 42 42 0 42".as_bytes())
            .copied()
            .collect();
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"42 ");
        let matches: Vec<usize> =
            finder.find_iter(&mut haystack).map(|x| x.unwrap()).collect();
        let expected: Vec<usize> = vec![0, 5, 8]
            .into_iter()
            .map(|x| x + DEFAULT_BUFFER_CAPACITY - 1)
            .collect();
        assert_eq!(matches, expected);
    }

    #[test]
    fn test_find_rev_iter_n3s8212() {
        let haystack: Vec<u8> = repeat(&0u8)
            .take(DEFAULT_BUFFER_CAPACITY - 1)
            .chain("42 0 42 42 0 42".as_bytes())
            .copied()
            .collect();
        let mut haystack = Cursor::new(haystack);

        let finder = StreamFinder::new(b"42 ");
        let matches: Vec<usize> = finder
            .rfind_iter(&mut haystack)
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        let expected: Vec<usize> = vec![0, 5, 8]
            .into_iter()
            .map(|x| x + DEFAULT_BUFFER_CAPACITY - 1)
            .rev()
            .collect();
        assert_eq!(matches, expected);
    }
}
