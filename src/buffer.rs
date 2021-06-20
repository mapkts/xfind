use std::cmp;
use std::io;
use std::ptr;

/// The default buffer capacity for the stream buffer is 8KB.
pub const DEFAULT_BUFFER_CAPACITY: usize = 8 * (1 << 10);

/// A fairly simple roll buffer for supporting stream searching.
#[derive(Debug)]
pub struct Buffer {
    /// A fixed-size raw buffer.
    buf: Vec<u8>,
    /// The minimum size of the buffer, which is equivalent to the length of the search string.
    min: usize,
    /// The end of the contents of this buffer.
    end: usize,
}

impl Buffer {
    /// Creates a new buffer for stream searching.
    pub fn new(min_buffer_len: usize) -> Buffer {
        let min = cmp::max(1, min_buffer_len);
        // The minimum buffer capacity is at least 1 byte bigger than our search string, but for
        // performance reasons we choose a lower bound of `8 * min`.
        let capacity = cmp::max(min * 8, DEFAULT_BUFFER_CAPACITY);
        Buffer { buf: vec![0; capacity], min, end: 0 }
    }

    /// Returns the minimum size of the buffer.
    #[inline]
    pub fn min_buffer_len(&self) -> usize {
        self.min
    }

    /// Returns the contents of this buffer.
    #[inline]
    pub fn buffer(&self) -> &[u8] {
        &self.buf[..self.end]
    }

    /// Returns the total length of the contents in this buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.end
    }

    /// Returns all free capactiy in this buffer.
    fn free_buffer(&mut self) -> &mut [u8] {
        &mut self.buf[self.end..]
    }

    /// Refill the contents of this buffer by reading as much as possible into this buffer's free
    /// capacity. If no more bytes could be read, then this returns false. Otherwise, this reads
    /// until it has filled the buffer past the minimum amount.
    pub fn fill<R: io::Read>(&mut self, mut rdr: R) -> io::Result<bool> {
        let mut readany = false;
        loop {
            let bytes_read = rdr.read(self.free_buffer())?;
            if bytes_read == 0 {
                return Ok(readany);
            }
            readany = true;
            self.end += bytes_read;
            if self.len() >= self.min {
                return Ok(true);
            }
        }
    }

    /// Rolls the contents of the buffer so that the suffix of this buffer is moved to the front
    /// and all other contents are dropped. The size of the suffix corresponds precisely to the
    /// minimum buffer length.
    ///
    /// This should only be called when the entire contents of this buffer have been searched.
    pub fn roll(&mut self) {
        let roll_start = self
            .end
            .checked_sub(self.min)
            .expect("buffer capacity should be bigger than minimum amount.");
        let roll_len = self.min;

        assert!(roll_start + roll_len <= self.end);
        unsafe {
            // SAFETY: A buffer contains Copy data, so there's no problem moving it around. Safety
            // also depends on our indices being in bounds, which they always should be, given the
            // assert above.
            ptr::copy(
                self.buf[roll_start..].as_ptr(),
                self.buf.as_mut_ptr(),
                roll_len,
            );
        }
        self.end = roll_len;
    }
}

/// A fairly simple roll buffer for supporting stream searching from the end of a stream.
#[derive(Debug)]
pub struct BufferRev {
    /// A fixed-size raw buffer.
    buf: Vec<u8>,
    /// The minimum size of the buffer, which is equivalent to the length of the search string.
    min: usize,
    /// The end of the contents of this buffer.
    end: usize,
}

impl BufferRev {
    /// Creates a new buffer for stream searching.
    pub fn new(min_buffer_len: usize) -> Self {
        let min = cmp::max(1, min_buffer_len);
        // The minimum buffer capacity is at least 1 byte bigger than our search string, but for
        // performance reasons we choose a lower bound of `8 * min`.
        let capacity = cmp::max(min * 8, DEFAULT_BUFFER_CAPACITY);
        BufferRev { buf: vec![0; capacity], min, end: 0 }
    }

    /// Returns the minimum size of the buffer.
    #[inline]
    pub fn min_buffer_len(&self) -> usize {
        self.min
    }

    /// Returns the capacity of the buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Returns the contents of this buffer.
    #[inline]
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.capacity() - self.end..]
    }

    /// Returns the total length of the contents in this buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.end
    }

    /// Returns all free capactiy in this buffer.
    pub fn free_buffer(&mut self) -> &mut [u8] {
        let capacity = self.capacity();
        &mut self.buf[..capacity - self.end]
    }

    /// Fill the contents of this buffer by reading exactly the given amount into this buffer. If
    /// there are no more than the given amount of bytes left to read, then this returns false.
    /// Otherwise, this reads until it has filled the buffer with the given amount of bytes.
    pub fn fill_exact<R: io::Read>(
        &mut self,
        mut rdr: R,
        amount: usize,
    ) -> io::Result<bool> {
        let free_buffer_len = self.free_buffer().len();
        match rdr
            .read_exact(&mut self.free_buffer()[free_buffer_len - amount..])
        {
            Ok(_) => {
                self.end += amount;
                Ok(true)
            }
            Err(e) => match e.kind() {
                io::ErrorKind::UnexpectedEof => Ok(false),
                _ => Err(e),
            },
        }
    }

    /// Rolls the contents of the buffer so that the prefix of this buffer is moved to the end
    /// and all other contents are dropped. The size of the prefix corresponds precisely to the
    /// minimum buffer length.
    ///
    /// This should only be called when the entire contents of this buffer have been searched. And
    /// this should only be called when it cooperates with `fill_exact`.
    pub fn roll_right(&mut self) {
        let roll_start = self
            .end
            .checked_sub(self.min)
            .expect("buffer capacity should be bigger than minimum amount.");
        let roll_len = self.min;

        assert!(roll_start + roll_len <= self.end);
        unsafe {
            // SAFETY: A buffer contains Copy data, so there's no problem moving it around. Safety
            // also depends on our indices being in bounds, which they always should be, given the
            // assert above.
            ptr::copy(
                self.buffer()[..roll_len].as_ptr(),
                self.buf.as_mut_ptr().add(self.capacity() - roll_len),
                roll_len,
            );
        }
        self.end = roll_len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use std::io::{Cursor, SeekFrom};

    #[test]
    fn test_buffer() {
        let mut haystack = Cursor::new("0123456789".as_bytes());
        let mut buf = Buffer::new(2);
        assert_eq!(buf.min_buffer_len(), 2);

        while buf.fill(&mut haystack).unwrap() {}
        assert_eq!(buf.buffer(), b"0123456789");
        assert_eq!(buf.len(), 10);

        buf.roll();
        assert_eq!(buf.buffer(), "89".as_bytes());
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_buffer_rev() {
        let mut haystack = Cursor::new("0123456789".as_bytes());
        let mut buf = BufferRev::new(2);
        assert_eq!(buf.min_buffer_len(), 2);

        haystack.seek(SeekFrom::End(-4)).unwrap();
        buf.fill_exact(&mut haystack, 4).unwrap();
        assert_eq!(buf.buffer(), "6789".as_bytes());
        assert_eq!(buf.len(), 4);

        buf.roll_right();
        assert_eq!(buf.buffer(), "67".as_bytes());
        assert_eq!(buf.len(), 2);

        haystack.seek(SeekFrom::End(-10)).unwrap();
        buf.fill_exact(&mut haystack, 6).unwrap();
        assert_eq!(buf.buffer(), "01234567".as_bytes());
        assert_eq!(buf.len(), 8);
    }
}
