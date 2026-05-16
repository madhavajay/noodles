mod compression_method;
mod content_type;

pub use self::{compression_method::CompressionMethod, content_type::ContentType};

/// A CRAM block content ID.
pub type ContentId = i32;
