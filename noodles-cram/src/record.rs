//! CRAM record.

mod cigar;
pub(crate) mod data;
pub(crate) mod feature;
mod flags;
mod mate_flags;
mod quality_scores;
mod sequence;

use std::{borrow::Cow, io};

use bstr::{BStr, ByteSlice};
use noodles_core::Position;
use noodles_sam::{
    self as sam,
    alignment::Record as _,
    alignment::record::{MappingQuality, data::field::Tag},
};

use self::{
    cigar::Cigar,
    data::{Data, field::Value},
    quality_scores::QualityScores,
    sequence::Sequence,
};
pub(crate) use self::{feature::Feature, flags::Flags, mate_flags::MateFlags};
use crate::{
    container::compression_header::preservation_map::SubstitutionMatrix,
    io::reader::container::slice::ReferenceSequence,
};

/// A CRAM record.
#[derive(Clone, Debug, PartialEq)]
pub struct Record<'c> {
    pub(crate) id: u64,
    pub(crate) header: Option<&'c sam::Header>,
    pub(crate) reference_sequence: Option<ReferenceSequence<'c>>,
    pub(crate) substitution_matrix: SubstitutionMatrix,
    pub(crate) bam_flags: sam::alignment::record::Flags,
    pub(crate) cram_flags: Flags,
    pub(crate) reference_sequence_id: Option<usize>,
    pub(crate) read_length: usize,
    pub(crate) alignment_start: Option<Position>,
    pub(crate) read_group_id: Option<usize>,
    pub(crate) name: Option<Cow<'c, [u8]>>,
    pub(crate) mate_flags: MateFlags,
    pub(crate) mate_reference_sequence_id: Option<usize>,
    pub(crate) mate_alignment_start: Option<Position>,
    pub(crate) template_length: i32,
    pub(crate) mate_distance: Option<usize>,
    pub(crate) data: Vec<(Tag, Value<'c>)>,
    pub(crate) sequence: Cow<'c, [u8]>,
    pub(crate) features: Vec<Feature<'c>>,
    pub(crate) mapping_quality: Option<MappingQuality>,
    pub(crate) quality_scores: Cow<'c, [u8]>,
}

