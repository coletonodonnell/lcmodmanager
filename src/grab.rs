use crate::util::{sha256_sum, path_exists};
use flate2::read::GzDecoder;
use std::fs::{File, write, read_to_string, create_dir, remove_file, remove_dir_all};
use reqwest::blocking::get;
use serde::{Serialize, Deserialize};
use tar::Archive;
use fs_extra::dir::{CopyOptions, move_dir};

pub type Plugins = Vec<Plugin>;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Plugin {
    pub tar_name: String,
    pub version: String,
    pub dll_name: String,
    pub folders: Option<Vec<String>>,
    pub sha256: String
}

pub struct Grab {
    pub lc_download: String,
    pub lc_path: String,
    pub plugins: Vec<Plugin>,
    pub wipe: bool
}

impl Grab {
    // Returns true if the client side plugin's contents matches its manifest
    fn validate(&self, client_plugin: &Plugin) -> bool {
        let plugin_file = format!("{0}/BepInEx/plugins/{1}", &self.lc_path, &client_plugin.dll_name);
        if !path_exists(&plugin_file) {
            return false;
        }

        if client_plugin.folders.is_some() {
            for folder in client_plugin.folders.clone().unwrap() {
                let plugin_dir = format!("{0}/BepInEx/plugins/{1}", &self.lc_path, &folder);
                if !path_exists(&plugin_dir) {
                    return false;
                }
            }
        }

        true
    }

    // If the files in the manifest exist, remove them.
    fn remove_plugin(&self, client_plugin: &Plugin) {
        // If the plugin still exists, have to delete it
        let plugin_file = format!("{0}/BepInEx/plugins/{1}", &self.lc_path, &client_plugin.dll_name);
        if path_exists(&plugin_file) {
            remove_file(&plugin_file)
                .expect(format!("Could not delete plugin: {0}", &client_plugin.dll_name)
                .as_str())
        }

        // If there are folders, search for them and delete them 
        if client_plugin.folders.is_some() {
            for folder in client_plugin.folders.clone().unwrap() {
                let plugin_dir = format!("{0}/BepInEx/plugins/{1}", &self.lc_path, &folder);
                if path_exists(&plugin_dir) {
                    remove_dir_all(&plugin_dir)
                    .expect(format!("Could not remove plugin folder: {0}", &folder)
                    .as_str());
                }
            }
        }
    }

    // Download and validate server side plugin
    fn create_plugin(&self, server_plugin: &Plugin) {
        // Download the corresponding plugin from the server
        let resp = get(format!("{0}/{1}", &self.lc_download, server_plugin.tar_name)).expect("Could not get file");
        let body = resp.bytes().expect("Could not convert file to bytes");
        let plugin_tar_file = format!("./lc/{0}", server_plugin.tar_name);
        write(&plugin_tar_file, body)
            .expect(format!("Could not write {0}", server_plugin.tar_name)
            .as_str());

        // Process the sha256sum so as to validate integrity
        let downloaded_plugin_tar_file_sha256 = sha256_sum(format!("./lc/{0}", server_plugin.tar_name).as_str());
        if downloaded_plugin_tar_file_sha256 != server_plugin.sha256 {
            panic!("Could not write {0} because of sha256sum discrepency:\nServer: {1}\nDownload: {2}\n", 
                server_plugin.tar_name,
                server_plugin.sha256,
                downloaded_plugin_tar_file_sha256
            );
        }

        // Decompress and unpack the plugin archive 
        let dest: File = File::open(&plugin_tar_file)
            .expect(format!("Could not open {0}", server_plugin.tar_name)
            .as_str());
        let tar = GzDecoder::new(dest);
        let mut archive = Archive::new(tar);
        archive.unpack(format!("{0}/BepInEx/plugins", self.lc_path))
            .expect(format!("Could not unpack {0}", server_plugin.tar_name)
            .as_str());

        // Delete the archive
        remove_file(&plugin_tar_file)
            .expect(format!("Could not remove {0}", server_plugin.tar_name)
            .as_str());
    }

