use std::{
    marker, mem,
    ops::{Deref, DerefMut},
    ptr,
};

use crate::{
    mdb::{error::mdb_result, ffi},
    *,
};

pub struct RoCursor<'txn> {
    cursor: *mut ffi::MDB_cursor,
    _marker: marker::PhantomData<&'txn ()>,
}

impl<'txn> RoCursor<'txn> {
    pub(crate) fn new<T>(txn: &'txn RoTxn<T>, dbi: ffi::MDB_dbi) -> Result<RoCursor<'txn>> {
        let mut cursor: *mut ffi::MDB_cursor = ptr::null_mut();

        unsafe { mdb_result(ffi::mdb_cursor_open(txn.txn, dbi, &mut cursor))? }

        Ok(RoCursor {
            cursor,
            _marker: marker::PhantomData,
        })
    }

    unsafe fn get(
        &mut self,
        op: ffi::MDB_cursor_op,
        key: Option<&[u8]>,
    ) -> Result<Option<(&'txn [u8], &'txn [u8])>> {
        let mut data_val = mem::MaybeUninit::uninit();

        let mut key_val = if let Some(key) = key {
            mem::MaybeUninit::new(crate::into_val(key))
        } else {
            mem::MaybeUninit::uninit()
        };

        let result = mdb_result(ffi::mdb_cursor_get(
            self.cursor,
            key_val.as_mut_ptr(),
            data_val.as_mut_ptr(),
            op,
        ));

        match result {
            Ok(()) => {
                let key = crate::from_val(key_val.assume_init());
                let data = crate::from_val(data_val.assume_init());
                Ok(Some((key, data)))
            }
            Err(e) if e.not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn current(&mut self) -> Result<Option<(&'txn [u8], &'txn [u8])>> {
        unsafe { self.get(ffi::cursor_op::MDB_GET_CURRENT, None) }
    }

    pub fn move_on_first(&mut self) -> Result<Option<(&'txn [u8], &'txn [u8])>> {
        // Move the cursor on the first database key
        unsafe { self.get(ffi::cursor_op::MDB_FIRST, None) }
    }

    pub fn move_on_last(&mut self) -> Result<Option<(&'txn [u8], &'txn [u8])>> {
        // Move the cursor on the first database key
        unsafe { self.get(ffi::cursor_op::MDB_LAST, None) }
    }

    pub fn move_on_key_greater_than_or_equal_to(
        &mut self,
        key: &[u8],
    ) -> Result<Option<(&'txn [u8], &'txn [u8])>> {
        // Move the cursor to the specified key
        unsafe { self.get(ffi::cursor_op::MDB_SET_RANGE, Some(key)) }
    }

    pub fn move_on_prev(&mut self) -> Result<Option<(&'txn [u8], &'txn [u8])>> {
        // Move the cursor to the previous non-dup key
        unsafe { self.get(ffi::cursor_op::MDB_PREV, None) }
    }

    pub fn move_on_next(&mut self) -> Result<Option<(&'txn [u8], &'txn [u8])>> {
        // Move the cursor to the next non-dup key
        unsafe { self.get(ffi::cursor_op::MDB_NEXT, None) }
    }
}

impl Drop for RoCursor<'_> {
    fn drop(&mut self) {
        unsafe { ffi::mdb_cursor_close(self.cursor) }
    }
}

pub struct RwCursor<'txn> {
    cursor: RoCursor<'txn>,
}

impl<'txn> RwCursor<'txn> {
    pub(crate) fn new<T>(txn: &'txn RwTxn<T>, dbi: ffi::MDB_dbi) -> Result<RwCursor<'txn>> {
        Ok(RwCursor {
            cursor: RoCursor::new(txn, dbi)?,
        })
    }

    pub fn del_current(&mut self) -> Result<bool> {
        // Delete the current entry
        let result = unsafe { mdb_result(ffi::mdb_cursor_del(self.cursor.cursor, 0)) };

        match result {
            Ok(()) => Ok(true),
            Err(e) if e.not_found() => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub fn put_current(&mut self, key: &[u8], data: &[u8]) -> Result<bool> {
        let mut key_val = unsafe { crate::into_val(&key) };
        let mut data_val = unsafe { crate::into_val(&data) };

        // Modify the pointed data
        let result = unsafe {
            mdb_result(ffi::mdb_cursor_put(
                self.cursor.cursor,
                &mut key_val,
                &mut data_val,
                ffi::MDB_CURRENT,
            ))
        };

        match result {
            Ok(()) => Ok(true),
            Err(e) if e.not_found() => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub fn append(&mut self, key: &[u8], data: &[u8]) -> Result<()> {
        let mut key_val = unsafe { crate::into_val(&key) };
        let mut data_val = unsafe { crate::into_val(&data) };

        // Modify the pointed data
        let result = unsafe {
            mdb_result(ffi::mdb_cursor_put(
                self.cursor.cursor,
                &mut key_val,
                &mut data_val,
                ffi::MDB_APPEND,
            ))
        };

        result.map_err(Into::into)
    }
}

impl<'txn> Deref for RwCursor<'txn> {
    type Target = RoCursor<'txn>;

    fn deref(&self) -> &Self::Target {
        &self.cursor
    }
}

impl DerefMut for RwCursor<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cursor
    }
}
