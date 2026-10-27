use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Modern Baseline browser targets aligned with Tailwind CSS v4.
const CSS_TARGETS: &str = "chrome >= 111, edge >= 111, firefox >= 115, safari >= 16.4";

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let root = Path::new(&manifest_dir);

    for rel in ["frontend", "templates", "src"] {
        let p = root.join(rel);
        if p.exists() {
            println!("cargo:rerun-if-changed={}", p.display());
        }
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set by cargo"));

    // Stage `frontend/styles` into `OUT_DIR/styles` so `shelf.css` acts as the
    // single source of truth while keeping the source tree strictly read-only
    // (required inside Nix sandbox builds).
    let src_styles = root.join("frontend/styles");
    let staged_styles = out_dir.join("styles");
    stage_dir(&src_styles, &staged_styles);

    // 1. Generate Tailwind CSS v4 utilities into the staged styles directory.
    let tailwind_output = staged_styles.join("tailwind.generated.css");
    run_tool(
        "tailwindcss",
        root,
        &[
            "-i",
            "frontend/styles/tailwind-input.css",
            "-o",
            path_str(&tailwind_output),
        ],
    );

    // 2. Bundle, lower, and minify core shelf CSS from `shelf.css` via `lightningcss` CLI.
    let shelf_entry = staged_styles.join("shelf.css");
    let shelf_output = out_dir.join("shelf.min.css");
    run_tool(
        "lightningcss",
        root,
        &[
            "--bundle",
            "--minify",
            "--targets",
            CSS_TARGETS,
            path_str(&shelf_entry),
            "-o",
            path_str(&shelf_output),
        ],
    );

    // 3. Bundle, lower, and minify all Web Component Shadow DOM stylesheets in a single pass.
    let components_src = src_styles.join("components");
    let components_out = out_dir.join("components");
    fs::create_dir_all(&components_out).expect("failed to create OUT_DIR/components");

    let mut component_files: Vec<PathBuf> = fs::read_dir(&components_src)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", components_src.display()))
        .map(|entry| entry.expect("failed to read dir entry").path())
        .filter(|path| path.extension() == Some(OsStr::new("css")))
        .collect();
    component_files.sort();

    if !component_files.is_empty() {
        let mut args = vec![
            "--bundle",
            "--minify",
            "--targets",
            CSS_TARGETS,
            "-d",
            path_str(&components_out),
        ];
        for file in &component_files {
            args.push(path_str(file));
        }
        run_tool("lightningcss", root, &args);
    }

    // 4. Bundle and minify frontend TypeScript with `esbuild`.
    let js_output_path = out_dir.join("shelf.js");
    let outfile_arg = format!("--outfile={}", path_str(&js_output_path));
    run_tool(
        "esbuild",
        root,
        &[
            "frontend/src/index.ts",
            "--bundle",
            "--minify",
            &outfile_arg,
        ],
    );
}

fn path_str(path: &Path) -> &str {
    path.to_str()
        .unwrap_or_else(|| panic!("path is not valid UTF-8: {}", path.display()))
}

fn stage_dir(src: &Path, dst: &Path) {
    if dst.exists() {
        fs::remove_dir_all(dst)
            .unwrap_or_else(|e| panic!("failed to clean {}: {e}", dst.display()));
    }
    copy_dir_recursive(src, dst);
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap_or_else(|e| panic!("failed to create {}: {e}", dst.display()));
    for entry in
        fs::read_dir(src).unwrap_or_else(|e| panic!("failed to read {}: {e}", src.display()))
    {
        let entry = entry.expect("failed to read dir entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry
            .file_type()
            .expect("failed to read file type")
            .is_dir()
        {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).unwrap_or_else(|e| {
                panic!(
                    "failed to copy {} to {}: {e}",
                    src_path.display(),
                    dst_path.display()
                )
            });
        }
    }
}

fn run_tool(program: &str, cwd: &Path, args: &[&str]) {
    let status = Command::new(program).current_dir(cwd).args(args).status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => panic!("`{program}` exited with {s}"),
        Err(e) => panic!(
            "failed to run `{program}` ({e}). It's provided by the nix dev shell \
             (nix/dev.nix) -- run this build from inside `nix develop` / direnv."
        ),
    }
}
