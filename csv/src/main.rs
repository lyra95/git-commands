use std::process::{Command, Output};

use anyhow::{anyhow, Result};

fn main() -> Result<()> {
    let branch = std::env::args()
        .take(2)
        .last()
        .expect("branch name expected");
    get_deltas(&branch)
}

fn get_deltas(branch: &str) -> Result<()> {
    let output = execute_command(&format!("git compare origin/main {}", branch))?;
    let list = parse(&output.stdout)?;

    for (is_cherrypicked, commit_id) in list {
        let output = execute_command(&format!(
            "git meta -f '{}' {}",
            r#""${author}","${jira}","${pr}","${summary}""#, commit_id
        ))?;
        let meta = std::str::from_utf8(&output.stdout)?;
        let line = if cfg!(target_os = "windows") {
            // 윈도우즈는 짜증나게 앞뒤로 '가 있음
            is_cherrypicked.to_string() + "," + &meta[1..meta.len() - 1]
        } else {
            is_cherrypicked.to_string() + "," + meta
        };

        print!("{}", line);
    }

    Ok(())
}

fn execute_command(command: &str) -> Result<Output> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", command]).output()?
    } else {
        Command::new("bash").arg("-c").arg(command).output()?
    };

    if !output.status.success() {
        let message = std::str::from_utf8(&output.stderr)?;
        return Err(anyhow!("command exited with nonzero code:{}", message));
    }

    Ok(output)
}

fn parse(message: &[u8]) -> Result<Vec<(bool, String)>> {
    let message = std::str::from_utf8(message)?;
    let mut result = vec![];
    for line in message.lines() {
        let mut iter = line.split_whitespace();
        let is_cherrypicked = {
            let status = iter.next().expect("unexpected format");
            match status {
                ">" => true,
                "=" => true,
                "<" => false,
                _ => false,
            }
        };
        let commit_id = iter.next().expect("unexpected format").to_string();
        result.push((is_cherrypicked, commit_id))
    }
    Ok(result)
}
