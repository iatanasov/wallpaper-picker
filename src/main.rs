use anyhow::{anyhow, bail, Result};
use clap::{CommandFactory, Parser};
use config::{Config, Environment, File};
use directories_next::BaseDirs;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};
use std::{fs, thread, time};

use sysinfo::{get_current_pid, System};
static CONFIG_FILE_NAME: &str = "wallpaper-picker.toml";

#[derive(Parser, Debug, Deserialize, Serialize, Clone)]
#[command(name = "wallpaper-picker")]
struct Cli {
    /// List of directories where you can find images
    /// Configurable in the configuration file
    #[arg(short, long, required = false, num_args = 0..=10)]
    image_paths: Option<Vec<PathBuf>>,
    /// Binary to execute
    /// Configurable in the configuration file
    #[arg(
        short,
        long,
        required = false,
        value_name = "DIR",
        default_value = "/usr/bin/feh"
    )]
    command: Option<String>,
    #[arg(long, required = false, value_name = "DIR")]
    config: Option<String>,
    /// Configure the command that will set the wallpaper
    /// Configurable in the configuration file
    #[arg(
        long,
        required = false, num_args = 0..=10,
        default_values = &["--no-fehbg", "--bg-scale"])]
    command_args: Vec<String>,
    /// List of the image extention to be loaded from the directory
    #[arg(
        long,
        required = false, num_args = 0..=10,
        default_values = &["png", "jpg", "jpeg"])]
    image_extentions: Vec<String>,
    /// Sleep time
    /// Configurable in the configuration file
    #[arg(short, long, default_value = "7200", value_name = "SECONDS")]
    sleep: u64,
    /// Rotate immediatley and exit
    /// This will not check for running process
    #[arg(short, long, default_value = "false", value_name = "ROTATE")]
    rotate: bool,
    /// Force duplicate process
    #[arg(short, long, default_value = "false", value_name = "FORCE_DUPLICATE")]
    force_duplicate: bool,
    /// Only print the image path to the standard out
    #[arg(short, long, default_value = "false", value_name = "ONLY_PRINT")]
    only_print: bool,
    /// Retry the the command execution
    /// In some casses we might have started the loop
    /// before we need all the stuff we need
    #[arg(long, default_value = "10", value_name = "RETRIES")]
    retries: usize,
}

fn load_images(
    image_paths: &Vec<PathBuf>,
    image_extentions: &Vec<String>,
) -> Result<Vec<String>, anyhow::Error> {
    let mut images: Vec<String> = vec![];
    for dir in image_paths.iter() {
        if dir.as_path().exists() {
            for entry in fs::read_dir(dir)? {
                let entry: DirEntry = entry?;
                if entry.path().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if is_allowed_extension(ext, image_extentions) {
                            images.push(String::from(entry.path().as_os_str().to_str().unwrap()));
                        }
                    }
                }
            }
        } else {
            eprintln!("Skipping {} not found", dir.to_str().unwrap())
        }
    }
    if images.is_empty() {
        return Err(anyhow!("No images where loaded from {:?}", image_paths));
    }
    Ok(images)
}

