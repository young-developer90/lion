use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Lion RS - Quick runner for Lion files");
        eprintln!("Usage: lion-rs <file> [args...]");
        std::process::exit(1);
    }

    let file = &args[1];
    let rest: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    // Find the lion binary (same directory as lion-rs)
    let exe = std::env::current_exe().unwrap_or_default();
    let dir = exe.parent().unwrap_or(std::path::Path::new("."));
    let lion_exe = if cfg!(windows) {
        dir.join("lion.exe")
    } else {
        dir.join("lion")
    };

    let status = Command::new(&lion_exe)
        .arg("run")
        .arg(file)
        .args(&rest)
        .status()
        .unwrap_or_else(|e| {
            // Fallback: try lion in PATH
            Command::new("lion")
                .arg("run")
                .arg(file)
                .args(&rest)
                .status()
                .unwrap_or_else(|_| {
                    eprintln!("error: cannot find lion binary ({})", e);
                    std::process::exit(1);
                })
        });

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}
