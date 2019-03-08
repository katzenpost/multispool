// spool.rs - Persistent spool data structures.
// Copyright (C) 2019  David Anthony Stainton.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! Spool data structures

extern crate base64;
extern crate sled;
extern crate appendix;
extern crate sphinxcrypto;

use std::sync::{Arc, Mutex};
use std::path::Path;
use base64::encode;

use sled::Db;
use appendix::Index;

use sphinxcrypto::constants::{RECIPIENT_ID_SIZE, USER_FORWARD_PAYLOAD_SIZE};

use errors::{SpoolError, SpoolSetError};

/// The size of a message identity in bytes.
const MESSAGE_ID_SIZE: usize = 4;

/// Flush spool set writeback cache every 10 seconds.
const SPOOL_SET_FLUSH_FREQUENCY: u64 = 10000;

/// Spool set size. The maximum allowed number of spools.
pub const SPOOL_SET_SIZE: usize = 10000;

/// SpoolSet is essentially a set of spool identities which is
/// persisted to disk.
pub struct SpoolSet {
    spools: Arc<Mutex<Db>>,
}

/// Spool is an append only message spool.
pub struct Spool {
    index: Index<[u8; MESSAGE_ID_SIZE], [u8; 10]>,
}

impl Spool {
    pub fn new(id: [u8; RECIPIENT_ID_SIZE], base_dir: &String) -> Result<Spool, SpoolError> {
        Ok(Spool {
            index: Index::new(&Path::new(base_dir).join(format!("{}.spool", encode(&id.to_vec()))))?,
        })
    }

    pub fn append() {

    }

    pub fn destroy() {

    }
}

impl SpoolSet {
    pub fn new(base_dir: &String) -> Result<SpoolSet, SpoolSetError> {
        let cache_cfg_builder = sled::ConfigBuilder::default()
            .path(Path::new(base_dir).join("spool_set"))
            .cache_capacity(SPOOL_SET_SIZE * MESSAGE_ID_SIZE)
            .use_compression(false)
            .flush_every_ms(Some(SPOOL_SET_FLUSH_FREQUENCY))
            .snapshot_after_ops(100_000); // XXX
        let cache_cfg = cache_cfg_builder.build();
        let cache = match Db::start(cache_cfg) {
            Ok(x) => x,
            Err(_e) => {
                return Err(SpoolSetError::CreateSpoolSetCacheFailed);
            },
        };
        Ok(SpoolSet{
            spools: Arc::new(Mutex::new(cache)),
        })
    }
}
