//! Index I/O functions

use crate::error::{Error, Result};
use crate::faiss_try;
use crate::index::{CpuIndex, FromInnerPtr, IndexImpl, NativeIndex};
use bytes::Bytes;
use faiss_sys::*;
use std::ffi::CString;
use std::os::raw::c_int;
use std::ptr;
use std::ptr::null_mut;
use std::sync::atomic::AtomicPtr;

pub use super::io_flags::IoFlags;

/// Write an index to a file.
///
/// # Error
///
/// This function returns an error if the description contains any byte with the value `\0` (since
/// it cannot be converted to a C string), or if the internal index writing operation fails.
pub fn write_index<I, P>(index: &I, file_name: P) -> Result<()>
where
    I: NativeIndex,
    I: CpuIndex,
    P: AsRef<str>,
{
    unsafe {
        let f = file_name.as_ref();
        let f = CString::new(f).map_err(|_| Error::BadFilePath)?;

        faiss_try(faiss_write_index_fname(index.inner_ptr(), f.as_ptr()))?;
        Ok(())
    }
}

pub unsafe fn serialize(index: &IndexImpl) -> Result<Vec<u8>> {
    unsafe {
        let mut size = 0;
        let mut capacity = 0;
        let mut bytes = null_mut();
        faiss_try(serialize_index(
            index.inner_ptr(),
            &mut bytes,
            &mut size,
            &mut capacity,
        ))?;
        let bytes = std::slice::from_raw_parts(bytes, size); //TODO: free memory
        let bytes = bytes.to_vec(); //TODO: avoid copy
        Ok(bytes)
    }
}

/// Read an index from a file.
///
/// # Error
///
/// This function returns an error if the description contains any byte with the value `\0` (since
/// it cannot be converted to a C string), or if the internal index reading operation fails.
pub fn read_index<P>(file_name: P) -> Result<IndexImpl>
where
    P: AsRef<str>,
{
    unsafe {
        let f = file_name.as_ref();
        let f = CString::new(f).map_err(|_| Error::BadFilePath)?;
        let mut inner = ptr::null_mut();
        faiss_try(faiss_read_index_fname(
            f.as_ptr(),
            IoFlags::MEM_RESIDENT.into(),
            &mut inner,
        ))?;
        Ok(IndexImpl::from_inner_ptr(inner))
    }
}

pub fn deserialize(bytes: &[u8]) -> Result<IndexImpl> {
    unsafe {
        let size = bytes.len() as usize;
        let bytes = bytes.as_ptr() as *const u8;
        let mut inner = null_mut();
        faiss_try(deserialize_index(bytes, size, &mut inner))?;
        Ok(IndexImpl::from_inner_ptr(inner))
    }
}

/// Read an index from a file with I/O flags.
///
/// You can memory map some index types with this.
///
/// # Error
///
/// This function returns an error if the description contains any byte with the value `\0` (since
/// it cannot be converted to a C string), or if the internal index reading operation fails.
pub fn read_index_with_flags<P>(file_name: P, io_flags: IoFlags) -> Result<IndexImpl>
where
    P: AsRef<str>,
{
    unsafe {
        let f = file_name.as_ref();
        let f = CString::new(f).map_err(|_| Error::BadFilePath)?;
        let mut inner = ptr::null_mut();
        faiss_try(faiss_read_index_fname(
            f.as_ptr(),
            io_flags.0 as c_int,
            &mut inner,
        ))?;
        Ok(IndexImpl::from_inner_ptr(inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::flat::FlatIndex;
    use crate::index::{Index, UpcastIndex};

    const D: u32 = 8;

    #[test]
    fn write_read() {
        let mut index = FlatIndex::new_l2(D).unwrap();
        assert_eq!(index.d(), D);
        assert_eq!(index.ntotal(), 0);
        let some_data = &[
            7.5_f32, -7.5, 7.5, -7.5, 7.5, 7.5, 7.5, 7.5, -1., 1., 1., 1., 1., 1., 1., -1., 4.,
            -4., -8., 1., 1., 2., 4., -1., 8., 8., 10., -10., -10., 10., -10., 10., 16., 16., 32.,
            25., 20., 20., 40., 15.,
        ];
        index.add(some_data).unwrap();
        assert_eq!(index.ntotal(), 5);

        let filepath = ::std::env::temp_dir().join("test_write_read.index");
        let filename = filepath.to_str().unwrap();
        write_index(&index, filename).unwrap();
        let index = read_index(&filename).unwrap();
        assert_eq!(index.ntotal(), 5);
        ::std::fs::remove_file(&filepath).unwrap();
    }

    #[test]
    fn serialize_deserialize() {
        let mut index = FlatIndex::new_l2(D).unwrap();
        assert_eq!(index.d(), D);
        assert_eq!(index.ntotal(), 0);
        let some_data = &[
            7.5_f32, -7.5, 7.5, -7.5, 7.5, 7.5, 7.5, 7.5, -1., 1., 1., 1., 1., 1., 1., -1., 4.,
            -4., -8., 1., 1., 2., 4., -1., 8., 8., 10., -10., -10., 10., -10., 10., 16., 16., 32.,
            25., 20., 20., 40., 15.,
        ];
        index.add(some_data).unwrap();
        assert_eq!(index.ntotal(), 5);

        let bytes = serialize(&index.upcast()).unwrap();
        let index = deserialize(&bytes).unwrap();
        assert_eq!(index.ntotal(), 5);
    }

    #[test]
    fn test_read_with_flags() {
        let index = read_index_with_flags("file_name", IoFlags::MEM_MAP | IoFlags::READ_ONLY);
        // we just want to ensure the method signature is right here
        assert!(index.is_err());
    }
}
