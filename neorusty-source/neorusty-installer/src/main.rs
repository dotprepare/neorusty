use std::env;
use std::path::PathBuf;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();
    let version = args.get(1).map(|s| s.as_str()).unwrap_or(VERSION);
    let target_dir = detect_install_dir();
    let platform = detect_platform();

    println!("NeoRusty Installer v{VERSION}");
    println!("  Version:  {version}");
    println!("  Target:   {}", target_dir.display());

    let url = format!(
        "https://github.com/neorusty/neorusty/releases/download/v{version}/neorusty-{version}-{platform}.tar.gz",
    );

    println!("  Download: {url}");

    let archive = tempfile("neorusty-install.tar.gz");
    let extract_dir = tempfile("neorusty-extract");

    // Download
    let status = Command::new("/usr/bin/curl")
        .args(["-sSfL", "-o", &archive, &url])
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Error: curl not found ({e}). Install curl or download manually from:");
            eprintln!("  {url}");
            std::process::exit(1);
        });

    if !status.success() {
        eprintln!("Error: failed to download from {url}");
        eprintln!("Make sure the release v{version} exists on GitHub.");
        eprintln!("Or build from source: cargo build --release && cp target/release/neorusty {}/neorusty",
            target_dir.display());
        std::process::exit(1);
    }

    // Extract
    let status = Command::new("/usr/bin/tar")
        .args(["-xzf", &archive, "-C", &extract_dir])
        .status()
        .expect("tar not found");

    if !status.success() {
        eprintln!("Error: failed to extract archive");
        std::process::exit(1);
    }

    // Copy binary
    let bin_src = PathBuf::from(&extract_dir).join("neorusty");
    let bin_dst = target_dir.join("neorusty");

    if cfg!(windows) {
        std::fs::rename(&bin_src, &bin_dst).unwrap_or_else(|e| {
            eprintln!("Error: failed to install binary: {e}");
            std::process::exit(1);
        });
    } else {
        std::fs::copy(&bin_src, &bin_dst).unwrap_or_else(|e| {
            eprintln!("Error: failed to install binary: {e}");
            std::process::exit(1);
        });
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin_dst, std::fs::Permissions::from_mode(0o755))
                .unwrap_or_else(|e| eprintln!("Warning: could not set executable bit: {e}"));
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(&archive);
    let _ = std::fs::remove_dir_all(&extract_dir);

    add_to_path(&target_dir);

    println!();
    println!("NeoRusty v{version} installed successfully!");
    println!("  Location: {}", bin_dst.display());
    println!();
    println!("Run `neorusty run` to start the server.");
}

fn tempfile(name: &str) -> String {
    let dir = env::temp_dir();
    let path = dir.join(format!("{}-{}", name, std::process::id()));
    let s = path.to_string_lossy().to_string();
    let _ = std::fs::create_dir_all(&dir);
    s
}

fn detect_install_dir() -> PathBuf {
    if let Ok(home) = env::var("CARGO_HOME") {
        let dir = PathBuf::from(home).join("bin");
        if dir.exists() {
            return dir;
        }
    }
    if let Ok(home) = env::var("HOME") {
        let dir = PathBuf::from(home).join(".cargo").join("bin");
        if dir.exists() || std::fs::create_dir_all(&dir).is_ok() {
            return dir;
        }
    }
    let dir = PathBuf::from("/usr/local/bin");
    if dir.exists() {
        return dir;
    }
    eprintln!("Warning: could not find install directory, using current dir");
    PathBuf::from(".")
}

fn detect_platform() -> String {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };
    format!("{os}-{arch}")
}

fn add_to_path(dir: &std::path::Path) {
    let dir_str = dir.to_string_lossy();
    let shell = env::var("SHELL").unwrap_or_default();

    let rc_file = if shell.ends_with("zsh") {
        Some(PathBuf::from(env::var("HOME").unwrap_or_default()).join(".zshrc"))
    } else if shell.ends_with("bash") {
        Some(PathBuf::from(env::var("HOME").unwrap_or_default()).join(".bashrc"))
    } else {
        None
    };

    if let Some(rc) = &rc_file {
        let export = format!("export PATH=\"{dir_str}:$PATH\"");
        if !std::fs::read_to_string(rc)
            .unwrap_or_default()
            .contains(&export)
        {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(rc)
            {
                let _ = writeln!(file, "\n{export}");
                println!("  Added PATH to {rc:?}");
            }
        }
    }
}
