fn main() {
    let headers = std::env::var("TWIZZLER_ABI_BUILTIN_HEADERS").ok();
    let sysroots = std::env::var("TWIZZLER_ABI_SYSROOTS").ok();
    let mut target = std::env::var("TARGET").unwrap();
    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("bindings.rs");

    // Building for the host: use the system headers, since io.h needs sys/select.h.
    //
    // This deliberately does not care whether TWIZZLER_ABI_SYSROOTS is set. It used to, and that
    // made the host copy unbuildable under xtask, which always sets it: the sysroots tree only ever
    // contains Twizzler targets, so the host build got `-nostdlibinc` plus an include path that
    // does not exist and failed on `sys/select.h`. Nothing noticed because the host std is normally
    // taken from stage0 and never rebuilt -- until you change libstd and it has to be.
    let host_build = headers.is_none() && target == std::env::var("HOST").unwrap();

    let prefix = "../include/twizzler/rt";

    let path = std::env::var("PATH").unwrap();
    let pwd = std::env::var("PWD").unwrap();
    unsafe {
        std::env::set_var("PATH", format!("{}/toolchain/install/bin:{}", pwd, path));
    }
    let mut bg = std::process::Command::new("bindgen");

    if let Some(val) = std::env::var("TWIZZLER_ABI_LLVM_CONFIG").ok() {
        bg.env("LLVM_CONFIG_PATH", val);
    }
    bg.arg("--override-abi").arg(".*=C-unwind");
    bg.arg("--use-core");
    bg.arg("--distrust-clang-mangling");
    bg.arg("--with-derive-default");
    bg.arg(format!("{}/__all.h", prefix));
    bg.arg("-o").arg(&out).arg("--").arg("-target").arg(&target);

    if headers.is_some() {
        bg.arg("-nostdinc");
    } else if !host_build {
        bg.arg("-nostdlibinc");
    }

    if let Some(headers) = headers {
        bg.arg("-I").arg(headers);
    }

    if let Some(sysroots) = sysroots.filter(|_| !host_build) {
        let sysheaders = format!("{}/{}/include", sysroots, target);
        bg.arg("-I").arg(sysheaders);
        // Stable equivalent of the former `replace_last`: the guard was already `ends_with`,
        // so this is a suffix rewrite. Kept off unstable APIs deliberately -- bootstrap builds
        // tools with a restricted `-Zallow-features`, so a `#![feature]` here fails the cargo
        // build (E0725) even though it is fine for the std build.
        if let Some(stem) = target.strip_suffix("-none") {
            target = format!("{}-twizzler", stem);
        }
        let sysheaders = format!("{}/{}/include", sysroots, target);
        bg.arg("-I").arg(sysheaders);
    }
    eprintln!("running: {:?}", bg);
    let status = bg.status().expect("failed to generate bindings");
    if !status.success() {
        panic!("failed to generate bindings");
    }
    println!("cargo::rerun-if-changed=../include");
}
