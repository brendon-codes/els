use std::io::Write;
use std::process::{Command, Stdio};

pub fn paged_display(output: &str) {
    let formatted = format!("\n{}\n\n", output);

    // @TODO: Replace unix `less` with a Rust lib solution
    let mut child = match Command::new("less")
        .args([
            "--RAW-CONTROL-CHARS",
            "--quit-at-eof",
            "--quit-if-one-screen",
            "--no-init",
        ])
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            print!("{}", formatted);
            return;
        }
    };

    if let Some(ref mut stdin) = child.stdin {
        let _ = stdin.write_all(formatted.as_bytes());
    }

    let _ = child.wait();
}

pub fn display(rows_str: &str) {
    paged_display(rows_str);
}
