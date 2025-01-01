fn main() {
    let output_path = std::env::current_dir().expect("").join("src").join("grpc").join("proto");
    let input_path = std::env::current_dir().expect("").join("proto").join("cute.proto");
    let include_path =  std::env::current_dir().expect("").join("proto");

    if let Err(e) = tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(output_path)
        .compile(&[input_path], &[include_path]) {
        println!("cargo:warning=tonic generate fail => {:?}",e)
    }
}