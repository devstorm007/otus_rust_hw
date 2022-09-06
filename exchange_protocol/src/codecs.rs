use std::io::{self};
use std::mem;

use tokio::net::tcp::OwnedReadHalf;

pub const MAX_SIZE: usize = LengthType::MAX as usize;

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

pub async fn decode_bytes_async(reader: &mut OwnedReadHalf) -> io::Result<Vec<u8>> {
    let mut buf = [0u8; LENGTH_SIZE];
    read_exact_async(reader, &mut buf).await?;
    let length = LengthType::from_be_bytes(buf) as usize;

    let mut data = vec![0u8; length];
    read_exact_async(reader, &mut data).await?;

    Ok(data.to_vec())

    /*let start_byte = iter.next();
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
    )*/
}

async fn read_exact_async(reader: &mut OwnedReadHalf, buf: &mut [u8]) -> io::Result<()> {
    let mut read = 0;
    while read < buf.len() {
        reader.readable().await?;
        match reader.try_read(&mut buf[read..]) {
            Ok(0) if read == 0 => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "bytes reader is empty",
                ))
            }
            Ok(0) => break,
            Ok(n) => read += n,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/*async fn send_string_async<Data: AsRef<str>>(data: Data, writer: &mut OwnedWriteHalf) -> io::Result<()> {
    let bytes = data.as_ref().as_bytes();
    let len = bytes.len() as u32;
    let len_bytes = len.to_be_bytes();
    write_all_async(stream, &len_bytes).await?;
    write_all_async(stream, bytes).await?;
    Ok(())
}*/
