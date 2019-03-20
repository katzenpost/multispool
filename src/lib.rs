// lib.rs - gRPC Multi-Spool service.
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

//! gRPC Multi-Spool service.

#[macro_use]
extern crate log;

#[macro_use]
extern crate arrayref;

extern crate log4rs;
extern crate protobuf;
extern crate grpc;
extern crate tls_api;
extern crate tls_api_stub;
extern crate base64;
extern crate byteorder;
extern crate sled;
extern crate ed25519_dalek;
extern crate rand;
extern crate serde;
extern crate sphinxcrypto;

pub mod spool;
pub mod errors;
pub mod proto;
pub mod big_array;

use std::collections::HashMap;
use std::str;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use rand::Rng;
use rand::rngs::OsRng;
use ed25519_dalek::{PublicKey, Signature, SIGNATURE_LENGTH, PUBLIC_KEY_LENGTH};

use ::proto::kaetzchen::{Request, Response, Params, Empty};
use ::proto::kaetzchen_grpc::Kaetzchen;
use spool::{MultiSpool, SPOOL_ID_SIZE, MESSAGE_SIZE, MESSAGE_ID_SIZE};
use errors::MultiSpoolError;
use big_array::BigArray;

pub const CREATE_SPOOL_COMMAND: u8 = 0;
pub const PURGE_SPOOL_COMMAND: u8 = 1;
pub const APPEND_MESSAGE_COMMAND: u8 = 2;
pub const RETRIEVE_MESSAGE_COMMAND: u8 = 3;


#[derive(Deserialize)]
pub struct SpoolRequest {
    command: u8,
    #[serde(with = "BigArray")]
    spool_id: [u8; SPOOL_ID_SIZE],
    #[serde(with = "BigArray")]
    signature: [u8; SIGNATURE_LENGTH],
    #[serde(with = "BigArray")]
    public_key: [u8; PUBLIC_KEY_LENGTH],
    message_id: [u8; MESSAGE_ID_SIZE],
    message: Vec<u8>,
}

#[derive(Serialize, Default)]
pub struct SpoolResponse {
    #[serde(with = "BigArray")]
    spool_id: [u8; SPOOL_ID_SIZE],
    message: Vec<u8>,
    status: String,
}

pub struct SpoolService {
    multi_spool: Arc<Mutex<MultiSpool>>,
    params: HashMap<String, String>,
}

impl SpoolService {
    pub fn new(base_dir: &String, multi_spool: MultiSpool) -> Result<Self, MultiSpoolError> {
        Ok(SpoolService {
            multi_spool: Arc::new(Mutex::new(multi_spool)),
            params: HashMap::new(),
        })
    }
}

fn error_response(error_message: &'static str) -> SpoolResponse {
    SpoolResponse{
        spool_id: [0u8; SPOOL_ID_SIZE],
        message: vec![],
        status: error_message.to_string(),
    }
}

fn create_spool(spool_request: SpoolRequest, multi_spool: &mut MultiSpool) -> SpoolResponse {
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

impl Kaetzchen for SpoolService {
    fn on_request(&self, _m: grpc::RequestOptions, req: Request) -> grpc::SingleResponse<Response> {
        info!("request received {}", req.ID); // XXX
        if !req.HasSURB {
            return grpc::SingleResponse::err(grpc::Error::Other("failure, SURB not found with Request"))
        }
        let spool_request = match serde_cbor::from_slice::<SpoolRequest>(&req.Payload) {
            Ok(x) => x,
            Err(_) => {
                return grpc::SingleResponse::err(grpc::Error::Other("failure, malformed Request, not valid CBOR"))
            }
        };

        let mut r = Response::new();
        let mut spool_response = SpoolResponse::default();
        match spool_request.command {
            CREATE_SPOOL_COMMAND => {
                spool_response = create_spool(spool_request, &mut self.multi_spool.lock().unwrap());
            },
            PURGE_SPOOL_COMMAND => {

            },
            APPEND_MESSAGE_COMMAND => {

            },
            RETRIEVE_MESSAGE_COMMAND => {

            },
            _ => {
                spool_response = SpoolResponse{
                    spool_id: [0u8; SPOOL_ID_SIZE],
                    message: vec![],
                    status: "error: no such command".to_string(),
                };
            },
        }
        r.set_Payload(serde_cbor::to_vec(&spool_response).unwrap());
        grpc::SingleResponse::completed(r)
    }

    fn parameters(&self, _m: grpc::RequestOptions, _empty: Empty) -> grpc::SingleResponse<Params> {
        let mut params = Params::new();
        params.set_Map(self.params.clone());
        grpc::SingleResponse::completed(params)
    }
}
