use crate::util::{path_exists, sha256_sum};
use std::fs::{File, write, remove_file, read_to_string};
use reqwest::blocking::get;
use flate2::read::GzDecoder;
use tar::Archive;
use std::process::Command;

pub struct Steam {
    pub lc_path: String,
    pub run_command: String,
    pub bepinex_download: String,
    pub bepinex_sha256: String,
    pub flatpak: bool
}

impl Steam {
    pub fn install_bepinex(&self) {
        let mut resp = get(&self.bepinex_download).expect("Could not get file");
        let mut body = resp.bytes().expect("Could not convert file to bytes");
        write("./lc/BepInEx.tar.gz", body).expect("Could not write BepInEx.tar.gz");

        resp = get(&self.bepinex_sha256).expect("Could not get BepInEx sha256 checksum");
        body = resp.bytes().expect("Could not convert  BepInEx sha256 checksum to bytes");
        write("./lc/BepInEx.sha256", body).expect("Could not write BepInEx.sha256");

        // Get the BepInEx sha256 and compare it to the local, if they aren't a match that is a problem
        let bepinex_sha256_file = read_to_string("./lc/BepInEx.sha256")
                                            .expect("Couldn't read BepInEx.sha256")
                                            .trim().to_string();
        let bepinex_sha256_checksum = sha256_sum("./lc/BepInEx.tar.gz");

        if bepinex_sha256_checksum != bepinex_sha256_file {
            panic!("BepInEx sha256 don't match\nServer: {0}\nClient: {1}", bepinex_sha256_file, bepinex_sha256_checksum);
        }

        let dest: File = File::open("./lc/BepInEx.tar.gz").expect("Could not open BepInEx.tar.gz");
        let tar = GzDecoder::new(dest);
        let mut archive = Archive::new(tar);
        archive.unpack(&self.lc_path).expect("Could not unpack tar"); // Write to the lethal company steam path

        remove_file("./lc/BepInEx.tar.gz").expect("Could not remove BepInEx.tar.gz");
        remove_file("./lc/BepInEx.sha256").expect("Could not remove BepInEx.sha256");

        // If this is a flatpak install, we have to run steam via flatpak
        if self.flatpak {
            Command::new(&self.run_command)
                        .args(["run", "com.valvesoftware.Steam", "steam://rungameid/1966720"])
                        .spawn()
                        .expect("Could not run Lethal Company");
        } else {
            Command::new(&self.run_command)
                        .arg("steam://rungameid/1966720")
                        .spawn()
                        .expect("Could not run Lethal Company");
        }

        while !path_exists(&format!("{0}/BepInEx/plugins", self.lc_path)) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}