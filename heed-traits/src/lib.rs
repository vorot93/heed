use std::borrow::Cow;

pub trait BytesEncode<'a> {
    type EItem: ?Sized + 'a;

    fn bytes_encode(item: &'a Self::EItem) -> Option<Cow<'a, [u8]>>;
}

pub trait BytesDecode<'a> {
    type DItem: 'a;

    fn bytes_decode(bytes: &'a [u8]) -> Option<Self::DItem>;
}

pub trait Database<'t> {
    type Key: 't;
    type Data: 't;
    type KeyCodec;
    type DataCodec;
    type Iter: 't;
    type Error;

    fn get<'a>(&'t self, txn: &'t (), key: &'a Self::Key) -> Result<Option<Self::Data>, Self::Error>
    where
        Self::KeyCodec: BytesEncode<'a, EItem = Self::Key>,
        Self::DataCodec: BytesDecode<'t, DItem = Self::Data>,
        Self::Data: 't;

    fn iter<'a>(&'t self, txn: &'t ()) -> Result<Self::Iter, Self::Error>;

    fn range<'a, R>(&'t self, txn: &'t (), range: &'a R) -> Result<Self::Iter, Self::Error>
    where
        Self::KeyCodec: BytesEncode<'a, EItem = Self::Key>,
        Self::Key: 'a,
        R: std::ops::RangeBounds<Self::Key>;
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::btree_map::Iter;
    use super::*;

    pub enum U32Codec {}

    impl BytesEncode<'_> for U32Codec {
        type EItem = u32;

        fn bytes_encode(item: &Self::EItem) -> Option<Cow<'_, [u8]>> {
            Some(Cow::Owned(item.to_be_bytes().to_vec()))
        }
    }

    impl BytesDecode<'_> for U32Codec {
        type DItem = u32;

        fn bytes_decode(bytes: &[u8]) -> Option<Self::DItem> {
            use std::convert::TryInto;
            bytes.try_into().ok().map(u32::from_be_bytes)
        }
    }

    pub struct BTreeMapIter<'a, KC, DC> {
        iter: Iter<'a, Vec<u8>, Vec<u8>>,
        _phantom: std::marker::PhantomData<(KC, DC)>,
    }

    impl<'a, KC, DC> Iterator for BTreeMapIter<'a, KC, DC>
    where
        KC: BytesDecode<'a>,
        DC: BytesDecode<'a>,
    {
        type Item = Result<(KC::DItem, DC::DItem), ()>;

        fn next(&mut self) -> Option<Self::Item> {
            let (key, data) = self.iter.next()?;
            let key = KC::bytes_decode(&key).unwrap();
            let data = DC::bytes_decode(&data).unwrap();
            Some(Ok((key, data)))
        }
    }

    impl<'t> Database<'t> for BTreeMap<Vec<u8>, Vec<u8>> {
        type Key = u32;
        type Data = u32;

        type KeyCodec = U32Codec;
        type DataCodec = U32Codec;
        type Iter = BTreeMapIter<'t, Self::KeyCodec, Self::DataCodec>;
        type Error = ();

        fn get<'a>(&'t self, txn: &'t (), key: &'a Self::Key) -> Result<Option<Self::Data>, Self::Error>
        where
            Self::KeyCodec: BytesEncode<'a, EItem = Self::Key>,
            Self::DataCodec: BytesDecode<'t, DItem = Self::Data>,
            Self::Data: 't,
        {
            let bytes = Self::KeyCodec::bytes_encode(key).unwrap();
            match self.get(bytes.as_ref()) {
                Some(data) => {
                    let data = Self::DataCodec::bytes_decode(&data).unwrap();
                    Ok(Some(data))
                },
                None => Ok(None),
            }
        }

        fn iter<'a>(&'t self, txn: &'t ()) -> Result<Self::Iter, Self::Error> {
            Ok(BTreeMapIter {
                iter: self.iter(),
                _phantom: std::marker::PhantomData,
            })
        }

        fn range<'a, R>(&'t self, txn: &'t (), range: &'a R) -> Result<Self::Iter, Self::Error>
        where
            Self::KeyCodec: BytesEncode<'a, EItem = Self::Key>,
            Self::Key: 'a,
            R: std::ops::RangeBounds<Self::Key>,
        {
            todo!()
        }
    }

    #[test]
    fn name() {
        unimplemented!();
    }
}
