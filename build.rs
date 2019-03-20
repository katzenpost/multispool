extern crate protoc_rust_grpc;

fn main() {
    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/proto",
        includes: &["/home/user/gopath/src/github.com/katzenpost/server/grpcplugin/proto/"],
        input: &["/home/user/gopath/src/github.com/katzenpost/server/grpcplugin/proto/kaetzchen.proto"],
        rust_protobuf: true,
        ..Default::default()
    }).expect("protoc-rust-grpc");
}
