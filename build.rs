use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // On windows or unix like architectures
    // if env::var("CARGO_CFG_WINDOWS").is_ok() || env::var("CARGO_CFG_UNIX").is_ok() {}

    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(any(windows, unix)) {
        let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let target = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or(String::from("target")));
        let profile = PathBuf::from(env::var("PROFILE").unwrap());
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let native = manifest.join("native").canonicalize().unwrap();

        let output_path = manifest.join(out_dir).canonicalize().unwrap();
        let target_path = manifest.join(target).join(profile).canonicalize().unwrap();

        let dealii_path = native.join("third-party/dealii").canonicalize().unwrap();
        let spacetime_path = native.join("spacetime").canonicalize().unwrap();

        let dealii_output_path = native.join("build/third-party/dealii");
        let spacetime_output_path = native.join("build/spacetime");

        fs::create_dir_all(&dealii_output_path).unwrap();
        fs::create_dir_all(&spacetime_output_path).unwrap();

        // env::set_var("CONSTELLATION_TARGET_DIR", target_path.as_os_str());
        // env::set_var(
        //     "CONSTELLATION_DEALII_OUTPUT_DIR",
        //     dealii_output_path.as_os_str(),
        // );

        let mut dealii_source_arg = OsString::from("-S ");
        dealii_source_arg.push(dealii_path.as_os_str());

        let mut dealii_build_arg = OsString::from("-B ");
        dealii_build_arg.push(dealii_output_path.as_os_str());

        println!("Test");
        // let mut dealii_build_arg = OsString::from("--build");
        // dealii_build_arg.push(" ");
        // dealii_build_arg.push(dealii_output_path.as_os_str());

        // Compile dealii
        Command::new("cmake")
            .arg(dealii_source_arg)
            .arg(dealii_build_arg)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        // // Compile spacetime
        // Command::new("cmake")
        //     .current_dir(spacetime_output_path)
        //     .arg(spacetime_path)
        //     .arg(".")
        //     .spawn()
        //     .unwrap()
        //     .wait()
        //     .unwrap();

        panic!("Running");
    }
}
