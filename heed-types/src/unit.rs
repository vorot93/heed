use bytemuck::PodCastError;
use heed_traits::{BytesDecode, BytesEncode};
use std::borrow::Cow;

/// Describes the `()` type.
pub struct Unit;

impl BytesEncode for Unit {
    type EItem = ();

    fn bytes_encode(_item: &Self::EItem) -> anyhow::Result<Cow<[u8]>> {
        Ok(Cow::Borrowed(&[]))
    }
}

impl BytesDecode<'_> for Unit {
    type DItem = ();

    fn bytes_decode(bytes: &[u8]) -> anyhow::Result<Self::DItem> {
        if bytes.is_empty() {
            Ok(())
        } else {
            Err(PodCastError::SizeMismatch.into())
        }
    }
}
