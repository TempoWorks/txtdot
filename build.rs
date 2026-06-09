use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use css_minify::optimizations::{Level, Minifier};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let static_out = out_dir.join("static");
    let static_src = Path::new("static");

    println!("cargo:rerun-if-changed=static");

    if static_out.exists() {
        fs::remove_dir_all(&static_out).expect("remove stale static output");
    }

    copy_static(static_src, &static_out).expect("copy static files");
    println!(
        "cargo:rustc-env=TXTDOT_BUILD_STATIC_DIR={}",
        static_out.display()
    );
}

fn copy_static(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        let target = dest.join(entry.file_name());

        if file_type.is_dir() {
            copy_static(&path, &target)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("css") {
            let css = fs::read_to_string(&path)?;
            let minified = Minifier::default()
                .minify(&css, Level::Three)
                .unwrap_or_else(|_| css.clone());
            fs::write(target, minified)?;
        } else {
            fs::copy(path, target)?;
        }
    }

    Ok(())
}
