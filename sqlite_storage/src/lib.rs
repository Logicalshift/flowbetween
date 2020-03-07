#[macro_use] extern crate rusqlite;

mod sqlite_core;
mod sqlite_storage;

#[cfg(test)] mod sqlite_core_tests;
#[cfg(test)] mod round_trip_tests;

pub use self::sqlite_storage::*;
