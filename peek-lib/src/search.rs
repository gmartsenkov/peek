use std::io::Write;
use std::process::{Command, Output};

pub fn fzf(prompt: String, inner_command: &mut Command) -> Output {
    let mut command = std::process::Command::new("fzf")
        .arg("--filter")
        .arg(&prompt)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let output = inner_command.output().unwrap();
    let mut stdin = command.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin.write_all(&output.stdout).expect("Failed to write to stdin");
    });

    command.wait_with_output().unwrap()
}
