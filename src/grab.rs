use flate2::read::GzDecoder;
use std::fs::{File, write, read_to_string, create_dir, remove_file, remove_dir_all};
use reqwest::blocking::get;
use serde::{Serialize, Deserialize};
use tar::Archive;
use fs_extra::dir::{CopyOptions, move_dir};

#[derive(Serialize, Deserialize)]
pub struct Plugins {
    pub plugins: Vec<Plugin>
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Plugin {
    pub name: String,
    pub folder: Option<String>,
    pub version: String,
}

pub struct Grab {
    pub lc_download: String,
    pub plugins: Plugins
}

impl Grab {
    pub fn grab(&mut self) {
        // Get the file from the link and store the body as bytes
        let resp = get(&self.lc_download).expect("Could not get file");
        let body = resp.bytes().expect("Could not convert file to bytes");
        // Create a directory and store the body as lc.tar.gz
        create_dir("./lc").expect("Could not create ./lc");
        write("./lc/lc.tar.gz", body).expect("Could not write lc.tar.gz");

        // Decompress and unpack the file
        let dest: File = File::open("./lc/lc.tar.gz").expect("Could not open lc.tar.gz");
        let tar = GzDecoder::new(dest);
        let mut archive = Archive::new(tar);
        archive.unpack("./lc").expect("Could not unpack tar");

        // Remove the archive
        remove_file("./lc/lc.tar.gz").expect("Could not remove archive");

        // Get the plugins info
        let plugins_str = read_to_string("./lc/plugins.json").expect("Can't read server plugins.json");
        self.plugins = serde_json::from_str(plugins_str.as_str()).expect("Could not serialize server plugins.json");
    }

    pub fn update(&mut self, lc_path: String, wipe: bool) {
        // If, for some reason, plugins doesn't exist, then create it
        if !std::fs::try_exists(format!("{0}/BepInEx/plugins", lc_path)).unwrap() {
            create_dir(format!("{0}/BepInEx/plugins", lc_path)).expect("Could not create plugins")
        }
        
        // If we are in wipe mode, delete then recreate
        else if wipe {
            remove_dir_all(format!("{0}/BepInEx/plugins", lc_path)).expect("Could not remove plugins");
            create_dir(format!("{0}/BepInEx/plugins", lc_path)).expect("Could not create plugins")
        }

        // Write the plugins over to BepInEx
        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.content_only = true;
        move_dir("./lc/", format!("{0}/BepInEx/plugins", lc_path), &options).expect("Could not move ./lc to plugins folder");
    }
}