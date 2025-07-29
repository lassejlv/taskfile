use colored::*;
use env_parser::{EnvConfig, EnvParser};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
pub struct TaskFile {
    pub tasks: HashMap<String, Task>,
    pub env: Option<EnvConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Task {
    pub cmd: String,
    pub desc: Option<String>,
    pub depends_on: Option<Vec<String>>,
}

pub struct TaskRunner {
    taskfile: TaskFile,
    env_parser: EnvParser,
    enhanced_path: Option<String>,
}

impl TaskRunner {
    pub async fn from_file(taskfile_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = Self::read_taskfile(taskfile_path).await?;
        let taskfile = Self::parse_taskfile(&contents)?;

        let env_parser = if let Some(env_config) = &taskfile.env {
            let parser = EnvParser::with_config(env_config.clone());
            let taskfile_dir = std::path::Path::new(taskfile_path).parent();
            parser.load_env_files_with_base_path(taskfile_dir)?;
            parser
        } else {
            EnvParser::new()
        };

        let enhanced_path = Self::setup_enhanced_path().await;

        Ok(Self {
            taskfile,
            env_parser,
            enhanced_path,
        })
    }

    pub fn new(taskfile: TaskFile) -> Self {
        Self::new_with_base_path(taskfile, None)
    }

    pub fn new_with_base_path(taskfile: TaskFile, base_path: Option<&std::path::Path>) -> Self {
        let env_parser = if let Some(env_config) = &taskfile.env {
            let parser = EnvParser::with_config(env_config.clone());
            if let Err(e) = parser.load_env_files_with_base_path(base_path) {
                eprintln!("{} Error loading environment files: {}", "✗".red(), e);
            }
            parser
        } else {
            EnvParser::new()
        };

        let enhanced_path = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::setup_enhanced_path())
        });

        Self {
            taskfile,
            env_parser,
            enhanced_path,
        }
    }

    async fn read_taskfile(taskfile_name: &str) -> Result<String, std::io::Error> {
        let mut file = tokio::fs::File::open(taskfile_name).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        Ok(contents)
    }

    fn parse_taskfile(contents: &str) -> Result<TaskFile, toml::de::Error> {
        let taskfile: TaskFile = toml::from_str(contents)?;
        Ok(taskfile)
    }

    async fn check_npm_script(script_name: &str) -> Option<String> {
        if let Ok(contents) = tokio::fs::read_to_string("package.json").await {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(scripts) = package_json.get("scripts") {
                    if let Some(script) = scripts.get(script_name) {
                        if let Some(script_cmd) = script.as_str() {
                            return Some(script_cmd.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    async fn setup_enhanced_path() -> Option<String> {
        if tokio::fs::try_exists("package.json").await.unwrap_or(false) {
            let mut enhanced_path = String::new();

            let node_modules_bin = std::path::Path::new("node_modules/.bin");
            if node_modules_bin.exists() {
                if let Ok(canonical) = node_modules_bin.canonicalize() {
                    enhanced_path.push_str(&canonical.to_string_lossy());
                }
            }

            if let Ok(current_path) = env::var("PATH") {
                if !enhanced_path.is_empty() {
                    enhanced_path.push(':');
                }
                enhanced_path.push_str(&current_path);
            }

            if !enhanced_path.is_empty() {
                return Some(enhanced_path);
            }
        }
        None
    }

    pub fn list_tasks(&self) {
        if self.taskfile.tasks.is_empty() {
            println!("No tasks found in Taskfile.");
            return;
        }

        let max_name_len = self
            .taskfile
            .tasks
            .keys()
            .map(|k| k.len())
            .max()
            .unwrap_or(0);
        let max_desc_len = self
            .taskfile
            .tasks
            .values()
            .map(|t| t.desc.as_deref().unwrap_or("No description").len())
            .max()
            .unwrap_or(0);
        let max_deps_len = self
            .taskfile
            .tasks
            .values()
            .map(|t| {
                t.depends_on
                    .as_ref()
                    .map(|deps| deps.join(", ").len())
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0);

        let name_width = (max_name_len + 2).max(6);
        let desc_width = (max_desc_len + 2).max(13);
        let deps_width = (max_deps_len + 2).max(12);

        println!(
            "┌{:─<name_width$}┬{:─<desc_width$}┬{:─<deps_width$}┐",
            "",
            "",
            "",
            name_width = name_width,
            desc_width = desc_width,
            deps_width = deps_width
        );
        println!(
            "│ {:^name_width$} │ {:^desc_width$} │ {:^deps_width$} │",
            "Task",
            "Description",
            "Dependencies",
            name_width = name_width - 2,
            desc_width = desc_width - 2,
            deps_width = deps_width - 2
        );
        println!(
            "├{:─<name_width$}┼{:─<desc_width$}┼{:─<deps_width$}┤",
            "",
            "",
            "",
            name_width = name_width,
            desc_width = desc_width,
            deps_width = deps_width
        );

        let mut tasks: Vec<_> = self.taskfile.tasks.iter().collect();
        tasks.sort_by(|a, b| a.0.cmp(b.0));

        for (name, task) in tasks {
            let desc = task.desc.as_deref().unwrap_or("No description");
            let deps = task
                .depends_on
                .as_ref()
                .map(|d| d.join(", "))
                .unwrap_or_else(|| "-".to_string());

            println!(
                "│ {:name_width$} │ {:desc_width$} │ {:deps_width$} │",
                name,
                desc,
                deps,
                name_width = name_width - 2,
                desc_width = desc_width - 2,
                deps_width = deps_width - 2
            );
        }

        println!(
            "└{:─<name_width$}┴{:─<desc_width$}┴{:─<deps_width$}┘",
            "",
            "",
            "",
            name_width = name_width,
            desc_width = desc_width,
            deps_width = deps_width
        );
    }

    pub async fn run_task(&self, task_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_task_with_deps(task_name, &mut Vec::new()).await
    }

    fn run_task_with_deps<'a>(
        &'a self,
        task_name: &'a str,
        visited: &'a mut Vec<String>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + 'a>,
    > {
        Box::pin(async move {
            if visited.contains(&task_name.to_string()) {
                return Err(
                    format!("Circular dependency detected for task '{}'", task_name).into(),
                );
            }

            if let Some(task) = self.taskfile.tasks.get(task_name) {
                if let Some(deps) = &task.depends_on {
                    for dep in deps {
                        if !self.has_task(dep) {
                            return Err(format!(
                                "Dependency '{}' not found for task '{}'",
                                dep, task_name
                            )
                            .into());
                        }

                        visited.push(task_name.to_string());
                        self.run_task_with_deps(dep, visited).await?;
                        visited.pop();
                    }
                }

                let substituted_cmd = self.env_parser.substitute_env_vars(&task.cmd);

                let parts: Vec<&str> = substituted_cmd.split_whitespace().collect();
                if parts.is_empty() {
                    return Err(format!("Empty command for task '{}'", task_name).into());
                }

                let (command, args): (String, Vec<&str>) =
                    if let Some(_npm_script) = Self::check_npm_script(parts[0]).await {
                        // If the first part is an npm script, run it with npm/yarn
                        let package_manager =
                            if tokio::fs::try_exists("yarn.lock").await.unwrap_or(false) {
                                "yarn"
                            } else if tokio::fs::try_exists("pnpm-lock.yaml")
                                .await
                                .unwrap_or(false)
                            {
                                "pnpm"
                            } else {
                                "npm"
                            };

                        let mut npm_args = vec!["run", parts[0]];
                        npm_args.extend_from_slice(&parts[1..]);
                        (package_manager.to_string(), npm_args)
                    } else {
                        let node_bin_path = format!("node_modules/.bin/{}", parts[0]);
                        if tokio::fs::try_exists(&node_bin_path).await.unwrap_or(false) {
                            (node_bin_path, parts[1..].to_vec())
                        } else {
                            (parts[0].to_string(), parts[1..].to_vec())
                        }
                    };

                let pb = ProgressBar::new_spinner();
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                        .template("{spinner:.cyan} {msg} [{elapsed_precise}]")
                        .unwrap(),
                );
                pb.set_message(format!("Running task '{}': {}", task_name, substituted_cmd));
                pb.enable_steady_tick(Duration::from_millis(80));

                let start_time = Instant::now();

                let mut cmd = Command::new(&command);
                cmd.args(&args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                if let Some(enhanced_path) = &self.enhanced_path {
                    cmd.env("PATH", enhanced_path);
                }

                let child = cmd.spawn()?;

                let pb_clone = pb.clone();
                let task_name_clone = task_name.to_string();
                let cmd_clone = substituted_cmd.clone();
                let spinner_task = tokio::spawn(async move {
                    let start = Instant::now();
                    loop {
                        let elapsed = start.elapsed();
                        pb_clone.set_message(format!(
                            "Running task '{}': {} [{}]",
                            task_name_clone,
                            cmd_clone,
                            format_duration(elapsed)
                        ));
                        sleep(Duration::from_millis(100)).await;
                    }
                });

                // Wait for the process to complete
                let output = child.wait_with_output().await?;
                let elapsed = start_time.elapsed();

                spinner_task.abort();
                pb.finish_and_clear();

                if !output.stdout.is_empty() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                }

                if !output.stderr.is_empty() {
                    eprint!("{}", String::from_utf8_lossy(&output.stderr));
                }

                if output.status.success() {
                    println!(
                        "{} Task '{}' completed successfully in {}",
                        "✓".green(),
                        task_name,
                        format_duration(elapsed).green()
                    );
                    Ok(())
                } else {
                    let code = output.status.code().unwrap_or(-1);
                    eprintln!(
                        "{} Task '{}' failed with exit code {} after {}",
                        "✗".red(),
                        task_name,
                        code,
                        format_duration(elapsed).red()
                    );
                    Err(format!("Task '{}' failed with exit code {}", task_name, code).into())
                }
            } else {
                Err(format!("Task '{}' not found in Taskfile", task_name).into())
            }
        })
    }

    pub fn has_task(&self, task_name: &str) -> bool {
        self.taskfile.tasks.contains_key(task_name)
    }

    pub fn get_task_names(&self) -> Vec<&String> {
        self.taskfile.tasks.keys().collect()
    }

    pub fn get_task(&self, task_name: &str) -> Option<&Task> {
        self.taskfile.tasks.get(task_name)
    }

    pub fn task_count(&self) -> usize {
        self.taskfile.tasks.len()
    }
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if total_secs >= 60 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        if millis > 0 {
            format!("{}m {}s {}ms", mins, secs, millis)
        } else {
            format!("{}m {}s", mins, secs)
        }
    } else if total_secs > 0 {
        if millis > 0 {
            format!("{}.{}s", total_secs, millis / 100)
        } else {
            format!("{}s", total_secs)
        }
    } else {
        format!("{}ms", millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[tokio::test]
    async fn test_task_runner_creation() {
        let toml_content = r#"
[tasks.test]
cmd = "echo 'test'"
desc = "Test task"
"#;

        let mut file = fs::File::create("test_taskfile.toml").unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let runner = TaskRunner::from_file("test_taskfile.toml").await;
        assert!(runner.is_ok());

        let runner = runner.unwrap();
        assert!(runner.has_task("test"));
        assert_eq!(runner.task_count(), 1);

        fs::remove_file("test_taskfile.toml").unwrap();
    }

    #[test]
    fn test_task_operations() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "test".to_string(),
            Task {
                cmd: "echo 'hello'".to_string(),
                desc: Some("Test description".to_string()),
                depends_on: None,
            },
        );

        let taskfile = TaskFile { tasks, env: None };
        let runner = TaskRunner::new(taskfile);

        assert!(runner.has_task("test"));
        assert!(!runner.has_task("nonexistent"));
        assert_eq!(runner.task_count(), 1);

        let task = runner.get_task("test");
        assert!(task.is_some());
        assert_eq!(task.unwrap().cmd, "echo 'hello'");
    }
}
