fn main() {
    let headers = std::env::var("TWIZZLER_ABI_BUILTIN_HEADERS").ok();
    let sysroots = std::env::var("TWIZZLER_ABI_SYSROOTS").ok();
    let target = std::env::var("TARGET").unwrap();
    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("bindings.rs");

    // See the matching comment in rt-abi/build.rs.
    let host_build =
        headers.is_none() && sysroots.is_none() && target == std::env::var("HOST").unwrap();

    let prefix = "../include/twizzler";

    let path = std::env::var("PATH").unwrap();
    let pwd = std::env::var("PWD").unwrap();
    std::env::set_var("PATH", format!("{}/toolchain/install/bin:{}", pwd, path));
    let mut bg = std::process::Command::new("bindgen");

    if let Some(val) = std::env::var("TWIZZLER_ABI_LLVM_CONFIG").ok() {
        bg.env("LLVM_CONFIG_PATH", val);
    }
    bg.arg("--override-abi").arg(".*=C-unwind");
    bg.arg("--use-core");
    bg.arg("--with-derive-default");
    bg.arg(format!("{}/types.h", prefix));
    bg.arg("-o").arg(&out).arg("--").arg("-target").arg(&target);

    if headers.is_some() {
        bg.arg("-nostdinc");
    } else if !host_build {
        bg.arg("-nostdlibinc");
    }

    if let Some(headers) = headers {
        bg.arg("-I").arg(headers);
    }
    if let Some(sysroots) = sysroots {
        let sysheaders = format!("{}/{}/include", sysroots, target);
        bg.arg("-I").arg(sysheaders);
    }

    let status = bg.status().expect("failed to generate bindings");
    if !status.success() {
        panic!("failed to generate bindings");
    }

    println!("cargo::rerun-if-changed=../include");
}
