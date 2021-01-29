//! Crate `heed` is a high-level wrapper of [LMDB], high-level doesn't mean heavy (think about Rust).
//!
//! It provides you a way to store types in LMDB without any limit and with a minimal overhead as possible,
//! relying on the [bytemuck] library to avoid copying bytes when that's unnecessary and the serde library
//! when this is unavoidable.
//!
//! The Lightning Memory-Mapped Database (LMDB) directly maps files parts into main memory, combined
//! with the bytemuck library allows us to safely zero-copy parse and serialize Rust types into LMDB.
//!
//! [LMDB]: https://en.wikipedia.org/wiki/Lightning_Memory-Mapped_Database
//!
//! # Examples
//!
//! Discern let you open a database, that will support some typed key/data
//! and ensures, at compile time, that you'll write those types and not others.
//!
//! ```
//! use std::fs;
//! use std::path::Path;
//! use heed::{EnvOpenOptions, Database};
//! use heed::types::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! fs::create_dir_all(Path::new("target").join("bytemuck.mdb"))?;
//! let env = EnvOpenOptions::new().open(Path::new("target").join("bytemuck.mdb"))?;
//!
//! // we will open the default unamed database
//! let db: Database<Str, OwnedType<i32>> = env.create_database(None)?;
//!
//! // opening a write transaction
//! let mut wtxn = env.write_txn()?;
//! db.put(&mut wtxn, &"seven", &7)?;
//! db.put(&mut wtxn, &"zero", &0)?;
//! db.put(&mut wtxn, &"five", &5)?;
//! db.put(&mut wtxn, &"three", &3)?;
//! wtxn.commit()?;
//!
//! // opening a read transaction
//! // to check if those values are now available
//! let mut rtxn = env.read_txn()?;
//!
//! let ret = db.get(&rtxn, &"zero")?;
//! assert_eq!(ret, Some(0));
//!
//! let ret = db.get(&rtxn, &"five")?;
//! assert_eq!(ret, Some(5));
//! # Ok(()) }
//! ```

mod cursor;
mod database;
mod env;
mod iter;
mod lazy_decode;
mod mdb;
mod txn;

pub use bytemuck;
pub use byteorder;
use heed_traits as traits;
pub use heed_types as types;

use self::{
    cursor::{RoCursor, RwCursor},
    mdb::ffi::{from_val, into_val},
};
pub use self::{
    database::Database,
    env::{env_closing_event, CompactionOption, Env, EnvClosingEvent, EnvOpenOptions},
    iter::{
        RoIter, RoPrefix, RoRange, RoRevIter, RoRevPrefix, RoRevRange, RwIter, RwPrefix, RwRange,
        RwRevIter, RwRevPrefix, RwRevRange,
    },
    lazy_decode::{Lazy, LazyDecode},
    mdb::{error::Error as MdbError, flags},
    traits::{BytesDecode, BytesEncode},
    txn::{RoTxn, RwTxn},
};

use std::{io, result};

/// An helper type alias for [`Database`]s that are not typed and returns raw bytes.
pub type UntypedDatabase = Database<types::ByteSlice<'static>, types::ByteSlice<'static>>;

/// An error that encapsulates all possible errors in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`{0}`")]
    Io(#[from] io::Error),
    #[error("`{0}`")]
    Mdb(MdbError),
    #[error("error while encoding")]
    Encoding(anyhow::Error),
    #[error("error while decoding")]
    Decoding(anyhow::Error),
    #[error("database was previously opened with different types")]
    InvalidDatabaseTyping,
    #[error("database is in a closing phase, you can't open it at the same time")]
    DatabaseClosing,
}

impl From<MdbError> for Error {
    fn from(error: MdbError) -> Error {
        match error {
            MdbError::Other(e) => Error::Io(io::Error::from_raw_os_error(e)),
            _ => Error::Mdb(error),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
