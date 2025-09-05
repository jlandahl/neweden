/*
 * Copyright (c) 2019. David "Tiran'Sol" Soria Parra
 * All rights reserved.
 */

//! neweden is a rust library for system information, wayfinding and
//! range queries for the MMORPG Eve Online from CCP Games.
//!
//! Online data can come from multiple data sources. Most commonly
//! a CCP static dump from https://www.fuzzwork.co.uk/dump/.
//!
//! The library must be compiled with the appropriate features depending
//! on the desired backend for universe data. The `postgres` feature
//! allows for loading from Postgres via the Diesel ORM tool, while the
//! `sqlite` feature allows loading from a local SQLite file.
//!
//! The `rpc` feature is for internal use at the moment as the dependent
//! crate is not open sourced.

// Must be at the crate root
#[cfg(feature = "postgres")]
#[macro_use]
extern crate diesel;

pub mod builder;
pub mod navigation;
pub mod rules;
pub mod source;

#[cfg(feature = "search")]
mod search;
mod types;

pub use types::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
