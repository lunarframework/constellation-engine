use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // On windows or unix like architectures
    // if env::var("CARGO_CFG_WINDOWS").is_ok() || env::var("CARGO_CFG_UNIX").is_ok() {}

    // println!("cargo:rerun-if-changed=build.rs");

    if cfg!(any(windows, unix)) {
        let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let target = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or(String::from("target")));
        let profile = PathBuf::from(env::var("PROFILE").unwrap_or(String::from("debug")));
        // let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let native = manifest.join("native").canonicalize().unwrap();

        if !native.join("third-party/dealii").exists() {
            println!("Retrieving dealii from git");

            fs::create_dir_all(native.join("third-party")).unwrap();

            // Git clone
            Command::new("git")
                .current_dir(&manifest)
                .arg("clone")
                .arg("--branch")
                .arg("dealii-9.3")
                .arg("https://github.com/dealii/dealii.git")
                .arg("native/third-party/dealii")
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }

        println!("Running cmake to configure packages");

        // Build Packages

        Command::new("cmake")
            .current_dir(&manifest)
            .arg("-S native/third-party/dealii")
            .arg("-B native/packages/dealii")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        println!("Running cmake to build packages");

        Command::new("cmake")
            .current_dir(&manifest)
            .arg("--build")
            .arg("native/packages/dealii")
            .arg("--config")
            .arg("debug")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        // Build native

        env::set_var(
            "CDYLIB_DIR",
            manifest.join(&target).join(&profile).as_os_str(),
        );

        println!("Running cmake to config native");

        Command::new("cmake")
            .current_dir(&manifest)
            .arg("-S native")
            .arg("-B native/build")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        println!("Running cmake to build native");

        Command::new("cmake")
            .current_dir(&manifest)
            .arg("--build")
            .arg("native/build")
            .arg("--config")
            .arg("debug")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        println!("Running cmake to install native");

        Command::new("cmake")
            .current_dir(&manifest)
            .arg("--install")
            .arg("native/build")
            .arg("--config")
            .arg("debug")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}
