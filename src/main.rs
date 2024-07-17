use anyhow::{anyhow, Result};
use clap::{CommandFactory, Parser};
use config::{Config, Environment, File};
use directories_next::BaseDirs;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs::DirEntry;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};
use std::{fs, thread, time};

use sysinfo::{ProcessExt, System, SystemExt};
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
    #[arg(
        long,
        required = false, num_args = 0..=10,
        default_values = &["png", "jpg"])]
    image_extentions: Vec<String>,
    /// Sleep time
    /// Configurable in the configuration file
    #[arg(short, long, default_value = "7200", value_name = "SECONDS")]
    sleep: u64,
    /// Rotate immediatley and exit
    #[arg(short, long, default_value = "false", value_name = "ROTATE")]
    rotate: bool,
    /// Force duplicate process
    #[arg(short, long, default_value = "false", value_name = "FORCE_DUPLICATE")]
    force_duplicate: bool,
    /// Only print the image path to the standard out
    #[arg(short, long, default_value = "false", value_name = "ONLY_PRINT")]
    only_print: bool,
}

fn load_images(
    image_paths: &Vec<PathBuf>,
    image_extentions: &Vec<String>,
) -> Result<Vec<String>, anyhow::Error> {
    let mut images: Vec<String> = vec![];
    for dir in image_paths.into_iter() {
        if dir.as_path().exists() {
            for entry in fs::read_dir(dir)? {
                let entry: DirEntry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        for e in image_extentions {
                            if ext.to_str().unwrap() == e {
                                images
                                    .push(String::from(entry.path().as_os_str().to_str().unwrap()));
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("Skipping {} not found", dir.to_str().unwrap())
        }
    }
    if images.len() == 0 {
        return Err(anyhow!("No images where loaded from {:?}", image_paths));
    }
    return Ok(images);
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd = Cli::command();
    let mut args = Cli::parse();
    let config_file = match args.config.clone() {
        Some(file) => Some(File::with_name(file.as_str())),
        None => match BaseDirs::new() {
            Some(dirs) => Some(File::with_name(
                format!(
                    "{}/{}",
                    dirs.config_dir().to_str().unwrap(),
                    CONFIG_FILE_NAME
                )
                .as_str(),
            )),
            None => None,
        },
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
    if args.image_paths.is_none() {
        if let Ok(v) = settings.get::<Vec<PathBuf>>("image_paths") {
            args.image_paths = Some(v);
        }
    }
    for a in cmd.get_arguments() {
        if a.get_id().to_string() == "command" {
            if a.get_default_values()[0].to_str().unwrap() == args.command.clone().unwrap() {
                if let Ok(v) = settings.get::<String>("command") {
                    args.command = Some(v);
                }
            }
        }
        if a.get_id().to_string() == "command_args" {
            if a.get_default_values() == args.command_args.clone() {
                if let Ok(v) = settings.get::<String>("command_args") {
                    args.command = Some(v);
                }
            }
        }
        if a.get_id().to_string() == "sleep" {
            if a.get_default_values()[0]
                .to_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                == args.sleep.clone()
            {
                if let Ok(v) = settings.get::<u64>("sleep") {
                    args.sleep = v;
                }
            }
        }
    }
    let cmd = args.command.clone().unwrap();
    let executable = std::path::Path::new(&cmd);
    if !executable.is_file() {}
    if !args.rotate && !args.force_duplicate {
        let sys = System::new_all();
        let sys_time = SystemTime::now();
        let mut cnt = 0;
        for process in sys.processes_by_name("wallpaper-pick") {
            let proc_time = SystemTime::UNIX_EPOCH + Duration::from_secs(process.start_time());
            cnt += 1;
            let dur = match sys_time.duration_since(proc_time) {
                Ok(d) => d.as_secs(),
                Err(_) => 0,
            };
            if cnt > 1 || dur > 3 {
                eprintln!("Process is already running at {:?}", process.pid());
                std::process::exit(0);
            }
        }
    }
    loop {
        match load_images(&args.image_paths.clone().unwrap(), &args.image_extentions) {
            Ok(images) => {
                let len = images.len();

                let mut rng = rand::thread_rng();
                let i = rng.gen_range(0..len);
                let wp = images[i].clone();
                let mut cmd = Command::new(&cmd);
                for a in args.command_args.iter() {
                    cmd.arg(a);
                }
                if args.only_print {
                    print!("{}", wp);
                } else {
                    cmd.arg(wp);
                    let output = cmd.output().expect("failed to execute process");
                    if !output.status.success() {
                        eprintln!(
                            "called {:?} ",
                            io::stderr().write_all(&output.stderr).unwrap()
                        );
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error {:?}", e);
                break;
            }
        }
        if args.rotate {
            break;
        }
        thread::sleep(time::Duration::from_secs(args.sleep));
    }
    Ok(())
}
