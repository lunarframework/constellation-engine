use cmake::Config;
use std::env;
use std::path::Path;

fn main() {
    // On windows or unix like architectures
    // if env::var("CARGO_CFG_WINDOWS").is_ok() || env::var("CARGO_CFG_UNIX").is_ok() {}

    if cfg!(any(windows, unix)) {
        let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
        let target = env::var("CARGO_TARGET_DIR").unwrap_or(String::from("target"));
        let profile = env::var("PROFILE").unwrap();

        let path = Path::new(&manifest)
            .join(target)
            .join(profile)
            .canonicalize()
            .unwrap();

        env::set_var("CDYLIB_DIR", path.into_os_string());

        let _native = Config::new("native").build();
    }
}
