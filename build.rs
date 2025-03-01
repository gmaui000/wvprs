use regex::Regex;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

fn main() -> io::Result<()> {
    replace_version_in_rs()?;
    Ok(())
}

fn replace_version_in_rs() -> io::Result<()> {
    let latest_version = format!(
        "{}.{}",
        get_latest_git_commit_hash(true)?,
        get_latest_git_commit_time()?
    );

    // Replace version string in.rs files
    let version_regex =
        Regex::new(r#"pub static APP_VERSION: &str = "([0-9a-f]{7})\.(\d{8})\.(\d{6})";"#).unwrap();
    let version_replacement = format!(r#"pub static APP_VERSION: &str = "{}";"#, latest_version);
    let file = String::from("./src/version.rs");
    if !Path::new(&file).exists() {
        let mut text = String::from(r#"pub static APP_NAME: &str = "wvprs";"#);
        text += "\n";
        text += &version_replacement;
        text += "\n";
        fs::write(&file, text)?;
    }
    println!("file: {}", &file);
    let original_content = fs::read_to_string(&file)?;
    let replaced_content = version_regex.replace_all(&original_content, &version_replacement);
    if original_content != replaced_content {
        println!(
            "std::fs::write, file: {}, version: {}",
            &file, &latest_version
        );
        fs::write(&file, replaced_content.as_ref())?;
    }
    Ok(())
}

fn get_latest_git_commit_hash(short: bool) -> io::Result<String> {
    // Run Git command to get the latest commit hash
    let output = Command::new("git")
        .args([
            "log",
            "-1",
            if short {
                "--pretty=format:%h"
            } else {
                "--pretty=format:%H"
            },
        ])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_latest_git_commit_time() -> io::Result<String> {
    // Run Git command to get the latest commit hash
    let output = Command::new("git")
        .args(["log", "-1", "--format=%ad", "--date=format:%Y%m%d.%H%M%S"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
