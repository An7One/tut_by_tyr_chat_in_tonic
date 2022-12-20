fn main() {
    tonic_build::configure()
        .type_attribute(
            ".",
            "#[derive(Eq, Hash, serde::Serialize, serde::Deserialize)]",
        )
        .out_dir("src/pb")
        .compile(&["./abi.proto"], &["./"])
        .unwrap();
    println!("cargo:rerun-if-changed=./abi.proto");
    println!("cargo:rerun-if-changed=./build.rs");
}
