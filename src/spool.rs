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

extern crate byteorder;
extern crate base64;
extern crate sled;
extern crate sphinxcrypto;

use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs::remove_file;
use byteorder::{ByteOrder, BigEndian};
use base64::encode;
use sled::{Db, Tree};

use sphinxcrypto::constants::{USER_FORWARD_PAYLOAD_SIZE};

use errors::{SpoolError, SpoolSetError};

/// The size of a message.
const MESSAGE_SIZE: usize = USER_FORWARD_PAYLOAD_SIZE;

/// The size of a message identity in bytes.
const MESSAGE_ID_SIZE: usize = 4;

/// The size of a spool in bytes.
const SPOOL_SIZE: usize = 1000;

/// The metadata tree identity.
const META_TREE_ID: &[u8] = b"meta_tree_id";

/// The key whose value points to the index of the end of the spool.
static END_KEY: &'static [u8] = b"key";

/// Flush spool set writeback cache every 10 seconds.
const SPOOL_SET_FLUSH_FREQUENCY: u64 = 10000;

/// Spool set size. The maximum allowed number of spools.
pub const SPOOL_SET_SIZE: usize = 10000;


/// Spool is an append only message spool.
pub struct Spool {
    path: PathBuf,
    last_key: u32,
    db: Db,
    meta: Arc<Tree>,
}

impl Spool {
    pub fn new<P: AsRef<Path>>(path: &P) -> Result<Spool, SpoolError> {

        fn increment_merge(_key: &[u8], old_value: Option<&[u8]>, new_value: &[u8]) -> Option<Vec<u8>> {
            if let Some(old_value_bytes) = old_value {
                let old: u32 = BigEndian::read_u32(old_value_bytes);
                let new: u32 = BigEndian::read_u32(new_value);
                if old == new {
                    return Some(old_value_bytes.to_vec())
                }
                if old > new {
                    return Some(old_value_bytes.to_vec())
                }
            }
            return Some(new_value.to_vec())
        }

        let spool_cfg_builder = sled::ConfigBuilder::default()
            .merge_operator(increment_merge)
            .path(path)
            .cache_capacity(SPOOL_SIZE * MESSAGE_SIZE)
            .use_compression(false)
            .flush_every_ms(Some(SPOOL_SET_FLUSH_FREQUENCY))
            .snapshot_after_ops(1000);
        let db = Db::start(spool_cfg_builder.build())?;
        let meta = db.open_tree(META_TREE_ID.to_vec())?;
        Ok(Spool {
            path: PathBuf::from(path.as_ref()),
            last_key: 0,
            db: db,
            meta: meta,
        })
    }

    pub fn purge(&mut self) -> Result<(), SpoolError> {
        self.meta.clear()?;
        self.db.drop_tree(META_TREE_ID)?;
        self.db.clear()?;
        *self = Self::new(&self.path)?;
        remove_file(&self.path)?;
        Ok(())
    }

    pub fn append(&mut self, message: [u8; MESSAGE_SIZE]) -> Result<(), SpoolError> {
        self.last_key += 1;
        let mut _last_key = [0; 4];
        BigEndian::write_u32(&mut _last_key, self.last_key);
        self.db.set(_last_key, message.to_vec())?;
        self.meta.merge(END_KEY, _last_key.to_vec())?;
        Ok(())
    }

    pub fn read(&self, key: &[u8]) -> Result<Option<sled::IVec>, SpoolError> {
        Ok(self.db.get(key)?)
    }
}

/// SpoolSet is essentially a set of spool identities which is
/// persisted to disk.
pub struct SpoolSet {
    db: Db,
}

impl SpoolSet {
    pub fn new(base_dir: &String) -> Result<SpoolSet, SpoolSetError> {
        let cache_cfg_builder = sled::ConfigBuilder::default()
            .path(Path::new(base_dir).join("spool_set"))
            .cache_capacity(SPOOL_SET_SIZE * MESSAGE_ID_SIZE)
            .use_compression(false)
            .flush_every_ms(Some(SPOOL_SET_FLUSH_FREQUENCY))
            .snapshot_after_ops(100);
        let cache_cfg = cache_cfg_builder.build();
        let cache = match Db::start(cache_cfg) {
            Ok(x) => x,
            Err(_e) => {
                return Err(SpoolSetError::CreateSpoolSetCacheFailed);
            },
        };
        Ok(SpoolSet{
            db: cache,
        })
    }
}
