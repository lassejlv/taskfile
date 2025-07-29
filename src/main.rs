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
                .help("The command to run (list, version, or task name)")
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
                Some(task_name) => {
                    if let Err(e) = runner.run_task(task_name).await {
                        eprintln!("{} Error running task '{}': {}", "✗".red(), task_name, e);
                        std::process::exit(1);
                    }
                }
                None => {
                    println!("Please specify a task to run or use 'list' to see available tasks.");
                    println!("Usage: task <task_name> | list | version");
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
