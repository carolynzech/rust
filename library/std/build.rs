use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH was not set");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS was not set");
    let target_vendor =
        env::var("CARGO_CFG_TARGET_VENDOR").expect("CARGO_CFG_TARGET_VENDOR was not set");
    let target_env = env::var("CARGO_CFG_TARGET_ENV").expect("CARGO_CFG_TARGET_ENV was not set");
    let target_pointer_width: u32 = env::var("CARGO_CFG_TARGET_POINTER_WIDTH")
        .expect("CARGO_CFG_TARGET_POINTER_WIDTH was not set")
        .parse()
        .unwrap();

    println!("cargo:rustc-check-cfg=cfg(netbsd10)");
    if target_os == "netbsd" && env::var("RUSTC_STD_NETBSD10").is_ok() {
        println!("cargo:rustc-cfg=netbsd10");
    }

    println!("cargo:rustc-check-cfg=cfg(restricted_std)");
    if target_os == "linux"
        || target_os == "android"
        || target_os == "netbsd"
        || target_os == "dragonfly"
        || target_os == "openbsd"
        || target_os == "freebsd"
        || target_os == "solaris"
        || target_os == "illumos"
        || target_os == "macos"
        || target_os == "ios"
        || target_os == "tvos"
        || target_os == "watchos"
        || target_os == "visionos"
        || target_os == "windows"
        || target_os == "fuchsia"
        || (target_vendor == "fortanix" && target_env == "sgx")
        || target_os == "hermit"
        || target_os == "l4re"
        || target_os == "redox"
        || target_os == "haiku"
        || target_os == "vxworks"
        || target_arch == "wasm32"
        || target_arch == "wasm64"
        || target_os == "espidf"
        || target_os.starts_with("solid")
        || (target_vendor == "nintendo" && target_env == "newlib")
        || target_os == "vita"
        || target_os == "aix"
        || target_os == "nto"
        || target_os == "xous"
        || target_os == "hurd"
        || target_os == "uefi"
        || target_os == "teeos"
        || target_os == "zkvm"

        // See src/bootstrap/src/core/build_steps/synthetic_targets.rs
        || env::var("RUSTC_BOOTSTRAP_SYNTHETIC_TARGET").is_ok()
    {
        // These platforms don't have any special requirements.
    } else {
        // This is for Cargo's build-std support, to mark std as unstable for
        // typically no_std platforms.
        // This covers:
        // - os=none ("bare metal" targets)
        // - mipsel-sony-psp
        // - nvptx64-nvidia-cuda
        // - arch=avr
        // - JSON targets
        // - Any new targets that have not been explicitly added above.
        println!("cargo:rustc-cfg=restricted_std");
    }

    println!("cargo:rustc-check-cfg=cfg(backtrace_in_libstd)");
    println!("cargo:rustc-cfg=backtrace_in_libstd");

    println!("cargo:rustc-env=STD_ENV_ARCH={}", env::var("CARGO_CFG_TARGET_ARCH").unwrap());

    // Emit these on platforms that have no known ABI bugs, LLVM selection bugs, lowering bugs,
    // missing symbols, or other problems, to determine when tests get run.
    // If more broken platforms are found, please update the tracking issue at
    // <https://github.com/rust-lang/rust/issues/116909>
    //
    // Some of these match arms are redundant; the goal is to separate reasons that the type is
    // unreliable, even when multiple reasons might fail the same platform.
    println!("cargo:rustc-check-cfg=cfg(reliable_f16)");
    println!("cargo:rustc-check-cfg=cfg(reliable_f128)");

    let has_reliable_f16 = match (target_arch.as_str(), target_os.as_str()) {
        // Selection failure until recent LLVM <https://github.com/llvm/llvm-project/issues/93894>
        // FIXME(llvm19): can probably be removed at the version bump
        ("loongarch64", _) => false,
        // Selection failure <https://github.com/llvm/llvm-project/issues/50374>
        ("s390x", _) => false,
        // Unsupported <https://github.com/llvm/llvm-project/issues/94434>
        ("arm64ec", _) => false,
        // MinGW ABI bugs <https://gcc.gnu.org/bugzilla/show_bug.cgi?id=115054>
        ("x86_64", "windows") => false,
        // x86 has ABI bugs that show up with optimizations. This should be partially fixed with
        // the compiler-builtins update. <https://github.com/rust-lang/rust/issues/123885>
        ("x86" | "x86_64", _) => false,
        // Missing `__gnu_h2f_ieee` and `__gnu_f2h_ieee`
        ("powerpc" | "powerpc64", _) => false,
        // Missing `__gnu_h2f_ieee` and `__gnu_f2h_ieee`
        ("mips" | "mips32r6" | "mips64" | "mips64r6", _) => false,
        // Missing `__extendhfsf` and `__truncsfhf`
        ("riscv32" | "riscv64", _) => false,
        // Most OSs are missing `__extendhfsf` and `__truncsfhf`
        (_, "linux" | "macos") => true,
        // Almost all OSs besides Linux and MacOS are missing symbols until compiler-builtins can
        // be updated. <https://github.com/rust-lang/rust/pull/125016> will get some of these, the
        // next CB update should get the rest.
        _ => false,
    };

    let has_reliable_f128 = match (target_arch.as_str(), target_os.as_str()) {
        // Unsupported <https://github.com/llvm/llvm-project/issues/94434>
        ("arm64ec", _) => false,
        // ABI and precision bugs <https://github.com/rust-lang/rust/issues/125109>
        // <https://github.com/rust-lang/rust/issues/125102>
        ("powerpc" | "powerpc64", _) => false,
        // Selection bug <https://github.com/llvm/llvm-project/issues/95471>
        ("nvptx64", _) => false,
        // ABI unsupported  <https://github.com/llvm/llvm-project/issues/41838>
        ("sparc", _) => false,
        // MinGW ABI bugs <https://gcc.gnu.org/bugzilla/show_bug.cgi?id=115054>
        ("x86_64", "windows") => false,
        // 64-bit Linux is about the only platform to have f128 symbols by default
        (_, "linux") if target_pointer_width == 64 => true,
        // Same as for f16, except MacOS is also missing f128 symbols.
        _ => false,
    };

    if has_reliable_f16 {
        println!("cargo:rustc-cfg=reliable_f16");
    }
    if has_reliable_f128 {
        println!("cargo:rustc-cfg=reliable_f128");
    }
}
