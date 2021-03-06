use std::io::{self, Cursor, Write};

use bytes::{Buf, BufMut};

#[derive(Debug)]
pub(crate) enum Error {
    IncompleteMessage,
    IntegerOverflow,
    Io(io::Error),
}

pub(crate) struct Message<B> {
    buf: B,
}

impl<B> Message<B> {
    pub fn new(buf: B) -> Message<B> {
        Message { buf }
    }

    pub fn get_ref(&self) -> &B {
        &self.buf
    }

    pub fn get_mut(&mut self) -> &mut B {
        &mut self.buf
    }

    pub fn into_inner(self) -> B {
        self.buf
    }
}

impl<B: Buf> Message<B> {
    #[inline]
    pub fn read_uint(&mut self) -> Result<u64, Error> {
        if self.buf.remaining() < 1 {
            return Err(Error::IncompleteMessage);
        }
        let u7_or_len = self.buf.get_u8();
        if u7_or_len < 128 {
            return Ok(u7_or_len as u64);
        }
        let len = !u7_or_len + 1;
        if self.buf.remaining() < len as usize {
            return Err(Error::IncompleteMessage);
        }
        Ok(self.buf.get_uint_be(len as usize))
    }

    #[inline]
    pub fn read_int(&mut self) -> Result<i64, Error> {
        let bits = self.read_uint()?;
        let sign = bits & 1;
        let sint = (bits >> 1) as i64;
        if sign == 0 {
            Ok(sint)
        } else {
            Ok(!sint)
        }
    }

    #[inline]
    pub fn read_float(&mut self) -> Result<f64, Error> {
        let bits = self.read_uint()?;
        Ok(f64::from_bits(bits.swap_bytes()))
    }

    #[inline]
    pub fn read_bool(&mut self) -> Result<bool, Error> {
        match self.read_uint()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::IntegerOverflow),
        }
    }

    #[inline]
    pub fn read_bytes_len(&mut self) -> Result<usize, Error> {
        let len = self.read_uint()?;
        if (self.buf.remaining() as u64) < len {
            return Err(Error::IncompleteMessage);
        }
        Ok(len as usize)
    }
}

impl<B: BufMut> Message<B> {
    #[inline]
    pub fn write_uint(&mut self, n: u64) -> Result<(), Error> {
        if n < 128 {
            self.buf.put_u8(n as u8);
        } else {
            let nbytes = 8 - (n.leading_zeros() / 8) as u8;
            self.buf.put_u8(!(nbytes - 1));
            self.buf.put_uint_be(n, nbytes as usize);
        }
        Ok(())
    }

    #[inline]
    pub fn write_bool(&mut self, b: bool) -> Result<(), Error> {
        match b {
            false => self.write_uint(0),
            true => self.write_uint(1),
        }
    }

    #[inline]
    pub fn write_int(&mut self, n: i64) -> Result<(), Error> {
        let u: u64;
        if n < 0 {
            u = (!(n as u64) << 1) | 1;
        } else {
            u = (n as u64) << 1;
        }
        self.write_uint(u)
    }

    #[inline]
    pub fn write_float(&mut self, n: f64) -> Result<(), Error> {
        let bits = n.to_bits();
        self.write_uint(bits.swap_bytes())
    }

    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.write_uint(bytes.len() as u64)?;
        self.buf.put_slice(bytes);
        Ok(())
    }
}

pub(crate) struct Writer<W> {
    inner: W,
}

impl<W> Writer<W> {
    pub fn new(inner: W) -> Writer<W> {
        Writer { inner }
    }

    pub fn borrow_mut(&mut self) -> Writer<&mut W> {
        Writer::new(self.get_mut())
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Writer<W> {
    fn write_buf(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.inner.write_all(buf).map_err(|err| Error::Io(err))
    }

    pub fn write_section(&mut self, type_id: i64, buf: &[u8]) -> Result<(), Error> {
        let mut type_id_msg = Message::new(Cursor::new([0u8; 9]));
        type_id_msg.write_int(type_id)?;
        let type_id_pos = type_id_msg.get_ref().position() as usize;
        let mut len_msg = Message::new(Cursor::new([0u8; 9]));
        len_msg.write_uint((buf.len() + type_id_pos) as u64)?;
        let len_pos = len_msg.get_ref().position() as usize;
        self.write_buf(&len_msg.get_ref().get_ref()[..len_pos])?;
        self.write_buf(&type_id_msg.get_ref().get_ref()[..type_id_pos])?;
        self.write_buf(buf)
    }
}
