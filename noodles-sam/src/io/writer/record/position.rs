use std::io::{self, Write};

use noodles_core::Position;

use crate::io::writer::num;

pub(super) fn write_position<W>(writer: &mut W, position: Option<Position>) -> io::Result<()>
where
    W: Write,
{
    let n = position.map(usize::from).unwrap_or_default();
    num::write_usize(writer, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_position() -> io::Result<()> {
        fn t(buf: &mut Vec<u8>, position: Option<Position>, expected: &[u8]) -> io::Result<()> {
            buf.clear();
            write_position(buf, position)?;
            assert_eq!(buf, expected);

            Ok(())
        }

        let mut buf = Vec::new();

        t(&mut buf, None, b"0")?;
        t(&mut buf, Position::new(13), b"13")?;

        t(&mut buf, Position::new(1 << 33), b"8589934592")?;

        Ok(())
    }
}
