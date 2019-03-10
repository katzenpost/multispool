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
extern crate sphinxcrypto;

pub mod spool;
pub mod errors;
pub mod proto;

use std::collections::HashMap;

use ::proto::kaetzchen::{Request, Response, Params, Empty};
use ::proto::kaetzchen_grpc::Kaetzchen;
use spool::MultiSpool;
use errors::MultiSpoolError;

pub struct SpoolService {
    params: HashMap<String, String>,
    multi_spool: MultiSpool,
}

impl SpoolService {
    pub fn new(base_dir: &String) -> Result<Self, MultiSpoolError> {
        Ok(SpoolService {
            params: HashMap::new(),
            multi_spool: MultiSpool::new(base_dir)?
        })
    }
}

impl Kaetzchen for SpoolService {

    fn on_request(&self, _m: grpc::RequestOptions, req: Request) -> grpc::SingleResponse<Response> {
        if !req.HasSURB {
            return grpc::SingleResponse::err(grpc::Error::Other("failure, SURB not found with Request"))
        }
        info!("request received");
        let mut r = Response::new();
        r.set_Payload(req.Payload);
        grpc::SingleResponse::completed(r)
    }

    fn parameters(&self, _m: grpc::RequestOptions, _empty: Empty) -> grpc::SingleResponse<Params> {
        let mut params = Params::new();
        params.set_Map(self.params.clone());
        grpc::SingleResponse::completed(params)
    }
}
