use std::io::{self};
use std::mem;

type LengthType = u16;

const LENGTH_SIZE: usize = mem::size_of::<LengthType>();

pub fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
  let len = bytes.len();
  let mut msg = vec![0u8; len + LENGTH_SIZE];
  let lb = &(len as LengthType).to_be_bytes();
  msg[0..LENGTH_SIZE].clone_from_slice(lb);
  msg[LENGTH_SIZE..].clone_from_slice(bytes);
  msg
}

pub fn decode_bytes<T: Iterator<Item = Result<u8, io::Error>>>(
  iter: &mut T,
) -> io::Result<Vec<u8>> {
  let start_byte = iter.next();
  start_byte.map_or_else(
    || {
      Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "bytes stream is empty",
      ))
    },
    |result| {
      result.map_or_else(
        |e| {
          eprintln!("read first byte failed: {e}");
          Err(e)
        },
        |first_byte| {
          let length_vec = iter
            .take(LENGTH_SIZE - 1)
            .collect::<io::Result<Vec<u8>>>()?;

          let mut buf = [0u8; LENGTH_SIZE];
          buf[0] = first_byte;
          buf[1..LENGTH_SIZE].clone_from_slice(&length_vec);
          let length = LengthType::from_be_bytes(buf);
          iter.take(length as usize).collect()
        },
      )
    },
  )
}
