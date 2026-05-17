use std::{error, fmt};

use bstr::BString;

use crate::{Header, header::get_reference_sequence_index_of};

/// An error when a raw SAM record reference sequence ID fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The reference sequence is not in the reference sequence dictionary.
    MissingReferenceSequenceDictionaryEntry(BString),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingReferenceSequenceDictionaryEntry(name) => {
                write!(f, "missing reference sequence dictionary entry: {name}")
            }
        }
    }
}

pub(super) fn parse_reference_sequence_id(
    header: &Header,
    src: &[u8],
) -> Result<usize, ParseError> {
    get_reference_sequence_index_of(header.reference_sequences(), src)
        .ok_or_else(|| ParseError::MissingReferenceSequenceDictionaryEntry(src.into()))
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use super::*;
    use crate::header::record::value::{
        Map,
        map::{ReferenceSequence, reference_sequence},
    };

    #[test]
    fn test_parse_reference_sequence_id() {
        let header = Header::builder()
            .add_reference_sequence(
                "sq0",
                Map::<ReferenceSequence>::new(const { NonZero::new(8).unwrap() }),
            )
            .add_reference_sequence(
                "sq1",
                Map::<ReferenceSequence>::new(const { NonZero::new(13).unwrap() }),
            )
            .build();

        assert_eq!(parse_reference_sequence_id(&header, b"sq0"), Ok(0));
        assert_eq!(parse_reference_sequence_id(&header, b"sq1"), Ok(1));

        assert_eq!(
            parse_reference_sequence_id(&header, b"*"),
            Err(ParseError::MissingReferenceSequenceDictionaryEntry(
                BString::from("*")
            ))
        );

        assert_eq!(
            parse_reference_sequence_id(&header, b"sq2"),
            Err(ParseError::MissingReferenceSequenceDictionaryEntry(
                BString::from("sq2")
            ))
        );
    }

    #[test]
    fn test_parse_reference_sequence_id_with_alternative_name() {
        let mut sq1 = Map::<ReferenceSequence>::new(const { NonZero::new(13).unwrap() });
        sq1.other_fields_mut().insert(
            reference_sequence::tag::ALTERNATIVE_NAMES,
            BString::from("alt1,alt2"),
        );

        let header = Header::builder()
            .add_reference_sequence(
                "sq0",
                Map::<ReferenceSequence>::new(const { NonZero::new(8).unwrap() }),
            )
            .add_reference_sequence("sq1", sq1)
            .build();

        assert_eq!(parse_reference_sequence_id(&header, b"alt2"), Ok(1));
    }
}
