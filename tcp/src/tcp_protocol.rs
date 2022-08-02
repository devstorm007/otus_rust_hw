use std::io::{self /*, Error*/};
use std::mem;
/*use std::io::{self, Bytes};
use std::io::{Read, Write};*/

const LENGTH_SIZE: usize = mem::size_of::<usize>();

pub fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
    let len = bytes.len();
    let mut msg = vec![0u8; len + LENGTH_SIZE];
    let lb = &len.to_be_bytes();
    msg[0..LENGTH_SIZE].clone_from_slice(lb);
    msg[LENGTH_SIZE..].clone_from_slice(bytes);
    //LittleEndian::write_u16(&mut msg[0..LENGTH_SIZE], len as u16);
    msg
}

pub fn decode_bytes<T: Iterator<Item = Result<u8, io::Error>>>(
    iter: &mut T,
) -> io::Result<Vec<u8>> {
    let start_byte = iter.next();
    start_byte.map_or(
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bytes stream is empty",
        )),
        |br| match br {
            Ok(b) => {
                let mut buf = [0u8; LENGTH_SIZE];
                let length_vec = iter
                    .take(LENGTH_SIZE - 1)
                    .collect::<io::Result<Vec<u8>>>()?;
                buf[0] = b;
                buf[1..LENGTH_SIZE].clone_from_slice(&length_vec);
                let length = usize::from_be_bytes(buf);
                //let length = LittleEndian::read_u16(&length_buf) as usize;
                iter /*.skip(LENGTH_SIZE)*/
                    .take(length)
                    .collect()
            }
            Err(e) => Err(e),
        },
    )

    /*let mut buf = [0u8; 2];
    copy_or_result(
        TakeNoMoveIterator {
            remaining: 2,
            iter: &mut iter,
        },
        &mut buf,
        2,
    )?;

    let mut buf = vec![0u8; length];
    copy_or_result(
        TakeNoMoveIterator {
            remaining: length,
            iter: &mut iter,
        },
        &mut buf,
        length,
    )?;*/

    //Ok(buf)
}

/*fn copy_or_result<T: Iterator<Item = Result<u8, io::Error>>>(
    mut iter: T,
    target: &mut [u8],
    amount: usize,
) -> io::Result<()> {
    for i in 0..amount {
        if let Err(x) = match iter.next() {
            Some(num_or_err) => match num_or_err {
                Ok(num) => {
                    target[i] = num;
                    Ok(())
                }
                Err(x) => Err(x),
            },
            None => Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "stopped in the middle of a message",
            )),
        } {
            return Err(x);
        }
    }
    Ok(())
}

struct TakeNoMoveIterator<'a> {
    remaining: usize,
    iter: &'a mut dyn Iterator<Item = Result<u8, io::Error>>,
}

impl Iterator for TakeNoMoveIterator<'_> {
    type Item = Result<u8, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            self.iter.next()
        }
    }
}
//iter: &'a mut dyn Iterator<Item = Result<u8, io::Error>>
fn copy<T: Iterator<Item = Result<u8, io::Error>>>(
    mut iter: T,
    target: &mut [u8],
) -> io::Result<()> {
    //iter.take()
    for i in 0..target.len() {
        if let Err(x) = match iter.next() {
            Some(num_or_err) => match num_or_err {
                Ok(num) => {
                    target[i] = num;
                    Ok(())
                }
                Err(x) => Err(x),
            },
            None => Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "stopped in the middle of a message",
            )),
        } {
            return Err(x);
        }
    }
    Ok(())
}*/
