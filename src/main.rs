
use std::{ thread, time, fs};
use std::io::{ self, Write};
use std::path::PathBuf; 
use std::process::Command;
use std::error::Error;
use std::fs::DirEntry;
use std::time::{SystemTime, Duration};
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
    #[arg(
        long, 
        required = false, num_args = 0..=10,
        default_values = &["png", "jpg"])]
    image_extentions: Vec<String>,
    /// Sleep time 
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

fn load_images(image_paths: &Vec<PathBuf>, image_extentions: &Vec<String>) -> Result<Vec<String>,anyhow::Error> {
    let mut images: Vec<String> = vec!();
    for dir in image_paths.into_iter() { 
        if dir.as_path().exists() {
            for entry in fs::read_dir(dir)? {
                let entry: DirEntry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        for e in image_extentions {
                            if ext.to_str().unwrap() == e {    
                                images.push(String::from(entry.path().as_os_str().to_str().unwrap()));
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
        return Err(anyhow!("No images where loaded"));
    }
    return Ok(images);
}

fn main() -> Result<(), Box<dyn Error>>{
    let args = Cli::parse();
    if !args.rotate && !args.force_duplicate {
        let sys = System::new_all();
        let sys_time = SystemTime::now();
        let mut cnt = 0;
        for process in sys.processes_by_name("wallpaper-pick") {
            let proc_time = SystemTime::UNIX_EPOCH + Duration::from_secs(process.start_time());
            cnt +=1;
            let dur = match sys_time.duration_since(proc_time) {
                Ok(d) => d.as_secs(),
                Err(_) => 0,
            };
            if cnt > 1 || dur > 3  {
                eprintln!("Process is already running at {:?}", process.pid());
                std::process::exit(0);
            }
        }
    }
    loop {
        match  load_images(&args.image_paths, &args.image_extentions) {
            Ok(images) => {
                let len = images.len();

                let mut rng = rand::thread_rng();
                let i = rng.gen_range(0..len);
                let wp = images[i].clone();
                let mut cmd = Command::new(args.command.clone().unwrap());
                for a in args.command_args.iter() {
                    cmd.arg(a);
                }
                if args.only_print {
                    print!("{}", wp);
                } else {
                    cmd.arg(wp);
                    let output = cmd.output().expect("failed to execute process");
                    if !output.status.success() { 
                        eprintln!("called {:?} ", io::stderr().write_all(&output.stderr).unwrap());
                        break;
                    }
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
