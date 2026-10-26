//! Picks the renderer: smud's own instanced one with the `webgl2` feature on
//! wasm32 (WebGL2 has no storage buffers), Bevy's Mesh2d everywhere else.
//! `native_webgl2_renderer` forces the instanced one, to compare the two.

fn main() {
    println!("cargo::rustc-check-cfg=cfg(smud_webgl2_renderer)");
    let webgl2 = std::env::var_os("CARGO_FEATURE_WEBGL2").is_some()
        && std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32");
    if webgl2 || std::env::var_os("CARGO_FEATURE_NATIVE_WEBGL2_RENDERER").is_some() {
        println!("cargo::rustc-cfg=smud_webgl2_renderer");
    }
}