fn is_allowed_extension(ext: &OsStr, image_extentions: &Vec<String>) -> bool {
    if let Some(ext) = ext.to_str() {
        for e in image_extentions {
            if ext == e {
                return true;
            }
        }
    }
    false
}
fn main() -> Result<(), anyhow::Error> {
    let mut args = Cli::parse();
    let mut sys = System::new_all();
    let sys_time = SystemTime::now();
    let config_file = match &args.config {
        Some(file) => Some(File::with_name(file.as_str())),
        None => BaseDirs::new().map(|dirs| {
            File::with_name(
                format!(
                    "{}/{}",
                    dirs.config_dir().to_str().unwrap(),
                    CONFIG_FILE_NAME
                )
                .as_str(),
            )
        }),
    };
    let settings = match config_file {
        Some(config_file) => Config::builder()
            .add_source(config_file)
            .add_source(Environment::with_prefix("WALLPAPER"))
            .build()
            .unwrap(),
        None => Config::builder()
            .add_source(Environment::with_prefix("WALLPAPER"))
            .build()
            .unwrap(),
    };
    let cmd = Cli::command();
    if args.image_paths.is_none() {
        if let Ok(v) = settings.get::<Vec<PathBuf>>("image_paths") {
            args.image_paths = Some(v);
        }
    }
    for a in cmd.get_arguments() {
        if a.get_id() == "command"
            && a.get_default_values()[0].to_str().unwrap() == args.command.as_ref().unwrap()
        {
            if let Ok(v) = settings.get::<String>("command") {
                args.command = Some(v);
            }
        }
        if a.get_id() == "command_args" && a.get_default_values() == args.command_args {
            if let Ok(v) = settings.get::<String>("command_args") {
                args.command = Some(v);
            }
        }
        if a.get_id() == "sleep"
            && a.get_default_values()[0]
                .to_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                == args.sleep
        {
            if let Ok(v) = settings.get::<u64>("sleep") {
                args.sleep = v;
            }
        }
    }
    let image_command = args.command.as_ref().unwrap();
    let executable = std::path::Path::new(&image_command);
    if !executable.is_file() {
        return Err(anyhow!(
            "Command {:?} is not executable",
            executable.to_str()
        ));
    }
    if !args.rotate && !args.force_duplicate {
        let mut cnt = 0;
        sys.refresh_all();
        let current_pid = if let Ok(pid) = get_current_pid() {
            Some(pid)
        } else {
            None
        };
        //dbg!(current_pid);
        for process in sys.processes_by_name("wallpaper-pick".as_ref()) {
            if process.parent() == current_pid {
                continue;
            }
            let proc_time = SystemTime::UNIX_EPOCH + Duration::from_secs(process.start_time());
            cnt += 1;
            let dur = match sys_time.duration_since(proc_time) {
                Ok(d) => d.as_secs(),
                Err(_) => 0,
            };
            if cnt > 1 || dur > 3 {
                eprintln!("Number of process running {cnt}");
                eprintln!(
                    "Process is already running at {:?} for the duration of {dur}",
                    process.pid()
                );
                std::process::exit(0);
            }
        }
    }
    do_work(args)
}
/// The main loop that does the actual work
/// to keep rotating the images
fn do_work(args: Cli) -> Result<(), anyhow::Error> {
    let mut error_count = 0;
    let mut rng = rand::rng();
    loop {
        let images = load_images(args.image_paths.as_ref().unwrap(), &args.image_extentions)?;
        let i = rng.random_range(0..images.len());
        let wp = &images[i];
        if args.only_print {
            println!("{}", wp);
            io::stdout().flush()?;
            break;
        }
        let mut cmd = Command::new(args.command.as_ref().unwrap());
        args.command_args.iter().for_each(|a| {
            cmd.arg(a);
        });
        cmd.arg(wp);
        match cmd.output() {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!(
                        "Command called {:?} :: {}",
                        io::stderr().write_all(&output.stderr)?,
                        error_count
                    );
                    error_count += 1;
                    eprintln!("Increment error {}", error_count);
                } else {
                    error_count = 0
                }
            }
            Err(e) => {
                eprintln!("Error {:?}", e);
                error_count += 1;
            }
        }
        if args.rotate {
            break;
        }
        if error_count >= args.retries {
            bail!(
                "Too many errors {error_count} is more than {}",
                args.retries
            );
        }
        if error_count > 0 {
            thread::sleep(time::Duration::from_secs(10));
        } else {
            thread::sleep(time::Duration::from_secs(args.sleep));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use crate::is_allowed_extension;

    #[test]
    fn is_allowed_extension_true() {
        let ext = OsStr::new("png");
        let image_extentions = vec![String::from("png"), String::from("jpg")];
        assert!(is_allowed_extension(ext, &image_extentions))
    }
    #[test]
    fn is_allowed_extension_false() {
        let ext = OsStr::new("gif");
        let image_extentions = vec![String::from("png"), String::from("jpg")];
        assert!(!is_allowed_extension(ext, &image_extentions))
    }
}
