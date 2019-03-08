
# multispool [![](https://travis-ci.org/katzenpost/multispool.png?branch=master)](https://www.travis-ci.org/katzenpost/multispool) [![](https://img.shields.io/crates/v/multispool.svg)](https://crates.io/crates/multispool) [![](https://docs.rs/multispool/badge.svg)](https://docs.rs/multispool/)

The multispool server functions as a gRPC plugin for the Katzenpost mix server.
That is to say, the service it adds to the mix network is that of multiple spools.
This allows remote mixnet users to create, destroy, append and read spools.


### katzenpost server configuration example

Note that the following paths should be replaced with the
path to the spool service executable file and the log directory:

```toml
  [[Provider.PluginKaetzchen]]
    Capability = "spool"
    Endpoint = "+spool"
    Disable = false
    Command = "/home/user/test_mixnet/bin/spool_server"
    MaxConcurrency = 1
    [Provider.PluginKaetzchen.Config]
      l = "/home/user/test_mixnet/service_logs"
```

### auto generate protobuf and grpc files

Modify the ``includes`` and ``input`` paths in the ``build.rs`` file
to point to the correct directory locations for the Katzenpost mix
server grpc.

### manually generate protobuf and grpc files

Running a "cargo build" should autogenerate the grpc and protobuf rust
code due to our build.rs file, however, you could also manually
generate the grpc and protobuf code using a command like the
following:

```bash
   # Set this to the location of your local github.com/katzenpost/multispool repo.
   # Yes, I have rust in my gopath, get over it.
   server=/home/user/gopath/src/github.com/katzenpost/multispool
   out=/home/user/gopath/src/github.com/katzenpost/multispool/src/proto
   protoc -I $server/common-plugin/proto/ $server/common-plugin/proto/kaetzchen.proto --rust-grpc_out=$out --rust_out=$out
```

### license

GNU AFFERO GENERAL PUBLIC LICENSE
