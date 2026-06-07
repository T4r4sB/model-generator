use std::env;
use std::fs;
use std::path::Path;
use std::fmt::Write;

// duplicate useful function 
fn a2t(a: f32) -> f32 {
    a.tan() - a
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("constants.rs");
    let a0 = 20.0f32.to_radians();
    let (sin0, cos0) = a0.sin_cos();
    let tan0 = a0.tan();
    let t0 = a2t(a0);

    let mut buf = String::new();

    writeln!(&mut buf, "const SIN0: f32 = {sin0};").unwrap();
    writeln!(&mut buf, "const COS0: f32 = {cos0};").unwrap();
    writeln!(&mut buf, "const TAN0: f32 = {tan0};").unwrap();
    writeln!(&mut buf, "const T0: f32 = {t0};").unwrap();
    
    println!("cargo:rerun-if-changed=build.rs");

    fs::write(&dest_path, buf).unwrap();
}
