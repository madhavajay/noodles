use super::{DEFAULT_RECORDS_PER_SLICE, DEFAULT_SLICES_PER_CONTAINER};
use crate::{container::BlockContentEncoderMap, file_definition::Version};

#[derive(Clone, Debug)]
pub struct Options {
    pub preserve_read_names: bool,
    pub encode_alignment_start_positions_as_deltas: bool,
    pub version: Version,
    pub block_content_encoder_map: BlockContentEncoderMap,
    /// Embed each mapped slice's reference span as an in-container
    /// block (`samtools view -O cram,embed_ref=1`), so the file
    /// decodes with no external reference.
    pub embed_reference: bool,
    /// Maximum number of records per slice
    /// (`samtools view -O cram,seqs_per_slice=N`). A new slice is cut
    /// once this many records have accumulated.
    pub records_per_slice: usize,
    /// Maximum number of slices per container
    /// (`samtools view -O cram,slices_per_slice=N`). A new container is
    /// flushed once `records_per_slice * slices_per_container` records
    /// have accumulated.
    pub slices_per_container: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            preserve_read_names: true,
            encode_alignment_start_positions_as_deltas: true,
            version: Version::default(),
            block_content_encoder_map: BlockContentEncoderMap::default(),
            embed_reference: false,
            records_per_slice: DEFAULT_RECORDS_PER_SLICE,
            slices_per_container: DEFAULT_SLICES_PER_CONTAINER,
        }
    }
}
