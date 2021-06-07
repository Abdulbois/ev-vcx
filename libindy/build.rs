use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("target={}", target);

    let sodium_static = env::var("CARGO_FEATURE_SODIUM_STATIC").ok();
    println!("sodium_static={:?}", sodium_static);

    if sodium_static.is_some() {
        println!("cargo:rustc-link-lib=static=sodium");
    }

    if target.contains("-windows-") {
        // do not build c-code on windows, use binaries
        let output_dir = env::var("OUT_DIR").unwrap();
        let prebuilt_dir = env::var("INDY_PREBUILT_DEPS_DIR").unwrap();

        let dst = Path::new(&output_dir[..]).join("..\\..\\..");
        let prebuilt_lib = Path::new(&prebuilt_dir[..]).join("lib");

        println!("cargo:rustc-link-search=native={}", prebuilt_dir);
        println!("cargo:rustc-flags=-L {}\\lib", prebuilt_dir);
        println!("cargo:include={}\\include", prebuilt_dir);

        let files = [
            "libeay32md.dll",
            "libsodium.dll",
            "libzmq.dll",
            "ssleay32md.dll",
        ];

        for f in files.iter() {
            let src_f = prebuilt_lib.join(f);
            let dst_f = dst.join(f);
            if fs::copy(&src_f, &dst_f).is_ok() {
                println!("copy {} -> {}", src_f.display(), dst_f.display());
            }
        }
    } else if target.contains("linux-android") {
        //statically link files

        let openssl = env::var("OPENSSL_LIB_DIR").map(PathBuf::from).unwrap_or_else(|_| {
            PathBuf::from(env::var("OPENSSL_DIR").expect("OPENSSL_LIB_DIR or OPENSSL_DIR"))
                .join("lib")
        });

        let sodium = env::var("SODIUM_LIB_DIR").expect("SODIUM_LIB_DIR");

        let zmq = env::var("LIBZMQ_LIB_DIR").map(PathBuf::from).unwrap_or_else(|_| {
            PathBuf::from(env::var("LIBZMQ_PREFIX").expect("LIBZMQ_PREFIX or LIBZMQ_LIB_DIR"))
                .join("lib")
        });
        println!("cargo:rustc-link-search=native={}", openssl.display());
        println!("cargo:rustc-link-lib=static=crypto");
        println!("cargo:rustc-link-lib=static=ssl");
        println!("cargo:rustc-link-search=native={}", sodium);
        println!("cargo:rustc-link-lib=static=sodium");
        println!("cargo:rustc-link-search=native={}", zmq.display());
        println!("cargo:rustc-link-lib=static=zmq");
    }
}
