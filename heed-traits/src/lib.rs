use std::borrow::Cow;
use std::cmp::Ordering;

pub trait BytesEncode {
    type EItem: ?Sized;

    fn bytes_encode<'a>(item: &'a Self::EItem) -> Option<Cow<'a, [u8]>>;
}

pub trait BytesDecode<'a> {
    type DItem: 'a;

    fn bytes_decode(bytes: &'a [u8]) -> Option<Self::DItem>;
}

pub trait CustomKeyCmp {
    fn compare(a: &[u8], b: &[u8]) -> Ordering;
}
