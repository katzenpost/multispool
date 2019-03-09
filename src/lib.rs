
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate protobuf;
extern crate grpc;
extern crate tls_api;
extern crate tls_api_stub;
extern crate base64;
extern crate byteorder;
extern crate sled;
extern crate sphinxcrypto;

pub mod spool;
pub mod errors;
pub mod proto;

use std::collections::HashMap;

use ::proto::kaetzchen::{Request, Response, Params, Empty};
use ::proto::kaetzchen_grpc::Kaetzchen;

pub struct SpoolService {
    params: HashMap<String, String>,
}

impl SpoolService {
    pub fn new() -> SpoolService {
        SpoolService {
            params: HashMap::new(),
        }
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
