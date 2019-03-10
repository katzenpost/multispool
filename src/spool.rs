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
extern crate arrayref;
extern crate ed25519_dalek;
extern crate sphinxcrypto;

use std::sync::Arc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::remove_file;
use byteorder::{ByteOrder, BigEndian};
use sled::{Db, Tree};
use ed25519_dalek::{PublicKey, Signature};
use rand::CryptoRng;
use rand::Rng;

use sphinxcrypto::constants::{USER_FORWARD_PAYLOAD_SIZE};

use errors::{SpoolError, SpoolSetError, MultiSpoolError};

// Spool constants

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

// SpoolSet constants

/// Spool identity size in bytes.
const SPOOL_ID_SIZE: usize = 12;

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

    pub fn read(&self, message_id: &[u8; MESSAGE_ID_SIZE]) -> Result<[u8; MESSAGE_SIZE], SpoolError> {
        if let Some(message) = self.db.get(message_id)? {
            return Ok(*array_ref![message, 0, MESSAGE_SIZE])
        }
        return Err(SpoolError::NoSuchMessage)
    }
}

/// SpoolSet is essentially a persistent set of spool identities.
pub struct SpoolSet {
    db: Db,
    meta: Arc<Tree>,
}

impl SpoolSet {
    pub fn new<P: AsRef<Path>>(path: &P) -> Result<SpoolSet, SpoolSetError> {
        let cache_cfg_builder = sled::ConfigBuilder::default()
            .path(path)
            .cache_capacity(SPOOL_SET_SIZE * SPOOL_ID_SIZE)
            .use_compression(false)
            .flush_every_ms(Some(SPOOL_SET_FLUSH_FREQUENCY))
            .snapshot_after_ops(100);
        let cache_cfg = cache_cfg_builder.build();
        let db = Db::start(cache_cfg)?;
        let meta = db.open_tree(META_TREE_ID.to_vec())?;
        Ok(SpoolSet{
            db: db,
            meta: meta,
        })
    }

    pub fn put(&mut self, spool_id: [u8; SPOOL_ID_SIZE], public_key: PublicKey) -> Result<(), SpoolSetError> {
        self.db.set(spool_id.to_vec(), vec![])?;
        self.meta.set(spool_id.to_vec(), public_key.to_bytes().to_vec())?;
        Ok(())
    }

    pub fn has(&self, spool_id: [u8; SPOOL_ID_SIZE]) -> Result<bool, SpoolSetError> {
        Ok(self.db.contains_key(spool_id.to_vec())?)
    }

    pub fn delete(&mut self, spool_id: [u8; SPOOL_ID_SIZE]) -> Result<(), SpoolSetError> {
        self.db.del(spool_id.to_vec())?;
        self.meta.del(spool_id.to_vec())?;
        Ok(())
    }

    pub fn keys<'a>(&'a self) -> impl 'a + DoubleEndedIterator<Item = Result<Vec<u8>, sled::Error<()>>> {
        self.db.iter().keys()
    }

    pub fn get_public_key(&self, spool_id: [u8; SPOOL_ID_SIZE]) -> Result<PublicKey, SpoolSetError> {
        if let Some(pub_key) = self.meta.get(spool_id.to_vec())? {
            return Ok(PublicKey::from_bytes(&pub_key)?);
        }
        Err(SpoolSetError::NoSuchSpoolId)
    }
}

/// MultiSpool allows for accessing multiple spools.
pub struct MultiSpool {
    map: HashMap<[u8; SPOOL_ID_SIZE], Spool>,
    spool_set: SpoolSet,
    base_dir: String,
}

fn spool_path(base_dir: &String, spool_id: [u8; SPOOL_ID_SIZE]) -> PathBuf {
    let path = Path::new(base_dir).join(format!("spool.{}.sled", base64::encode(&spool_id)));
    let pathbuf: PathBuf = path.to_owned();
    pathbuf
}

impl MultiSpool {

    pub fn new(base_dir: &String) -> Result<Self, MultiSpoolError> {
        let spool_set_path = Path::new(base_dir).join("spool_set.sled");
        let spool_set = SpoolSet::new(&spool_set_path)?;
        let mut map = HashMap::new();
        for spool_id_result in spool_set.keys() {
            let raw_spool_id = spool_id_result?;
            let spool_id = *array_ref![raw_spool_id, 0, SPOOL_ID_SIZE];
            let path = spool_path(base_dir, spool_id.clone());
            map.insert(spool_id, Spool::new(&path)?);
        }
        Ok(MultiSpool {
            map: map,
            spool_set: spool_set,
            base_dir: base_dir.clone(),
        })
    }

    fn get_mut_spool(&mut self, spool_id: [u8; SPOOL_ID_SIZE]) -> Result<&mut Spool, MultiSpoolError> {
        let spool: &mut Spool = match self.map.get_mut(&spool_id) {
            Some(x) => x,
            None => {
                return Err(MultiSpoolError::NoSuchSpool);
            },
        };
        Ok(spool)
    }

    fn get_spool(&self, spool_id: [u8; SPOOL_ID_SIZE]) -> Result<&Spool, MultiSpoolError> {
        if let Some(spool) = self.map.get(&spool_id) {
            return Ok(spool)
        }
        Err(MultiSpoolError::NoSuchSpool)
    }

    pub fn create_spool<T>(&mut self,
                           public_key: PublicKey,
                           signature: Signature,
                           csprng: &mut T)
                           -> Result<[u8; SPOOL_ID_SIZE], MultiSpoolError>
    where
        T: CryptoRng + Rng,
    {
        public_key.verify(&public_key.to_bytes(), &signature)?;
        let mut spool_id = [0u8; SPOOL_ID_SIZE];
        csprng.fill_bytes(&mut spool_id);
        let spool_path = spool_path(&self.base_dir, spool_id);
        self.spool_set.put(spool_id, public_key)?;
        self.map.insert(spool_id, Spool::new(&spool_path)?);
        Err(MultiSpoolError::NoSuchSpool) // XXX
    }

    pub fn purge_spool(&mut self, spool_id: [u8; SPOOL_ID_SIZE], signature: Signature) -> Result<(), MultiSpoolError> {
        let pub_key = self.spool_set.get_public_key(spool_id)?;
        pub_key.verify(&pub_key.to_bytes(), &signature)?;
        {
            let spool = self.get_mut_spool(spool_id)?;
            spool.purge()?;
        }
        self.spool_set.delete(spool_id)?;
        self.map.remove(&spool_id);
        Ok(())
    }

    pub fn append_to_spool(&mut self,
                           spool_id: [u8; SPOOL_ID_SIZE],
                           message: [u8; MESSAGE_SIZE])
                           -> Result<(), MultiSpoolError> {
        let spool = self.get_mut_spool(spool_id)?;
        spool.append(message)?;
        return Ok(())
    }

    pub fn read_from_spool(&self,
                           spool_id: [u8; SPOOL_ID_SIZE],
                           signature: Signature,
                           message_id: &[u8; MESSAGE_ID_SIZE])
                           -> Result<[u8; MESSAGE_SIZE], MultiSpoolError> {
        let pub_key = self.spool_set.get_public_key(spool_id)?;
        pub_key.verify(&pub_key.to_bytes(), &signature)?;
        Ok(self.get_spool(spool_id)?.read(message_id)?)
    }
}
