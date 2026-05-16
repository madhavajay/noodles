use indexmap::IndexMap;

use super::{Encoding, encoding::codec::ByteArray};
use crate::container::block;

/// CRAM tag encodings, keyed by tag content id.
///
/// An [`IndexMap`] (rather than a `HashMap`) so the compression
/// header's tag insertion order is preserved — `samtools cram-size
/// -e` emits tag encodings in that order, matching htslib's
/// `cram_codec_iter`.
pub type TagEncodings = IndexMap<block::ContentId, Encoding<ByteArray>>;
