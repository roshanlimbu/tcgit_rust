use dialoguer::{Confirm, Select};
use std::process::Command;

fn run_command(command: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute '{}': {}", command, e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn generate_commit_message() -> Result<String, String> {
    // Check for staged changes
    let status = run_command("git diff --cached --name-only")?;
    if status.is_empty() {
        return Err("No staged changes found".to_string());
    }

    // Get the raw suggestion command
    let suggestion = run_command(
        r#"gh copilot suggest -t git "Suggest a git commit message based on staged changes" --shell-out"#,
    )?;
    if suggestion.is_empty() {
        return Err("No suggestion provided by gh copilot".to_string());
    }

    // Execute the suggested command to get the message
    run_command(&suggestion)
}

fn main() {
    println!("Welcome to the Git TUI App!");

    loop {
        let options = vec!["Generate Commit and Push", "Exit"];
        let choice = Select::new()
            .with_prompt("What would you like to do?")
            .items(&options)
            .interact()
            .unwrap();

        match options[choice] {
            "Exit" => {
                println!("Goodbye!");
                return;
            }
            "Generate Commit and Push" => {
                // Check for changes
                match run_command("git status --porcelain") {
                    Ok(status) if status.is_empty() => {
                        println!("No changes to commit.");
                        continue;
                    }
                    Err(e) => {
                        println!("Git status error: {}", e);
                        continue;
                    }
                    _ => {}
                }

                // Stage changes
                if let Err(e) = run_command("git add .") {
                    println!("Failed to stage changes: {}", e);
                    continue;
                }
                println!("Changes staged.");

                // Generate commit message
                let msg = match generate_commit_message() {
                    Ok(msg) => msg,
                    Err(e) => {
                        println!("Failed to generate commit message: {}", e);
                        continue;
                    }
                };
                println!("Suggested commit message: {:?}", msg);

                if !Confirm::new()
                    .with_prompt("Use this commit message?")
                    .default(true)
                    .interact()
                    .unwrap()
                {
                    let custom_msg = dialoguer::Input::<String>::new()
                        .with_prompt("Enter custom commit message")
                        .interact()
                        .unwrap();
                    if let Err(e) = run_command(&format!("git commit -m {:?}", custom_msg)) {
                        println!("Commit failed: {}", e);
                        continue;
                    }
                }

                if Confirm::new()
                    .with_prompt("Use this commit message?")
                    .default(true)
                    .interact()
                    .unwrap()
                {
                    // Commit
                    if let Err(e) = run_command(&format!("git commit -m {:?}", msg)) {
                        println!("Commit failed: {}", e);
                        continue;
                    }
                    println!("Changes committed.");

                    // Push
                    if let Err(e) = run_command("git push origin master") {
                        println!("Push failed: {}", e);
                        continue;
                    }
                    println!("Pushed to master successfully!");
                } else {
                    println!("Commit cancelled.");
                }
            }
            _ => unreachable!(),
        }
    }
}
