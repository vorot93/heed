use std::error::Error;
use std::fs;
use std::path::Path;
use heed::{Database, RoTxn, EnvOpenOptions};
use heed::types::{Str, OwnedType};

pub type BEU16 = heed::zerocopy::U16<heed::byteorder::BE>;

// 0 "hello"  | "coucou" 1
// 1 "coucou" | "hello"  0
// 2 "papa"   | "kiki"   5
// 5 "kiki"   | "papa"   2
pub fn double_map<'txn>(
    rtxn: &'txn RoTxn,
    ids_userids: Database<OwnedType<BEU16>, Str>,
    userids_ids: Database<Str, OwnedType<BEU16>>,
    userids: &[&str],
) -> heed::Result<Vec<u16>>
{
    // We construct a cursor to get next available ids
    let mut ids_iter = ids_userids.iter(rtxn)?;
    let mut left_id;
    let mut right_id = ids_iter.next().transpose()?.map(|(k, _)| k.get());
    let mut available_range = 0..right_id.unwrap_or(u16::max_value());

    let mut available_ids = Vec::<u16>::new();

    loop {
        match available_range.next() {
            // The available range gives us a new id, we return it.
            Some(id) => available_ids.push(id),
            // The available range is exhausted, we need to find the next one.
            None if available_range.end == u16::max_value() => break,
            None => {
                loop {
                    left_id = right_id.take();
                    right_id = ids_iter.next().transpose()?.map(|(k, _)| k.get());

                    match (left_id, right_id) {
                        // We found a gap in the used ids, we can yield all ids
                        // until the end of the gap
                        (Some(l), Some(r)) => if l.saturating_add(1) != r {
                            available_range = (l + 1)..r;
                            break;
                        },
                        // The last used id has been reached, we can use all ids
                        // until u16 MAX
                        (Some(l), None) => {
                            available_range = l.saturating_add(1)..u16::max_value();
                            break;
                        },
                        _ => (),
                    }
                }
            }
        }
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
    let ids_userids: Database<OwnedType<BEU16>, Str> = env.create_database(Some("ids-userids"))?;
    let userids_ids: Database<Str, OwnedType<BEU16>> = env.create_database(Some("userids-ids"))?;

    // clear db
    let mut wtxn = env.write_txn()?;
    ids_userids.clear(&mut wtxn)?;
    userids_ids.clear(&mut wtxn)?;
    wtxn.commit()?;

    // preregister ids
    let mut wtxn = env.write_txn()?;
    ids_userids.put(&mut wtxn, &BEU16::new(0), "hello0")?;
    ids_userids.put(&mut wtxn, &BEU16::new(1), "hello1")?;
    // ids_userids.put(&mut wtxn, &BEU16::new(2), "hello2")?;
    ids_userids.put(&mut wtxn, &BEU16::new(3), "hello3")?;
    ids_userids.put(&mut wtxn, &BEU16::new(4), "hello4")?;
    wtxn.commit()?;

    let rtxn = env.read_txn()?;
    let ids = double_map(&rtxn, ids_userids, userids_ids, &[])?;
    println!("{:?}..{:?}", &ids[..10], &ids[ids.len() - 10..ids.len()]);

    Ok(())
}
