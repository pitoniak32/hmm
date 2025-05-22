use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use error::Error;

pub mod cli;
pub mod error;

fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::builder()
        .filter_level(cli.verbosity.log_level_filter())
        .init();

    let mut entries = load_entries()?;

    log::trace!("before: {:#?}", entries);

    match cli.cmd {
        Some(cmd) if cli.edit => {
            log::trace!("editing '{cmd}'!");
            if let Some(t) = entries.get_mut(&cmd) {
                let res = edit_item(t)?;
                *t = res;
            }
            save_entries(&entries)?;
        }
        Some(cmd) => {
            if let Some(entry) = entries.get(&cmd) {
                present_docs(entry)
            } else {
                // see if user wants to create a new entry if one does't already exist
                if inquire::Confirm::new(&format!("No entry for '{cmd}'! Create one?"))
                    .with_default(true)
                    .prompt()?
                {
                    let res = edit_item(&format!("# {}", cmd.clone()))?;
                    entries.insert(cmd.clone(), res);
                    save_entries(&entries)?;
                }
            }
        }
        None if cli.interactive => {
            let commands: Vec<String> = entries.keys().cloned().collect();
            let res = inquire::Select::new("Which Command", commands).prompt()?;

            present_docs(entries.get(&res).unwrap());
        }
        None if cli.health_check => {
            health_check()?;
        }
        None => {}
    }

    log::trace!("after : {:#?}", entries);

    Ok(())
}

fn present_docs(entry: &str) {
    let result = glow_md(entry).or_else(|err| {
        log::debug!("failed to display doc entry using glow, try running --health-check: {err}");
        less_md(entry)
    });
    if result.is_err() {
        log::error!("failed to display doc entry: {result:?}")
    }
}

fn health_check() -> Result<()> {
    let commands = [
        "glow",
        "less",
        &std::env::var("EDITOR").unwrap_or("vi".to_string()),
    ];
    for cmd in commands {
        let res = std::process::Command::new("which")
            .arg(cmd)
            .output()
            .unwrap();

        let msg = if res.status.success() {
            let path = String::from_utf8_lossy(&res.stdout).trim().to_string();
            format!("ok ({})", &path)
        } else {
            "missing".to_string()
        };
        log::info!("health-check: {cmd} - {msg}!",);
    }

    Ok(())
}

fn less_md(content: &str) -> Result<()> {
    let mut child = std::process::Command::new("less")
        .arg("-r")
        .arg("-") // use a command that reads from stdin
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    // Get a handle to the child's stdin
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(content.as_bytes())?;
    }

    // Wait for the child to complete
    child.wait_with_output()?;

    Ok(())
}

fn glow_md(content: &str) -> Result<()> {
    let mut child = std::process::Command::new("glow")
        .arg("-p")
        .arg("-") // use a command that reads from stdin
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    // Get a handle to the child's stdin
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(content.as_bytes())?;
    }

    // Wait for the child to complete
    child.wait_with_output()?;

    Ok(())
}

fn load_entries() -> Result<BTreeMap<String, String>> {
    let entries_dir =
        get_entries_dir().expect("should be able to create directory to store entries");

    let mut entries = BTreeMap::new();

    for entry in fs::read_dir(entries_dir)? {
        let entry = entry?;
        let path = entry.path();
        log::trace!("loaded: {:?}", path);

        if path.is_file() {
            let ext = path.extension().unwrap().to_string_lossy().to_string();
            if ext == "md" {
                if let Some(filestem) = Path::new(&path).file_stem() {
                    let name = filestem.to_string_lossy().to_string();
                    entries.insert(name.clone(), fs::read_to_string(path)?);
                }
            }
        }
    }

    Ok(entries)
}

fn save_entries(entries: &BTreeMap<String, String>) -> Result<()> {
    let entries_dir =
        get_entries_dir().expect("should be able to create directory to store entries");

    for (cmd, entry) in entries {
        let mut path = entries_dir.join(cmd);
        path.set_extension("md");
        log::trace!("saving: {cmd} ({path:?})");
        std::fs::write(path, entry)?;
    }

    Ok(())
}

fn get_entries_dir() -> Result<PathBuf> {
    let mut xdg_config_path = match std::env::var("XDG_CONFIG_HOME") {
        Ok(config_dir) => PathBuf::from(config_dir),
        Err(_err) => {
            let mut home = PathBuf::from(std::env::var("").expect("HOME should be set"));
            home.push(".config");
            home
        }
    };

    xdg_config_path.push("hmm");
    xdg_config_path.push("entries");

    if !xdg_config_path.exists() {
        fs::create_dir_all(&xdg_config_path)?;
    }

    Ok(xdg_config_path)
}

fn edit_item(item: &str) -> Result<String> {
    let temp_edit_file = create_tmp_file(item)?;
    open_with_editor(&temp_edit_file)?;
    let response = std::fs::read_to_string(&temp_edit_file)?;
    std::fs::remove_file(temp_edit_file)?;
    Ok(response)
}

fn create_tmp_file(content: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("edit_file.yml");
    let mut file = File::create(&file_path).map_err(Error::FailedCreatingTempFile)?;
    writeln!(file, "{}", content).map_err(Error::FailedWritingTempFileContent)?;

    Ok(file_path)
}

fn open_with_editor(file_path: &Path) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or("vi".to_string());

    let output = std::process::Command::new(editor)
        .arg(file_path)
        .spawn()
        .expect("default editor should spawn")
        .wait_with_output()
        .expect("default editor should finish successfully");

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::FailedEditorLaunch)?
    }
}
