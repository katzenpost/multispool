// lib.rs - Multi-Spool mixnet protocol.
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

//! Multi-Spool protocol

#[macro_use] extern crate log;

#[macro_use] extern crate arrayref;

#[macro_use] extern crate serde_derive;

#[macro_use] extern crate serde;

extern crate serde_bytes;

extern crate log4rs;
extern crate base64;
extern crate byteorder;
extern crate sled;
extern crate ed25519_dalek;
extern crate rand;
extern crate sphinxcrypto;

pub mod spool;
pub mod errors;
pub mod big_array;

use std::str;
use serde::{Deserialize, Serialize};
use rand::rngs::OsRng;
use ed25519_dalek::{PublicKey, Signature, SIGNATURE_LENGTH, PUBLIC_KEY_LENGTH};

use spool::{MultiSpool, SPOOL_ID_SIZE, MESSAGE_ID_SIZE, MESSAGE_SIZE};
use errors::MultiSpoolError;
use big_array::BigArray;

pub const CREATE_SPOOL_COMMAND: u8 = 0;
pub const PURGE_SPOOL_COMMAND: u8 = 1;
pub const APPEND_MESSAGE_COMMAND: u8 = 2;
pub const RETRIEVE_MESSAGE_COMMAND: u8 = 3;


#[derive(Deserialize)]
pub struct SpoolRequest {
    pub command: u8,
    #[serde(with = "BigArray")]
    pub spool_id: [u8; SPOOL_ID_SIZE],
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_LENGTH],
    #[serde(with = "BigArray")]
    pub public_key: [u8; PUBLIC_KEY_LENGTH],
    #[serde(with = "BigArray")]
    pub message_id: [u8; MESSAGE_ID_SIZE],
    #[serde(with = "serde_bytes")]
    pub message: Vec<u8>,
}

#[derive(Serialize, Default)]
pub struct SpoolResponse {
    #[serde(with = "BigArray")]
    pub spool_id: [u8; SPOOL_ID_SIZE],
    #[serde(with = "serde_bytes")]
    pub message: Vec<u8>,
    pub status: String,
}

fn error_response(error_message: &'static str) -> SpoolResponse {
    SpoolResponse{
        spool_id: [0u8; SPOOL_ID_SIZE],
        message: vec![],
        status: error_message.to_string(),
    }
}

pub fn create_spool(spool_request: SpoolRequest, multi_spool: &mut MultiSpool) -> SpoolResponse {
    let mut spool_response = SpoolResponse::default();
    if let Ok(signature) = Signature::from_bytes(&spool_request.signature) {
        if let Ok(pub_key) = PublicKey::from_bytes(&spool_request.public_key) {
            let mut csprng: OsRng = OsRng::new().unwrap();
            match multi_spool.create_spool(pub_key, signature, &mut csprng) {
                Ok(spool_id) => {
                    spool_response = SpoolResponse {
                        spool_id: spool_id,
                        message: vec![],
                        status: "OK".to_string(),
                    }
                },
                Err(_) => {
                    spool_response = error_response("error: invalid create spool failed");
                },
            };
        } else {
            spool_response = error_response("error: invalid ed25519 public key");
        }
    } else {
        spool_response = error_response("error: invalid signature");
    }
    spool_response
}

pub fn purge_spool(spool_request: SpoolRequest, multi_spool: &mut MultiSpool) -> SpoolResponse {
    let mut spool_response = SpoolResponse::default();
    if let Ok(signature) = Signature::from_bytes(&spool_request.signature) {
        if let Ok(pub_key) = PublicKey::from_bytes(&spool_request.public_key) {
            let mut csprng: OsRng = OsRng::new().unwrap();
            match multi_spool.purge_spool(spool_request.spool_id, signature) {
                Ok(_) => {
                    spool_response = SpoolResponse {
                        spool_id: spool_request.spool_id,
                        message: vec![],
                        status: "OK".to_string(),
                    }
                },
                Err(_) => {
                    spool_response = error_response("error: purge spool failed");
                },
            }
        } else {
            spool_response = error_response("error: invalid ed25519 public key");
        }
    } else {
        spool_response = error_response("error: invalid signature");
    }
    spool_response
}

pub fn append_to_spool(spool_request: SpoolRequest, multi_spool: &mut MultiSpool) -> SpoolResponse {
    let mut spool_response = SpoolResponse::default();
    let mut message = [0u8; MESSAGE_SIZE];
    message.copy_from_slice(&spool_request.message);
    match multi_spool.append_to_spool(spool_request.spool_id, message) {
        Ok(_) => {
            spool_response = SpoolResponse {
                spool_id: spool_request.spool_id,
                message: vec![],
                status: "OK".to_string(),
            }
                },
        Err(_) => {
            spool_response = error_response("error: purge spool failed");
        },
    }
    spool_response
}

pub fn read_from_spool(spool_request: SpoolRequest, multi_spool: &MultiSpool) -> SpoolResponse {
    let mut spool_response = SpoolResponse::default();
    if let Ok(signature) = Signature::from_bytes(&spool_request.signature) {
        if let Ok(pub_key) = PublicKey::from_bytes(&spool_request.public_key) {
            let mut csprng: OsRng = OsRng::new().unwrap();
            match multi_spool.read_from_spool(spool_request.spool_id, signature, &spool_request.message_id) {
                Ok(response_message) => {
                    spool_response = SpoolResponse {
                        spool_id: spool_request.spool_id,
                        message: response_message.to_vec(),
                        status: "OK".to_string(),
                    }
                },
                Err(_) => {
                    spool_response = error_response("error: purge spool failed");
                },
            }
        } else {
            spool_response = error_response("error: invalid ed25519 public key");
        }
    } else {
        spool_response = error_response("error: invalid signature");
    }
    spool_response
}