impl Record<'_> {
    fn alignment_span(&self) -> usize {
        calculate_alignment_span(self.read_length, &self.features)
    }

    pub(crate) fn alignment_end(&self) -> Option<Position> {
        self.alignment_start.and_then(|alignment_start| {
            let end = usize::from(alignment_start) + self.alignment_span() - 1;
            Position::new(end)
        })
    }

    pub fn base_quality_at_reference_position(
        &self,
        target: Position,
        reference_base: u8,
    ) -> Option<(u8, u8)> {
        if self.bam_flags.is_unmapped() {
            return None;
        }

        let mut reference_position = self.alignment_start?;
        let mut read_position = Position::MIN;
        let quality_scores: Vec<u8> = self.quality_scores().iter().collect::<io::Result<_>>().ok()?;

        for feature in &self.features {
            let feature_position = usize::from(feature.position());
            let match_len = feature_position.checked_sub(usize::from(read_position))?;

            if let Some(end) = reference_position.checked_add(match_len) {
                if target >= reference_position && target < end {
                    let offset = usize::from(target) - usize::from(reference_position);
                    let read_index = usize::from(read_position) - 1 + offset;
                    let quality_score = quality_scores.get(read_index).copied().unwrap_or(0);
                    return Some((reference_base, quality_score));
                }
            }

            reference_position = reference_position.checked_add(match_len)?;
            read_position = read_position.checked_add(match_len)?;

            match feature {
                Feature::Bases { bases, .. } => {
                    let len = bases.len();
                    if let Some(end) = reference_position.checked_add(len) {
                        if target >= reference_position && target < end {
                            let offset = usize::from(target) - usize::from(reference_position);
                            let read_index = usize::from(read_position) - 1 + offset;
                            let quality_score =
                                quality_scores.get(read_index).copied().unwrap_or(0);
                            return bases.get(offset).copied().map(|base| (base, quality_score));
                        }
                    }
                    reference_position = reference_position.checked_add(len)?;
                    read_position = read_position.checked_add(len)?;
                }
                Feature::ReadBase { base, .. } => {
                    if target == reference_position {
                        let read_index = usize::from(read_position) - 1;
                        let quality_score = quality_scores.get(read_index).copied().unwrap_or(0);
                        return Some((*base, quality_score));
                    }
                    reference_position = reference_position.checked_add(1)?;
                    read_position = read_position.checked_add(1)?;
                }
                Feature::Substitution { code, .. } => {
                    if target == reference_position {
                        let reference_base =
                            crate::container::compression_header::preservation_map::substitution_matrix::Base::try_from(reference_base)
                                .unwrap_or(crate::container::compression_header::preservation_map::substitution_matrix::Base::N);
                        let base = u8::from(self.substitution_matrix.get(reference_base, *code));
                        let read_index = usize::from(read_position) - 1;
                        let quality_score = quality_scores.get(read_index).copied().unwrap_or(0);
                        return Some((base, quality_score));
                    }
                    reference_position = reference_position.checked_add(1)?;
                    read_position = read_position.checked_add(1)?;
                }
                Feature::Insertion { bases, .. } => {
                    read_position = read_position.checked_add(bases.len())?;
                }
                Feature::Deletion { len, .. } | Feature::ReferenceSkip { len, .. } => {
                    let end = reference_position.checked_add(*len)?;
                    if target >= reference_position && target < end {
                        return None;
                    }
                    reference_position = end;
                }
                Feature::InsertBase { .. } => {
                    read_position = read_position.checked_add(1)?;
                }
                Feature::SoftClip { bases, .. } => {
                    read_position = read_position.checked_add(bases.len())?;
                }
                Feature::Scores { .. }
                | Feature::QualityScore { .. }
                | Feature::Padding { .. }
                | Feature::HardClip { .. } => {}
            }
        }

        let remaining = self.read_length.checked_sub(usize::from(read_position) - 1)?;
        let end = reference_position.checked_add(remaining)?;
        if target >= reference_position && target < end {
            let offset = usize::from(target) - usize::from(reference_position);
            let read_index = usize::from(read_position) - 1 + offset;
            let quality_score = quality_scores.get(read_index).copied().unwrap_or(0);
            Some((reference_base, quality_score))
        } else {
            None
        }
    }
}

impl Default for Record<'_> {
    fn default() -> Self {
        Self {
            id: 0,
            header: None,
            reference_sequence: None,
            substitution_matrix: SubstitutionMatrix::default(),
            bam_flags: sam::alignment::record::Flags::UNMAPPED,
            cram_flags: Flags::default(),
            reference_sequence_id: None,
            read_length: 0,
            alignment_start: None,
            read_group_id: None,
            name: None,
            mate_flags: MateFlags::default(),
            mate_reference_sequence_id: None,
            mate_alignment_start: None,
            template_length: 0,
            mate_distance: None,
            data: Vec::new(),
            sequence: Cow::from(&[]),
            features: Vec::new(),
            mapping_quality: None,
            quality_scores: Cow::from(&[]),
        }
    }
}

