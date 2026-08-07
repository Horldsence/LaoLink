fn main() {
    // This call will make config entries available in the code for every device tree node, to
    // allow conditional compilation based on whether it is present in the device tree.
    // For example, it will be possible to have:
    // ```rust
    // #[cfg(dt = "aliases::led0")]
    // ```
    zephyr_build::dt_cfgs();

    // Link WCH's precompiled UHSIF (universal high-speed interface) driver.
    // Its missing dependencies (a few SPL functions, millicode, line
    // callbacks) are provided by src/shims.rs.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search=native={manifest_dir}/lib");
    println!("cargo:rustc-link-lib=static:+whole-archive=UHSIF");
}
