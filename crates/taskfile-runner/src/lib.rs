use colored::*;
use env_parser::{EnvConfig, EnvParser};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
pub struct TaskFile {
    pub tasks: HashMap<String, Task>,
    pub env: Option<EnvConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Task {
    pub cmd: String,
    pub desc: Option<String>,
}

pub struct TaskRunner {
    taskfile: TaskFile,
    env_parser: EnvParser,
}

impl TaskRunner {
    pub async fn from_file(taskfile_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = Self::read_taskfile(taskfile_path).await?;
        let taskfile = Self::parse_taskfile(&contents)?;

        let env_parser = if let Some(env_config) = &taskfile.env {
            let parser = EnvParser::with_config(env_config.clone());
            parser.load_env_files()?;
            parser
        } else {
            EnvParser::new()
        };

        Ok(Self {
            taskfile,
            env_parser,
        })
    }

    pub fn new(taskfile: TaskFile) -> Self {
        let env_parser = if let Some(env_config) = &taskfile.env {
            let parser = EnvParser::with_config(env_config.clone());
            if let Err(e) = parser.load_env_files() {
                eprintln!("{} Error loading environment files: {}", "✗".red(), e);
            }
            parser
        } else {
            EnvParser::new()
        };

        Self {
            taskfile,
            env_parser,
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

        let name_width = (max_name_len + 2).max(6);
        let desc_width = (max_desc_len + 2).max(13);

        println!(
            "┌{:─<width$}┬{:─<desc_width$}┐",
            "",
            "",
            width = name_width,
            desc_width = desc_width
        );
        println!(
            "│ {:^width$} │ {:^desc_width$} │",
            "Task",
            "Description",
            width = name_width - 2,
            desc_width = desc_width - 2
        );
        println!(
            "├{:─<width$}┼{:─<desc_width$}┤",
            "",
            "",
            width = name_width,
            desc_width = desc_width
        );

        let mut tasks: Vec<_> = self.taskfile.tasks.iter().collect();
        tasks.sort_by(|a, b| a.0.cmp(b.0));

        for (name, task) in tasks {
            let desc = task.desc.as_deref().unwrap_or("No description");
            println!(
                "│ {:width$} │ {:desc_width$} │",
                name,
                desc,
                width = name_width - 2,
                desc_width = desc_width - 2
            );
        }

        println!(
            "└{:─<width$}┴{:─<desc_width$}┘",
            "",
            "",
            width = name_width,
            desc_width = desc_width
        );
    }

    pub async fn run_task(&self, task_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(task) = self.taskfile.tasks.get(task_name) {
            let substituted_cmd = self.env_parser.substitute_env_vars(&task.cmd);
            println!("Running task '{}': {}", task_name, substituted_cmd);

            let parts: Vec<&str> = substituted_cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Err(format!("Empty command for task '{}'", task_name).into());
            }

            let command = parts[0];
            let args = &parts[1..];

            let child = Command::new(command)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let output = child.wait_with_output().await?;

            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }

            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }

            if output.status.success() {
                println!(
                    "{} Task '{}' completed successfully",
                    "✓".green(),
                    task_name
                );
                Ok(())
            } else {
                let code = output.status.code().unwrap_or(-1);
                eprintln!(
                    "{} Task '{}' failed with exit code {}",
                    "✗".red(),
                    task_name,
                    code
                );
                Err(format!("Task '{}' failed with exit code {}", task_name, code).into())
            }
        } else {
            Err(format!("Task '{}' not found in Taskfile", task_name).into())
        }
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
