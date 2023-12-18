#![feature(fs_try_exists)]

mod grab;
mod steam;

use crate::grab::*;
use crate::steam::*;
// use std::io::{BufWriter, Write};
use clap::Parser;
use core::panic;
use dotenvy_macro::dotenv;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value_t = false)]
    windows: bool,

    #[arg(short, long, default_value_t = false)]
    linux: bool,

    #[arg(short, long, default_value_t = false)]
    flatpak: bool,

    #[arg(long, default_value_t = false)]
    wipe: bool
}

fn main() {
    let mut cli = Cli::parse();

    if cfg!(windows) && cli.flatpak || cli.linux {
        panic!("Running Linux/Flatpak options on Windows")
    }

    if cfg!(unix) && cli.windows {
        panic!("Running Windows options on Unix")
    }

    if !cli.windows && !cli.flatpak && !cli.linux {
        cli.windows = true;
    } 
    if cli.windows && cli.linux || cli.windows && cli.flatpak {
        panic!("Windows and Linux options cannot be mixed")
    }

    if cli.flatpak && cli.linux {
        panic!("Please specify either Linux or Windows")
    }

    let lc_download = dotenv!("LCDOWNLOAD").to_string();
    let bepinex_download = dotenv!("BEPINEXDOWNLOAD").to_string();
    let mut grabber = Grab{ lc_download: lc_download, plugins: Plugins{plugins: vec![]} };
    grabber.grab();

    let steam: Steam;
    let lc_path;
    if cli.windows {
        lc_path = dotenv!("WINDOWSLCPATH").to_string();
        steam = Steam { lc_path: lc_path.clone(), 
                        run_command: dotenv!("WINDOWSTEAMPATH").to_string(), 
                        bepinex_download: bepinex_download,
                        flatpak: cli.flatpak
                    }
    }
    else if cli.linux {
        lc_path = dotenv!("LINUXLCPATH").to_string()
                    .replace("~", &std::env::var("HOME").unwrap());
        steam = Steam { lc_path: lc_path.clone(), 
                        run_command: dotenv!("LINUXSTEAMPATH").to_string(), 
                        bepinex_download: bepinex_download,
                        flatpak: cli.flatpak
                    }
    }
    else {
        lc_path = dotenv!("FLATPAKLCPATH").to_string()
                    .replace("~", &std::env::var("HOME").unwrap());
        steam = Steam { lc_path: lc_path.clone(),
                        run_command: dotenv!("FLATPAKPATH").to_string(),
                        bepinex_download: bepinex_download,
                        flatpak: cli.flatpak
                    }
    }

    if !steam.check_bepinex() { steam.create_bepinex(); }

    if cfg!(windows)
    {
        grabber.update(lc_path.clone(), cli.wipe);
    } else {
        grabber.update(lc_path.clone(), cli.wipe);
    }
}