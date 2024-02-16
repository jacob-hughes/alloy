use cmake;
use std::env;
use std::path::PathBuf;
use std::process::Command;

const BDWGC_REPO: &str = "https://github.com/softdevteam/bdwgc.git";
const BDWGC_ATOMICS_REPO: &str = "https://github.com/ivmai/libatomic_ops.git";
const BDWGC_DEFAULT_SRC_DIR: &str = "bdwgc";
const BDWGC_BUILD_DIR: &str = "lib";

#[cfg(not(all(target_pointer_width = "64", target_arch = "x86_64")))]
compile_error!("Requires x86_64 with 64 bit pointer width.");

fn run<F>(name: &str, mut configure: F)
where
    F: FnMut(&mut Command) -> &mut Command,
{
    let mut command = Command::new(name);
    let configured = configure(&mut command);
    if !configured.status().is_ok() {
        panic!("failed to execute {:?}", configured);
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut bdwgc_src = PathBuf::from(&out_dir);

    match env::var("BDWGC") {
        Ok(path) => bdwgc_src.push(path),
        Err(_) => bdwgc_src.push(BDWGC_DEFAULT_SRC_DIR),
    }

    let mut build_dir = PathBuf::from(&out_dir);
    build_dir.push(BDWGC_BUILD_DIR);

    if !bdwgc_src.exists() && env::var("BDWGC").is_err() {
        run("git", |cmd| cmd.arg("clone").arg(BDWGC_REPO).arg(&bdwgc_src));
        run("git", |cmd| cmd.arg("clone").arg(BDWGC_ATOMICS_REPO).current_dir(&bdwgc_src));
    }

    let mut build = cmake::Config::new(&bdwgc_src);
    build
        .pic(true)
        .define("BUILD_SHARED_LIBS", "OFF")
        .cflag("-DGC_ALWAYS_MULTITHREADED")
        .cflag("-DGC_JAVA_FINALIZATION");

    if env::var("ENABLE_GC_ASSERTIONS").map_or(false, |v| v == "true") {
        build.define("enable_gc_assertions", "ON");
    }

    if env::var("ENABLE_GC_DEBUG").map_or(false, |v| v == "true") {
        build.profile("Debug");
    } else {
        build.profile("Release");
    }

    build.build();

    println!("cargo:rustc-link-search=native={}", &build_dir.display());
    println!("cargo:rustc-link-lib=static=gc");
}
