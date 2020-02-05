fn main() {
    let target = std::env::var("TARGET").unwrap();
    std::env::set_var("LLVM_CONFIG_PATH", "/usr/local/opt/llvm/bin/llvm-config");

    #[cfg(regenerate_bindings)]
    bindgen::Builder::default()
        .header("portaudio/include/portaudio.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustified_non_exhaustive_enum("PaHostApiTypeId|PaErrorCode|PaStreamCallbackResult")
        .new_type_alias("PaStream")
        .blacklist_type("PaStreamCallbackFlags|PaStreamFlags|PaSampleFormat|PaError")
        .raw_line("use crate::{PaSampleFormat, PaStreamFlags, PaStreamCallbackFlags};")
        .raw_line("pub type PaError = PaErrorCode;")
        .derive_copy(false)
        .derive_debug(false)
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings.")
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings to file.");

    // Actually build.
    let dst = cmake::Config::new("portaudio")
        .define("PA_BUILD_SHARED", "OFF")
        .build();
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=portaudio");

    // OSX
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreServices");
    }
}
