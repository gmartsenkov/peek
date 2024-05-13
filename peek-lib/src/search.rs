use std::io::Write;

pub fn fzf(prompt: String, stdout: Vec<u8>) -> Vec<String> {
    let mut command = std::process::Command::new("fzf")
        .arg("--exact")
        .arg("--filter")
        .arg(&prompt)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = command.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin.write_all(&stdout).expect("Failed to write to stdin");
    });

    let output = command.wait_with_output().unwrap().stdout;
    std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

#[allow(clippy::ptr_arg)]
pub fn contains(tokens: &Vec<&str>, line: &str) -> bool {
    tokens.iter().any(|token| line.contains(token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_containts() {
        assert!(contains(&vec!("rom", "repo"), "lib/rom/repo/jon.rb"));
        assert!(!contains(&vec!("rom"), "lib/repo/jon.rb"));
        assert!(contains(&vec!("rom"), "rom"));
    }
}
