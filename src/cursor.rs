use std::simd::{Simd, cmp::SimdPartialEq, u8x16};

pub struct Cursor<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for Cursor<'a> {
    fn from(buffer: &'a [u8]) -> Self {
        Self(buffer)
    }
}

impl<'a> Cursor<'a> {
    const SPLAT: u8x16 = Simd::splat(b'\n');
    const COUNT: usize = 16;

    fn find_new_line(mut buffer: &[u8]) -> usize {
        let mut ptr = 0;
        while let Some((chunk, rest)) = buffer.split_first_chunk() {
            let bytes = Simd::from_array(*chunk);
            let index = bytes.simd_eq(Cursor::SPLAT).first_set().map(|i| i + ptr);
            if let Some(index) = index {
                return index;
            }
            ptr += Cursor::COUNT;
            buffer = rest;
        }

        let bytes = Simd::load_or_default(buffer);
        let index = bytes.simd_eq(Cursor::SPLAT).first_set().map(|i| i + ptr);
        unsafe { index.unwrap_unchecked() }
    }

    fn find_semicolon(buffer: &[u8]) -> usize {
        let mut pos = buffer.len() - 4;
        while unsafe { *buffer.get_unchecked(pos) } != b';' {
            pos -= 1;
        }
        pos
    }
}

impl<'a> Cursor<'a> {
    pub fn chunks(self, count: usize) -> impl Iterator<Item = &'a [u8]> {
        ChunkIterator {
            buffer: self.0,
            len: self.0.len(),
            ptr: 0,
            size: self.0.len() / count,
        }
    }
}

struct ChunkIterator<'a> {
    buffer: &'a [u8],
    len: usize,
    ptr: usize,
    size: usize,
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.len {
            return None;
        }

        let mut end = self.ptr + self.size;
        if end < self.len - Cursor::COUNT {
            end += Cursor::find_new_line(&self.buffer[end..]) + 1;
        } else {
            end = self.len;
        }
        let buffer = unsafe { self.buffer.get_unchecked(self.ptr..end) };

        self.ptr = end;
        Some(buffer)
    }
}

impl<'a> Cursor<'a> {
    pub fn lines(self) -> impl Iterator<Item = (&'a [u8], &'a [u8])> {
        LineIterator {
            buffer: self.0,
            len: self.0.len(),
            ptr: 0,
        }
    }
}

struct LineIterator<'a> {
    buffer: &'a [u8],
    len: usize,
    ptr: usize,
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.len {
            return None;
        }

        let idx = self.ptr + Cursor::find_new_line(&self.buffer[self.ptr..]);
        let line = unsafe { self.buffer.get_unchecked(self.ptr..idx) };

        let pos = Cursor::find_semicolon(line);
        let station = unsafe { line.get_unchecked(..pos) };
        let temperature = unsafe { line.get_unchecked(pos + 1..) };

        self.ptr = idx + 1;
        Some((station, temperature))
    }
}
