use crate::util::path_exists;
use std::fs::{File, write, remove_file};
use reqwest::blocking::get;
use flate2::read::GzDecoder;
use tar::Archive;
use std::process::Command;

pub struct Steam {
    pub lc_path: String,
    pub run_command: String,
    pub bepinex_download: String,
    pub flatpak: bool
}

impl Steam {
    pub fn check_bepinex(&self) -> bool {
        path_exists(format!("{0}/BepInEx", self.lc_path).as_str())
    }

    pub fn install_bepinex(&self) {
        let resp = get(&self.bepinex_download).expect("Could not get file");
        let body = resp.bytes().expect("Could not convert file to bytes");
        write("./lc/BepInEx.tar.gz", body).expect("Could not write BepInEx.tar.gz");

        let dest: File = File::open("./lc/BepInEx.tar.gz").expect("Could not open BepInEx.tar.gz");
        let tar = GzDecoder::new(dest);
        let mut archive = Archive::new(tar);
        archive.unpack(self.lc_path.as_str()).expect("Could not unpack tar"); // Write to the lethal company steam path

        remove_file("./lc/BepInEx.tar.gz").expect("Could not remove archive");

        // If this is a flatpak install, we have to run steam via flatpak
        if self.flatpak {
            Command::new(self.run_command.as_str())
                        .args(["run", "com.valvesoftware.Steam", "steam://rungameid/1966720"])
                        .spawn()
                        .expect("Could not run Lethal Company");
        } else {
            Command::new(self.run_command.as_str())
                        .arg("steam://rungameid/1966720")
                        .spawn()
                        .expect("Could not run Lethal Company");
        }

        while !path_exists(format!("{0}/BepInEx/plugins", self.lc_path).as_str()) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}