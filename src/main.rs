
use std::{ thread, time, fs};
use std::io::{ self, Write};
use std::path::PathBuf; 
use std::process::Command;
use std::error::Error;
use std::fs::DirEntry;
use rand::Rng;
use clap::Parser;
use anyhow::{anyhow, Result};

use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Parser)]
#[command(name = "wallpaper-picker")]
struct Cli {
    /// List of directories where you can find images
    #[arg(short, long, required = true, num_args = 0..=10)]
    image_paths: Vec<PathBuf>,
    /// Binary to execute
    #[arg(short, long, required = false, value_name ="DIR", default_value="feh")]
    command: Option<String>,
    #[arg(
        long, 
        required = false, num_args = 0..=10,
        default_values = &["--no-fehbg", "--bg-scale"])]
    command_args: Vec<String>,
    /// Sleep time 
    #[arg(short, long, default_value = "7200", value_name = "SECONDS")]
    sleep: u64,
    /// Rotate immediatley and exit
    #[arg(short, long, default_value = "false", value_name = "ROTATE")]
    rotate: bool,
    /// Force duplicate process
    #[arg(short, long, default_value = "false", value_name = "FORCE_DUPLICATE")]
    force_duplicate: bool,
}

fn load_images(image_paths: &Vec<PathBuf>) -> Result<Vec<String>,anyhow::Error> {
    let mut images: Vec<String> = vec!();
    for dir in image_paths.into_iter() { 
        if dir.as_path().exists() {
            for entry in fs::read_dir(dir)? {
                let entry: DirEntry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "jpg" || ext =="png" {
                            images.push(String::from(entry.path().as_os_str().to_str().unwrap()));
                        }
                    }
                }
            }
        } else {
            eprintln!("Skipping {} not found", dir.to_str().unwrap())
        }
    }
    if images.len() == 0 {
        return Err(anyhow!("No images where loaded"));
    }
    return Ok(images);
}

fn main() -> Result<(), Box<dyn Error>>{
    let args = Cli::parse();
    if !args.rotate && !args.force_duplicate {
        let sys = System::new_all();
        for process in sys.processes_by_name("wallpaper-pick") {
            eprintln!("Process is already running at {}", process.pid());
            std::process::exit(0);
        }
    }
    loop {
        match  load_images(&args.image_paths) {
            Ok(images) => {
                let len = images.len();

                let mut rng = rand::thread_rng();
                let i = rng.gen_range(0..len);
                let wp = images[i].clone();
                let mut cmd = Command::new(args.command.clone().unwrap());
                for a in args.command_args.iter() {
                    cmd.arg(a);
                }
                cmd.arg(wp);
                let output = cmd.output().expect("failed to execute process");
                if !output.status.success() { 
                    eprintln!("called {:?} ", io::stderr().write_all(&output.stderr).unwrap());
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error {:?}",e);
                break;
            }
        }
        if args.rotate {
            break
        }
        thread::sleep(time::Duration::from_secs(args.sleep)); 
    }
    Ok(())
}
