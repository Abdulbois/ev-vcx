use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("target={}", target);

    if target.contains("-windows-") {
        // do not build c-code on windows, use binaries
        let output_dir = env::var("OUT_DIR").unwrap();
        let prebuilt_dir = env::var("INDY_PREBUILT_DEPS_DIR").unwrap();

        let dst = Path::new(&output_dir).join("..\\..\\..");
        let prebuilt_lib = Path::new(&prebuilt_dir).join("lib");

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
    } else {
        println!("cargo:rustc-link-lib=static=zmq");
        println!("cargo:rustc-link-lib=static=sodium");
        println!("cargo:rustc-link-lib=stdc++");
    }
}

