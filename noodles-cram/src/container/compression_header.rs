//! CRAM container compression header.
//!
//! These types are exposed as a low-level inventory surface (used by
//! `samtools cram-size` via htslib-rs); the individual accessors are
//! self-describing, so `missing_docs` is allowed for this subtree.
#![allow(missing_docs)]

pub mod data_series_encodings;
pub mod encoding;
pub mod preservation_map;
pub mod tag_encodings;

// Public so external consumers (e.g. `samtools cram-size` via
// htslib-rs) can inspect a container's encodings / preservation map.
pub use self::{
    data_series_encodings::DataSeriesEncodings, encoding::Encoding,
    preservation_map::PreservationMap, tag_encodings::TagEncodings,
};

/// A CRAM container compression header.
///
/// The compression header has three maps with information about how the data is compressed: a
/// preservation map, a data series encodings, and a tag encoding map.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CompressionHeader {
    pub(crate) preservation_map: PreservationMap,
    pub(crate) data_series_encodings: DataSeriesEncodings,
    pub(crate) tag_encodings: TagEncodings,
}

impl CompressionHeader {
    pub(crate) fn new(
        preservation_map: PreservationMap,
        data_series_encodings: DataSeriesEncodings,
        tag_encodings: TagEncodings,
    ) -> Self {
        Self {
            preservation_map,
            data_series_encodings,
            tag_encodings,
        }
    }

    /// The container's preservation map.
    pub fn preservation_map(&self) -> &PreservationMap {
        &self.preservation_map
    }

    /// The container's data-series encodings.
    pub fn data_series_encodings(&self) -> &DataSeriesEncodings {
        &self.data_series_encodings
    }

    /// The container's tag encodings.
    pub fn tag_encodings(&self) -> &TagEncodings {
        &self.tag_encodings
    }
}