    // Method used to sync the server's plugins with the client
    pub fn update(&mut self) {
        // Get the plugins.json from the link and store the body as bytes
        let mut resp = get(format!("{0}/plugins.json", &self.lc_download)).expect("Could not get plugins.json");
        let mut body = resp.bytes().expect("Could not convert plugins.json to bytes");
        write("./lc/plugins.json", body).expect("Could not write plugins.json");

        resp = get(format!("{0}/plugins.sha256", &self.lc_download)).expect("Could not get plugins.sha256");
        body = resp.bytes().expect("Could not convert plugins.sha256 to bytes");
        write("./lc/plugins.sha256", body).expect("Could not write plugins.sha256");

        // Get the server plugins sha256 and compare it to the local, if they aren't a match that is a problem
        let plugins_sha256_file = read_to_string("./lc/plugins.sha256")
                                            .expect("Couldn't read plugins.sha256")
                                            .trim().to_string();
        let download_plugins_sha256 = sha256_sum("./lc/plugins.json");

        if plugins_sha256_file != download_plugins_sha256 {
            panic!("Plugin sha256 do not match\nServer: {}\nDownload: {}", plugins_sha256_file, download_plugins_sha256);
        }

        // Convert the latest server plugins.json to plugins object 
        let server_plugins_str = read_to_string("./lc/plugins.json").expect("Can't read server plugins.json to string");
        let server_plugins_temp: Plugins = serde_json::from_str(server_plugins_str.as_str()).expect("Could not serialize server plugins.json as Plugins");
        self.plugins = server_plugins_temp;

        // If, for some reason, plugins doesn't exist, then create it
        if !path_exists(format!("{0}/BepInEx/plugins", &self.lc_path).as_str()) {
            create_dir(format!("{0}/BepInEx/plugins", &self.lc_path)).expect("Could not create plugins")
        }

        // If we are in wipe mode, delete then recreate
        else if self.wipe
        {
            remove_dir_all(format!("{0}/BepInEx/plugins", self.lc_path)).expect("Could not remove plugins");
            create_dir(format!("{0}/BepInEx/plugins", self.lc_path)).expect("Could not create plugins in wipe")
        }

        if path_exists(format!("{0}/BepInEx/plugins/plugins.json", self.lc_path).as_str()) {
            let client_plugins_str = read_to_string(format!("{0}/BepInEx/plugins/plugins.json", self.lc_path)).expect("Can't read client plugins.json to string");
            let client_plugins_temp: Plugins = serde_json::from_str(client_plugins_str.as_str()).expect("Could not serialize client plugins.json as Plugins");
            let client_plugins = client_plugins_temp;

            let mut count: usize = 0;
            let client_plugins_size: usize = client_plugins.len().clone();

            // Iterate through the server plugins
            for plugin in &self.plugins {
                // If the plugins on the client are lexographically less they need to be removed (they don't match with the server)
                while count < client_plugins_size && client_plugins[count].dll_name.to_lowercase() < plugin.dll_name.to_lowercase() {
                    self.remove_plugin(&client_plugins[count]);
                    count += 1;
                }
                if count < client_plugins_size {
                    // If the client and server match and their versions or sha256 aren't the same, update it
                    if client_plugins[count].dll_name.to_lowercase() == plugin.dll_name.to_lowercase() {
                        if !self.validate(&client_plugins[count]) || client_plugins[count].version != plugin.version ||
                        client_plugins[count].sha256 != plugin.sha256 {
                            self.remove_plugin(&client_plugins[count]);
                            self.create_plugin(&plugin);
                        }
                        count += 1;
                    }

                    else if client_plugins[count].dll_name.to_lowercase() > plugin.dll_name.to_lowercase() {
                        self.create_plugin(&plugin);
                    }
                } else { self.create_plugin(&plugin); }
            }

            // If there is anything left over on the client side, delete it
            while count < client_plugins_size {
                self.remove_plugin(&client_plugins[count]);
                count += 1;
            }

        // Fresh or broken install, just write everything over to BepInEx
        } else {
            remove_dir_all(format!("{0}/BepInEx/plugins", self.lc_path)).expect("Could not remove plugins");
            create_dir(format!("{0}/BepInEx/plugins", self.lc_path)).expect("Could not create plugins in fresh/broken install");
            for plugin in &self.plugins {
                self.create_plugin(plugin);
            }
        }

        // Move over plugins.json and plugins.sha256 and remove the directory.
        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.content_only = true;
        move_dir("./lc/", format!("{0}/BepInEx/plugins", self.lc_path), &options).expect("Could not move ./lc to plugins folder");
    }
}