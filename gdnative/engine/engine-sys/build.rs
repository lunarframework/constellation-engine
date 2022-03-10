fn main() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    println!(
        "cargo:rustc-link-search={}",
        manifest_dir
            .join("cpp")
            .join("lib")
            .as_os_str()
            .to_str()
            .unwrap()
    );
    println!("cargo:rustc-link-lib=engine-cpp");
    // println!("cargo:rustc-link-lib=mfem");
}
