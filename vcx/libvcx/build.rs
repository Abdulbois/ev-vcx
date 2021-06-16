use std::fs;
use std::path::Path;
use std::{env, path::PathBuf};

extern crate toml;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("target={}", target);

    if env::var("LIBINDY_STATIC").is_ok() {
        let libindy_lib_path = env::var("LIBINDY_DIR").unwrap();
        println!("cargo:rustc-link-search=native={}", libindy_lib_path);
        println!("cargo:rustc-link-lib=static=indy");
    } else if target.contains("aarch64-linux-android")
        || target.contains("armv7-linux-androideabi")
        || target.contains("arm-linux-androideabi")
        || target.contains("i686-linux-android")
        || target.contains("x86_64-linux-android")
        || target.contains("aarch64-apple-ios")
        || target.contains("armv7-apple-ios")
        || target.contains("armv7s-apple-ios")
        || target.contains("i386-apple-ios")
        || target.contains("x86_64-apple-ios")
    {
        let libindy_lib_path = env::var("LIBINDY_DIR").expect("LIBINDY_DIR");
        let openssl = env::var("OPENSSL_LIB_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(env::var("OPENSSL_DIR").expect("OPENSSL_DIR or OPENSSL_LIB_DIR"))
                    .join("lib")
            });
        println!("cargo:rustc-link-search=native={}", libindy_lib_path);
        println!("cargo:rustc-link-lib=static=indy");
        println!("cargo:rustc-link-search=native={}", openssl.display());
        println!("cargo:rustc-link-lib=static=crypto");
        println!("cargo:rustc-link-lib=static=ssl");
    } else if target.contains("darwin") {
        // OSX specific logic
        println!("cargo:rustc-link-lib=sodium");
        println!("cargo:rustc-link-lib=zmq");
        println!("cargo:rustc-link-lib=indy");
        // OSX does not allow 3rd party libs to be installed in /usr/lib. Instead install it in /usr/local/lib
        println!("cargo:rustc-link-search=native=/usr/local/lib");
    } else if target.contains("-linux-") {
        // Linux specific logic
        println!("cargo:rustc-link-lib=indy");
        println!("cargo:rustc-link-search=native=/usr/lib");
    } else if target.contains("-windows-") {
        println!("cargo:rustc-link-lib=indy.dll");

        let profile = env::var("PROFILE").unwrap();
        println!("profile={}", profile);

        let output_dir = env::var("OUT_DIR").unwrap();
        println!("output_dir={}", output_dir);
        let output_dir = Path::new(output_dir.as_str());

        let indy_dir =
            env::var("INDY_DIR").unwrap_or(format!("..\\..\\libindy\\target\\{}", profile));
        println!("indy_dir={}", indy_dir);
        let indy_dir = PathBuf::from(indy_dir);

        let dst = output_dir.join("..\\..\\..\\..");
        println!("cargo:rustc-flags=-L {}", indy_dir.display());

        let files = [
            "indy.dll",
            "libeay32md.dll",
            "libsodium.dll",
            "libzmq.dll",
            "ssleay32md.dll",
        ];

        for f in files.iter() {
            let src_f = indy_dir.join(f);
            let dst_f = dst.join(f);
            if fs::copy(&src_f, &dst_f).is_ok() {
                println!("copy {} -> {}", src_f.display(), dst_f.display());
            }
        }
    }
    if env::var("CARGO_FEATURE_CI").is_ok() {
        println!("injecting version information");
        let revision = get_revision();
        let contents = format!(
            r#"pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub static REVISION: &str = "+{}";
"#,
            revision
        );
        fs::write("src/utils/version_constants.rs", contents).unwrap();
    } else {
        println!("NOT injecting version information");
    }
}

// Gets the revision number from the Cargo.toml file.
pub fn get_revision() -> String {
    let p = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
    let input = fs::read_to_string(p).unwrap();
    toml::from_str::<toml::Value>(&input)
        .ok()
        .and_then(|cnts| {
            Some(
                cnts["package"]["metadata"]["deb"]
                    .get("revision")?
                    .as_str()?
                    .to_string(),
            )
        })
        .unwrap_or_default()
}
