mod grab;
mod steam;
mod util;

use crate::grab::*;
use crate::steam::*;
use crate::util::path_exists;
use std::fs::create_dir;
use clap::Parser;
use core::panic;
use dotenvy_macro::dotenv;
use dotenvy::dotenv;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value_t = false)]
    windows: bool,

    #[arg(short, long, default_value_t = false)]
    linux: bool,

    #[arg(short, long, default_value_t = false)]
    flatpak: bool,

    #[arg(short = 'i', long, default_value_t = false)]
    wipe: bool,

    #[arg(long, default_value_t = ("").to_string())]
    lethal_company_path: String,

    #[arg(long, default_value_t = ("").to_string())]
    steam_path: String,
}

fn main() {
    dotenv().expect(".env file not found");

    let mut cli = Cli::parse();

    if !cli.steam_path.is_empty() && cli.flatpak {
        panic!("Can't specify steam path with flatpak")
    }

    if cfg!(windows) && cli.flatpak || cli.linux {
        panic!("Running Linux/Flatpak options on Windows")
    }

    if cfg!(unix) && cli.windows {
        panic!("Running Windows options on Unix")
    }

    if cli.windows && cli.linux || cli.windows && cli.flatpak {
        panic!("Windows and Linux options cannot be mixed")
    }

    if cli.flatpak && cli.linux {
        panic!("Please specify either Linux or Windows")
    }

    // In the event no specific system option is set, just go with system defaults
    if !cli.windows && !cli.flatpak && !cli.linux {
        if cfg!(windows) {
            cli.windows = true;
        } else {
            cli.linux = true;
        }
    } 

    let lc_download = dotenv!("LCDOWNLOAD").to_string();
    let bepinex_download = dotenv!("BEPINEXDOWNLOAD").to_string();

    // If this 
    if !path_exists("./lc") {
        create_dir("./lc").expect("Could not create ./lc");
    }

    let steam: Steam;
    let mut lc_path: String = cli.lethal_company_path;

    if cli.windows {
        // If the path is empty (e.g. not set by cli) then grab from default environmental variable
        if lc_path.is_empty() {
            lc_path = dotenv!("WINDOWSLCPATH").to_string();
        }

        let run_command;
        if cli.steam_path.is_empty() {
            run_command = dotenv!("WINDOWSTEAMPATH").to_string();
        } else {
            run_command = cli.steam_path;
        }
        steam = Steam { lc_path: lc_path.clone(), 
                        run_command: run_command, 
                        bepinex_download: bepinex_download,
                        flatpak: cli.flatpak
                    }
    }
    else if cli.linux {
        if lc_path.is_empty() {
            lc_path = dotenv!("LINUXLCPATH").to_string()
                        .replace("~", &std::env::var("HOME").expect("Export your HOME, e.g. export HOME=/home/user"));
        }

        let run_command;
        if cli.steam_path.is_empty() {
            run_command = dotenv!("LINUXSTEAMPATH").to_string();
        } else {
            run_command = cli.steam_path;
        }
        steam = Steam { lc_path: lc_path.clone(), 
                        run_command: run_command, 
                        bepinex_download: bepinex_download,
                        flatpak: cli.flatpak
                    }
    }
    else {
        if lc_path.is_empty() {
            lc_path = dotenv!("FLATPAKLCPATH").to_string()
                       .replace("~", &std::env::var("HOME").expect("Export your HOME, e.g. export HOME=/home/user"));
        }

        let run_command;
        if cli.steam_path.is_empty() {
            run_command = dotenv!("FLATPAKPATH").to_string();
        } else {
            run_command = cli.steam_path;
        }
        steam = Steam { lc_path: lc_path.clone(),
                        run_command: run_command,
                        bepinex_download: bepinex_download,
                        flatpak: cli.flatpak
                    }
    }

    // Check for BepInEx install at LC PATH, if it isn't there, install BepInEx to this machine
    if !steam.check_bepinex() { steam.install_bepinex(); }

    let mut grabber = Grab{ lc_download: lc_download, plugins: vec![], lc_path: lc_path, wipe: cli.wipe };
    grabber.update();
}