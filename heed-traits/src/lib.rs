use std::borrow::Cow;

pub trait BytesEncode {
    type EItem: ?Sized;

    fn bytes_encode(item: &Self::EItem) -> anyhow::Result<Cow<[u8]>>;
}

pub trait BytesDecode<'a> {
    type DItem: 'a;

    fn bytes_decode(bytes: &'a [u8]) -> anyhow::Result<Self::DItem>;
}
