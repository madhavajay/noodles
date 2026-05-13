//! CRAM container and fields.

pub(crate) mod block;
pub mod block_content_encoder_map;
pub mod compression_header;
mod header;
/// Reference sequence context for CRAM containers and slices.
pub mod reference_sequence_context;
pub(crate) mod slice;

pub(crate) use self::header::Header;
pub use self::{
    block_content_encoder_map::BlockContentEncoderMap, compression_header::CompressionHeader,
    reference_sequence_context::ReferenceSequenceContext,
};
