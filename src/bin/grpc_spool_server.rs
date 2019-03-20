
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate clap;

extern crate futures;
extern crate rand;
extern crate tls_api_stub;
extern crate multispool;

use clap::{Arg, App};
use std::path::Path;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use log::LevelFilter;

use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use multispool::errors::MultiSpoolError;
use multispool::proto::kaetzchen_grpc::KaetzchenServer;
use multispool::{SpoolResponse, SpoolRequest, create_spool};
use multispool::spool::{MultiSpool, SPOOL_ID_SIZE};

use multispool::proto::kaetzchen::{Request, Response, Params, Empty};
use multispool::proto::kaetzchen_grpc::Kaetzchen;


/// CORE_PROTOCOL_VERSION must match the plugin protocol version
/// that the server's go-plugin library is using.
const CORE_PROTOCOL_VERSION: usize = 1;

/// KAETZENPOST_PLUGIN_VERSION must match the
/// Katzenpost server plugin protocol version.
const KAETZENPOST_PLUGIN_VERSION: usize = 1;


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



fn init_logger(log_dir: &str) {
    use log4rs::append::file::FileAppender;

    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect();
    let log_path = Path::new(log_dir).join(format!("multispool_{}.log", rand_string));

    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(log_path)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("requests", Box::new(requests)))
        .build(Root::builder().appender("requests").build(LevelFilter::Info))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();
}

fn main() {
    let matches = App::new("Katzenpost SpoolService Service")
        .version("1.0")
        .author("David Stainton <dawuud@riseup.net>")
        .about("Functions as a plugin to be executed by the Katzenpost server.")
        .arg(Arg::with_name("data_dir")
             .short("d")
             .long("data_dir")
             .required(true)
             .value_name("DIR")
             .help("Sets the data directory.")
             .takes_value(true))
        .arg(Arg::with_name("log_dir")
             .short("l")
             .long("log_dir")
             .required(true)
             .value_name("DIR")
             .help("Sets the log directory.")
             .takes_value(true))
        .get_matches();
    let log_dir = matches.value_of("log_dir").unwrap();
    let data_dir = String::from(matches.value_of("data_dir").unwrap());

    // Ensure log_dir exists and is a directory.
    if !Path::new(log_dir).is_dir() {
        panic!("log_dir must exist and be a directory");
    }

    // Ensure data_dir exists and is a directory.
    if !Path::new(&data_dir).is_dir() {
        panic!("data_dir must exist and be a directory");
    }

    // Setup logging.
    init_logger(log_dir);

    // Start our grpc service.
    info!("multi-spool starting up");
    let mut server_builder: grpc::ServerBuilder<tls_api_stub::TlsAcceptor> = grpc::ServerBuilder::new();
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect();
    let socket = format!("/tmp/multispool_{}.sock", rand_string);
    server_builder.http.set_unix_addr(socket.to_string()).unwrap();
    let multi_spool = MultiSpool::new(&data_dir).unwrap();
    let spool_service = match SpoolService::new(&data_dir, multi_spool) {
        Ok(x) => x,
        Err(e) => {
            panic!(e);
        },
    };
    server_builder.add_service(KaetzchenServer::new_service_def(spool_service));
    server_builder.http.set_cpu_pool_threads(4); // XXX
    let _server = server_builder.build().expect("server");

    println!("{}|{}|unix|{}|grpc", CORE_PROTOCOL_VERSION, KAETZENPOST_PLUGIN_VERSION, socket);

    loop {
        thread::park();
    }
}
