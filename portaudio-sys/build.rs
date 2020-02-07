use std::path::Path;
use std::process::Command;

fn main() {
    let target = std::env::var("TARGET").unwrap();
   //  std::env::set_var("LLVM_CONFIG_PATH", "/usr/local/opt/llvm/bin/llvm-config");

    if !Path::new("portaudio/CMakeLists.txt").exists() {
        Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status()
            .expect("Could not update portaudio submodule. Is git in path?");
        assert!(Path::new("portaudio/CMakeLists.txt").exists(),
            "Could not fetch the portaudio submodule. Likely due to a git bug in an old version. \
             Please manually remove the portaudio directory and try again.");
    }

    #[cfg(feature = "regenerate_bindings")]
    bindgen::Builder::default()
        .header("portaudio/include/portaudio.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustified_non_exhaustive_enum("PaHostApiTypeId|PaErrorCode|PaStreamCallbackResult")
        .new_type_alias("PaStream")
        .blacklist_type("PaStreamCallbackFlags|PaStreamFlags|PaSampleFormat|PaError")
        .raw_line("use crate::{PaSampleFormat, PaStreamFlags, PaStreamCallbackFlags};")
        .raw_line("pub type PaError = PaErrorCode;")
        .derive_debug(false)
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings.")
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings to file.");

    // Actually build.
    let dst = cmake::Config::new("portaudio")
        .define("PA_BUILD_SHARED", "OFF")
        // Don't use legacy windows APIs (DirectSound, MME, WDMKS).
        .define("PA_USE_DS", "OFF")
        .define("PA_USE_WMME", "OFF")
        .define("PA_USE_WDMKS", "OFF")
        .define("PA_USE_WDMKS_DEVICE_INFO", "OFF")
        // Keep library names consistent in Windows (since we don't build shared).
        .define("PA_LIBNAME_ADD_SUFFIX", "OFF")
        // Enable the usage of the skeleton API.
        .cflag("-DPA_USE_SKELETON=1")
        .build();
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=portaudio");

    // OSX
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreServices");
    } else if target.contains("windows") {
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=uuid");
    } else if target.contains("linux") {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        // This is the easiest way I can think of to figure out if portaudio was compiled
        // with ALSA.
        if Path::new(&out_dir).join("include/pa_linux_alsa.h").exists() {
            println!("cargo:rustc-link-lib=asound");
        } else {
            println!("cargo:warning=Could not find ALSA (libasound2-dev) on this machine. \
                      Linux support will be disabled.");
        }
    }
}
