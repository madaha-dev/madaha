// madaha/build.rs
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("sample_speed_ratio_table.rs");

    let mut code = String::from("pub const TABLE: [[f64; 128]; 128] = [\n");
    for sample in 0..128 {
        code.push_str("    [");
        for note in 0..128 {
            let val = note as f64 / sample.max(1) as f64;
            code.push_str(&format!("{:.6},", val));
        }
        code.push_str("],\n");
    }
    code.push_str("];\n");

    fs::write(dest_path, code).unwrap();
}
