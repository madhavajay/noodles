use std::{
    io::{self, Write},
    num::NonZero,
};

use crate::header::record::value::map::reference_sequence::tag;

pub(super) fn write_length_field<W>(writer: &mut W, length: NonZero<usize>) -> io::Result<()>
where
    W: Write,
{
    use crate::io::writer::{
        header::record::{value::map::write_separator, write_delimiter},
        num,
    };

    write_delimiter(writer)?;
    writer.write_all(tag::LENGTH.as_ref())?;
    write_separator(writer)?;

    num::write_usize(writer, usize::from(length))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_length_field() -> io::Result<()> {
        let mut buf = Vec::new();
        write_length_field(&mut buf, const { NonZero::new(8).unwrap() })?;
        assert_eq!(buf, b"\tLN:8");
        Ok(())
    }
}
