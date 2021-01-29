use heed::{types::*, Database, EnvOpenOptions};
use std::{fs, path::Path};

// In this test we are checking that we can append ordered entries in one
// database even if there is multiple databases which already contain entries.
fn main() -> anyhow::Result<()> {
    let env_path = Path::new("target").join("cursor-append.mdb");

    let _ = fs::remove_dir_all(&env_path);

    fs::create_dir_all(&env_path)?;
    let env = EnvOpenOptions::new()
        .map_size(10 * 1024 * 1024) // 10MB
        .max_dbs(3)
        .open(env_path)?;

    let first: Database<Str, Str> = env.create_database(Some("first"))?;
    let second: Database<Str, Str> = env.create_database(Some("second"))?;

    let mut wtxn = env.write_txn()?;

    // We fill the first database with entries.
    first.put(&mut wtxn, &"I am here", &"to test things")?;
    first.put(&mut wtxn, &"I am here too", &"for the same purpose")?;

    // We try to append ordered entries in the second database.
    let mut iter = second.iter_mut(&mut wtxn)?;

    iter.append(&"aaaa", &"lol")?;
    iter.append(&"abcd", &"lol")?;
    iter.append(&"bcde", &"lol")?;

    drop(iter);

    wtxn.commit()?;

    Ok(())
}
