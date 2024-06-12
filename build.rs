
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let proto_file = "./proto/gateway.proto";
    // let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    //
    // tonic_build::configure()
    //     .protoc_arg("--experimental_allow_proto3_optional") // for older systems
    //     .build_client(true)
    //     .build_server(true)
    //     .file_descriptor_set_path(out_dir.join("store_descriptor.bin"))
    //     .out_dir("./src/grpc/")
    //     .compile(&[proto_file], &["proto"])?;

    let bussiness = "src/proto/bussiness.proto";
    let vlc = "src/proto/vlc.proto";
    let zmessage = "src/proto/zmessage.proto";
    let gateway = "src/proto/gateway.proto";

    prost_build::compile_protos(&[bussiness,vlc,zmessage,gateway], &["src/"])?;

    Ok(())
}
