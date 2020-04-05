use std::error::Error;
use std::fs;
use std::path::Path;
use heed::{Database, RoTxn, RoIter, EnvOpenOptions};
use heed::types::{Str, OwnedType};

pub type BEU32 = heed::zerocopy::U32<heed::byteorder::BE>;

struct DiscoverIds<'txn> {
    ids_iter: RoIter<'txn, OwnedType<BEU32>, Str>,
    left_id: Option<u32>,
    right_id: Option<u32>,
    available_range: std::ops::Range<u32>,
}

impl DiscoverIds<'_> {
    pub fn new(mut ids_iter: RoIter<OwnedType<BEU32>, Str>) -> heed::Result<DiscoverIds> {
        let right_id = ids_iter.next().transpose()?.map(|(k, _)| k.get());
        let available_range = 0..right_id.unwrap_or(u32::max_value());
        Ok(DiscoverIds { ids_iter, left_id: None, right_id, available_range })
    }
}

impl Iterator for DiscoverIds<'_> {
    type Item = heed::Result<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.available_range.next() {
                // The available range gives us a new id, we return it.
                Some(id) => return Some(Ok(id)),
                // The available range is exhausted, we need to find the next one.
                None if self.available_range.end == u32::max_value() => return None,
                None => loop {
                    self.left_id = self.right_id.take();
                    self.right_id = match self.ids_iter.next() {
                        Some(Ok((k, _))) => Some(k.get()),
                        Some(Err(e)) => return Some(Err(e)),
                        None => None,
                    };

                    match (self.left_id, self.right_id) {
                        // We found a gap in the used ids, we can yield all ids
                        // until the end of the gap
                        (Some(l), Some(r)) => if l.saturating_add(1) != r {
                            self.available_range = (l + 1)..r;
                            break;
                        },
                        // The last used id has been reached, we can use all ids
                        // until u32 MAX
                        (Some(l), None) => {
                            self.available_range = l.saturating_add(1)..u32::max_value();
                            break;
                        },
                        _ => (),
                    }
                },
            }
        }
    }
}

// 0 "hello"  | "coucou" 1
// 1 "coucou" | "hello"  0
// 2 "papa"   | "kiki"   5
// 5 "kiki"   | "papa"   2
pub fn double_map<'txn>(
    rtxn: &'txn RoTxn,
    ids_userids: Database<OwnedType<BEU32>, Str>,
    userids_ids: Database<Str, OwnedType<BEU32>>,
    userids: &[&str],
) -> heed::Result<Vec<u32>>
{
    // We construct a cursor to get next available ids
    let ids_iter = ids_userids.iter(rtxn)?;
    let iter = DiscoverIds::new(ids_iter)?;

    let mut available_ids = Vec::new();
    for result in iter.take(userids.len()) {
        available_ids.push(result?);
    }

    Ok(available_ids)
}

fn main() -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(Path::new("target").join("zerocopy-double-map.mdb"))?;

    let env = EnvOpenOptions::new()
        .map_size(10 * 1024 * 1024 * 1024) // 10GB
        .max_dbs(10)
        .open(Path::new("target").join("zerocopy-double-map.mdb"))?;

    // here the key will be an str and the data will be a slice of u8
    let ids_userids: Database<OwnedType<BEU32>, Str> = env.create_database(Some("ids-userids"))?;
    let userids_ids: Database<Str, OwnedType<BEU32>> = env.create_database(Some("userids-ids"))?;

    // clear db
    let mut wtxn = env.write_txn()?;
    ids_userids.clear(&mut wtxn)?;
    userids_ids.clear(&mut wtxn)?;
    wtxn.commit()?;

    // preregister ids
    let mut wtxn = env.write_txn()?;
    ids_userids.put(&mut wtxn, &BEU32::new(0), "hello0")?;
    ids_userids.put(&mut wtxn, &BEU32::new(1), "hello1")?;
    // ids_userids.put(&mut wtxn, &BEU32::new(2), "hello2")?;
    ids_userids.put(&mut wtxn, &BEU32::new(3), "hello3")?;
    ids_userids.put(&mut wtxn, &BEU32::new(4), "hello4")?;
    wtxn.commit()?;

    let rtxn = env.read_txn()?;
    let ids = double_map(&rtxn, ids_userids, userids_ids, &[""; 100])?;
    println!("{:?}..{:?}", &ids[..10], &ids[ids.len() - 10..ids.len()]);

    let mut iter = ids.into_iter();
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.as_slice(), (5..=103).collect::<Vec<u32>>().as_slice());

    Ok(())
}