impl sam::alignment::Record for Record<'_> {
    fn name(&self) -> Option<&BStr> {
        self.name.as_deref().map(|name| name.as_bstr())
    }

    fn flags(&self) -> io::Result<sam::alignment::record::Flags> {
        Ok(self.bam_flags)
    }

    fn reference_sequence_id<'r, 'h: 'r>(
        &'r self,
        _: &'h sam::Header,
    ) -> Option<io::Result<usize>> {
        self.reference_sequence_id.map(Ok)
    }

    fn alignment_start(&self) -> Option<io::Result<Position>> {
        self.alignment_start.map(Ok)
    }

    fn mapping_quality(&self) -> Option<io::Result<MappingQuality>> {
        self.mapping_quality.map(Ok)
    }

    fn cigar(&self) -> Box<dyn sam::alignment::record::Cigar + '_> {
        Box::new(Cigar::new(
            &self.features,
            self.bam_flags.is_unmapped(),
            self.read_length,
        ))
    }

    fn mate_reference_sequence_id<'r, 'h: 'r>(
        &'r self,
        _: &'h sam::Header,
    ) -> Option<io::Result<usize>> {
        self.mate_reference_sequence_id.map(Ok)
    }

    fn mate_alignment_start(&self) -> Option<io::Result<Position>> {
        self.mate_alignment_start.map(Ok)
    }

    fn template_length(&self) -> io::Result<i32> {
        Ok(self.template_length)
    }

    fn sequence(&self) -> Box<dyn sam::alignment::record::Sequence + '_> {
        if self.bam_flags.is_unmapped() || self.cram_flags.sequence_is_missing() {
            Box::new(Bases(&self.sequence[..]))
        } else {
            let (reference_sequence, alignment_start) = match self.reference_sequence.as_ref() {
                Some(ReferenceSequence::Embedded {
                    reference_start,
                    sequence,
                }) => {
                    let alignment_start = usize::from(self.alignment_start.unwrap());
                    let offset = usize::from(*reference_start);
                    let offset_alignment_start =
                        Position::new(alignment_start - offset + 1).unwrap();
                    (Some(*sequence), offset_alignment_start)
                }
                Some(ReferenceSequence::External { sequence, .. }) => {
                    (Some((**sequence).as_ref()), self.alignment_start.unwrap())
                }
                None => (None, Position::MIN),
            };

            Box::new(Sequence::new(
                reference_sequence,
                self.substitution_matrix.clone(),
                &self.features,
                alignment_start,
                self.read_length,
            ))
        }
    }

    fn quality_scores(&self) -> Box<dyn sam::alignment::record::QualityScores + '_> {
        if self.bam_flags.is_unmapped() || self.cram_flags.quality_scores_are_stored_as_array() {
            Box::new(Scores(&self.quality_scores[..]))
        } else {
            Box::new(QualityScores::new(&self.features, self.read_length))
        }
    }

    fn data(&self) -> Box<dyn sam::alignment::record::Data + '_> {
        if let Some(header) = self.header {
            Box::new(Data::new(header, &self.data, self.read_group_id))
        } else {
            Box::new(sam::alignment::record_buf::Data::default())
        }
    }

    fn alignment_span(&self) -> Option<io::Result<usize>> {
        Some(Ok(self.alignment_span()))
    }

    fn alignment_end(&self) -> Option<io::Result<Position>> {
        self.alignment_end().map(Ok)
    }
}

struct Bases<'c>(&'c [u8]);

impl sam::alignment::record::Sequence for Bases<'_> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, i: usize) -> Option<u8> {
        self.0.get(i).copied()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = u8> + '_> {
        Box::new(self.0.iter().copied())
    }
}

struct Scores<'c>(&'c [u8]);

impl sam::alignment::record::QualityScores for Scores<'_> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<u8>> + '_> {
        Box::new(self.0.iter().copied().map(Ok))
    }
}

pub(crate) fn calculate_alignment_span(read_length: usize, features: &[Feature]) -> usize {
    features
        .iter()
        .fold(read_length, |alignment_span, feature| match feature {
            Feature::Insertion { bases, .. } => alignment_span - bases.len(),
            Feature::InsertBase { .. } => alignment_span - 1,
            Feature::Deletion { len, .. } => alignment_span + len,
            Feature::ReferenceSkip { len, .. } => alignment_span + len,
            Feature::SoftClip { bases, .. } => alignment_span - bases.len(),
            _ => alignment_span,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_alignment_span() -> Result<(), noodles_core::position::TryFromIntError> {
        let features = [];
        assert_eq!(calculate_alignment_span(4, &features), 4);

        let features = [Feature::HardClip {
            position: Position::try_from(1)?,
            len: 4,
        }];
        assert_eq!(calculate_alignment_span(4, &features), 4);

        let features = [
            Feature::Insertion {
                position: Position::try_from(1)?,
                bases: Cow::from(b"AC"),
            },
            Feature::InsertBase {
                position: Position::try_from(4)?,
                base: b'G',
            },
            Feature::Deletion {
                position: Position::try_from(6)?,
                len: 3,
            },
            Feature::ReferenceSkip {
                position: Position::try_from(10)?,
                len: 5,
            },
            Feature::SoftClip {
                position: Position::try_from(16)?,
                bases: Cow::from(b"ACGT"),
            },
        ];
        assert_eq!(calculate_alignment_span(20, &features), 21);

        Ok(())
    }
}
