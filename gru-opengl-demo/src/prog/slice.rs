use std::io::{Read, Seek, SeekFrom, Result, Error, ErrorKind};

pub struct SliceReadSeek<'a>(&'a [u8], i64);

impl<'a> SliceReadSeek<'a>
{
    pub fn new(slice: &'a [u8]) -> Self
    {
        Self(slice, 0)
    }
}

impl Read for SliceReadSeek<'_>
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>
    {
        let start = self.1 as usize;
        let len = buf.len().min(self.0.len() - start);
        if len <= 0 { Ok(0) }
        else
        {
            buf[..len].copy_from_slice(&self.0[start .. (start + len)]);
            self.1 += len as i64;
            Ok(len)
        }
    }
}

impl Seek for SliceReadSeek<'_>
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>
    {
        let index = match pos
        {
            SeekFrom::Start(index) => index as i64,
            SeekFrom::End(index) => index + self.0.len() as i64,
            SeekFrom::Current(index) => index + self.1
        };
        if index < 0 { return Result::Err(Error::new(ErrorKind::InvalidInput, "out of bounds")); }
        self.1 = index;
        Ok(index as u64)
    }
}
