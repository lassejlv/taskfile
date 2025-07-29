use clap::{Arg, Command};
use colored::*;
use taskfile_runner::TaskRunner;

#[tokio::main]
async fn main() {
    let matches = Command::new("taskfile")
        .version("1.0")
        .about("A simple task runner")
        .arg(
            Arg::new("command")
                .help("The command to run (list, version, update, or task name)")
                .value_name("COMMAND")
                .index(1),
        )
        .get_matches();

    let taskfile_name = "Taskfile.toml";

    match tokio::fs::try_exists(taskfile_name).await {
        Ok(true) => match TaskRunner::from_file(taskfile_name).await {
            Ok(runner) => match matches.get_one::<String>("command") {
                Some(cmd) if cmd == "list" => {
                    runner.list_tasks();
                }
                Some(cmd) if cmd == "version" => {
                    println!("taskfile-runner v{}", env!("CARGO_PKG_VERSION"));
                    println!("A simple task runner written in Rust");
                }
                Some(cmd) if cmd == "update" => {
                    println!("Updating task runner...");
                    match update_task_runner().await {
                        Ok(_) => println!("✓ Update completed successfully!"),
                        Err(e) => {
                            eprintln!("{} Update failed: {}", "✗".red(), e);
                            std::process::exit(1);
                        }
                    }
                }
                Some(task_name) => {
                    if let Err(e) = runner.run_task(task_name).await {
                        eprintln!("{} Error running task '{}': {}", "✗".red(), task_name, e);
                        std::process::exit(1);
                    }
                }
                None => {
                    println!("Please specify a task to run or use 'list' to see available tasks.");
                    println!("Usage: task <task_name> | list | version | update");
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("{} Error loading taskfile: {}", "✗".red(), e);
                std::process::exit(1);
            }
        },
        Ok(false) => {
            eprintln!("{} Taskfile '{}' does not exist", "✗".red(), taskfile_name);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("{} Error checking taskfile existence: {}", "✗".red(), e);
            std::process::exit(1);
        }
    }
}

async fn update_task_runner() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::process::Stdio;
    use tokio::process::Command;

    let current_exe = env::current_exe()?;
    let install_dir = current_exe.parent().unwrap();

    println!("Downloading latest version...");

    let output = Command::new("curl")
        .args(&[
            "-sSL",
            "https://raw.githubusercontent.com/lassejlv/taskfile/main/install.sh",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        return Err("Failed to download install script".into());
    }

    let install_script = String::from_utf8(output.stdout)?;

    let output = Command::new("bash")
        .arg("-c")
        .arg(&format!(
            "echo '{}' | bash -s -- --install-dir '{}'",
            install_script.replace("'", "'\"'\"'"),
            install_dir.display()
        ))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if output.status.success() {
        println!("Installation output:");
        print!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    } else {
        eprintln!("Error output:");
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        Err("Update installation failed".into())
    }
}
