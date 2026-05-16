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
}

impl Default for Options {
    fn default() -> Self {
        Self {
            preserve_read_names: true,
            encode_alignment_start_positions_as_deltas: true,
            version: Version::default(),
            block_content_encoder_map: BlockContentEncoderMap::default(),
            embed_reference: false,
        }
    }
}
