use std::{
    borrow::Cow,
    ffi::CStr,
    io::{self, Read},
    mem,
    slice,
};

pub trait IntoDatabaseValue {
    fn database_byte_size(&self) -> usize;
    fn copy_into_database(&self, bytes: &mut [u8]) -> io::Result<()>;
}

pub trait FromDatabaseValue: Sized {
    fn copy_from_database(bytes: &[u8]) -> io::Result<Self>;
}

pub trait AsDatabaseBytes {
    fn as_database_bytes(&self) -> Cow<[u8]>;
}

// Trait implementations
impl IntoDatabaseValue for [u8] {
    fn database_byte_size(&self) -> usize {
        self.len()
    }

    fn copy_into_database(&self, bytes: &mut [u8]) -> io::Result<()> {
        if bytes.len() < self.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Insufficient space in buffer",
            ));
        }
        bytes[..self.len()].copy_from_slice(self);
        Ok(())
    }
}

impl IntoDatabaseValue for str {
    fn database_byte_size(&self) -> usize {
        self.len()
    }

    fn copy_into_database(&self, bytes: &mut [u8]) -> io::Result<()> {
        if bytes.len() < self.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Insufficient space in buffer",
            ));
        }
        bytes[..self.len()].copy_from_slice(self.as_bytes());
        Ok(())
    }
}

impl FromDatabaseValue for String {
    fn copy_from_database(bytes: &[u8]) -> io::Result<Self> {
        Ok(String::from_utf8(bytes.to_vec())?)
    }
}

macro_rules! impl_from_database_value_for_int {
    ($type:ty) => {
        impl FromDatabaseValue for $type {
            fn copy_from_database(bytes: &[u8]) -> io::Result<Self> {
                if bytes.len() != mem::size_of::<$type>() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Incorrect byte size for type",
                    ));
                }
                let mut array = [0; mem::size_of::<$type>()];
                array.copy_from_slice(bytes);
                Ok(Self::from_ne_bytes(array))
            }
        }
    };
}

impl_from_database_value_for_int!(u32);
impl_from_database_value_for_int!(u64);

impl FromDatabaseValue for Vec<u8> {
    fn copy_from_database(bytes: &[u8]) -> io::Result<Self> {
        Ok(bytes.to_vec())
    }
}

impl AsDatabaseBytes for Vec<u8> {
    fn as_database_bytes(&self) -> Cow<[u8]> {
        Cow::Borrowed(self)
    }
}

impl AsDatabaseBytes for str {
    fn as_database_bytes(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.as_bytes())
    }
}

impl AsDatabaseBytes for CStr {
    fn as_database_bytes(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.to_bytes())
    }
}

macro_rules! impl_as_database_bytes_for_primitive {
    ($type:ty) => {
        impl AsDatabaseBytes for $type {
            fn as_database_bytes(&self) -> Cow<[u8]> {
                unsafe {
                    Cow::Borrowed(slice::from_raw_parts(
                        self as *const $type as *const u8,
                        mem::size_of::<$type>(),
                    ))
                }
            }
        }

        impl AsDatabaseBytes for [$type] {
            fn as_database_bytes(&self) -> Cow<[u8]> {
                unsafe {
                    Cow::Borrowed(slice::from_raw_parts(
                        self.as_ptr() as *const u8,
                        self.len() * mem::size_of::<$type>(),
                    ))
                }
            }
        }
    };
}

impl_as_database_bytes_for_primitive!(u8);
impl_as_database_bytes_for_primitive!(u16);
impl_as_database_bytes_for_primitive!(i16);
impl_as_database_bytes_for_primitive!(i32);
impl_as_database_bytes_for_primitive!(u64);
impl_as_database_bytes_for_primitive!(i64);
impl_as_database_bytes_for_primitive!(f32);
impl_as_database_bytes_for_primitive!(f64);
impl_as_database_bytes_for_primitive!(char);

