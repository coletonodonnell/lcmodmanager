use crate::util::{sha256_sum, path_exists, LCError, uninstall};
use crate::steam::Steam;
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs::{File, write, read_to_string, create_dir, remove_file, remove_dir_all};
use reqwest::blocking::get;
use serde::{Serialize, Deserialize};
use tar::Archive;
use fs_extra::dir::{CopyOptions, move_dir};

pub type Plugins = Vec<Plugin>;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Plugin {
    pub identifier: String,
    pub sha256: String,
    pub version: String,
    pub tar_name: String,
    pub files: Option<Vec<String>>,
    pub folders: Option<Vec<String>>,
    pub root: bool // This means the contents should be inside of BepInEx
}

pub struct Grab {
    pub lc_download: String,
    pub lc_path: String,
    pub plugins: Vec<Plugin>,
    pub wipe: bool,
    pub steam: Steam
}

impl Grab {
    // Returns true if the client side plugin's contents matches its manifest
    fn validate(&self, client_plugin: &Plugin) -> bool {
        if client_plugin.files.is_some() {
            for file in client_plugin.files.clone().unwrap() {
                let plugin_file: String;
                if client_plugin.root {
                    plugin_file = format!("{0}/BepInEx/{1}", &self.lc_path, &file);
                } else {
                    plugin_file = format!("{0}/BepInEx/plugins/{1}", &self.lc_path, &file);
                }
                if !path_exists(&plugin_file) {
                    return false;
                }
            }
        }

        if client_plugin.folders.is_some() {
            for folder in client_plugin.folders.clone().unwrap() {
                let plugin_dir: String;
                if client_plugin.root {
                    plugin_dir = format!("{0}/BepInEx/{1}", &self.lc_path, &folder);
                } else {
                    plugin_dir = format!("{0}/BepInEx/plugins/{1}", &self.lc_path, &folder);
                }
                if !path_exists(&plugin_dir) {
                    return false;
                }
            }
        }

        true
    }

    // If the files in the manifest exist, remove them.
    fn remove_plugin(&self, client_plugin: &Plugin) -> Result<()> {
        if client_plugin.root {
            println!("{}", format!("{0} is a root plugin!", client_plugin.identifier));
            return Ok(());
        }
        // If the plugin still exists, have to delete it.
        if client_plugin.files.is_some() {
            for file in client_plugin.files.clone().unwrap() {
                let plugin_file: String;
                if client_plugin.root {
                    plugin_file = format!("{0}/BepInEx/{1}", &self.lc_path, &file);
                } else {
                    plugin_file = format!("{0}/BepInEx/plugins/{1}", &self.lc_path, &file);
                }
                if path_exists(&plugin_file) {
                    remove_file(&plugin_file).with_context(|| format!("Could not delete plugin: {0}", &file))?;
                }
            }
        }

        // If there are folders, search for them and delete them.
        if client_plugin.folders.is_some() {
            for folder in client_plugin.folders.clone().unwrap() {
                let plugin_dir: String;
                if client_plugin.root {
                    plugin_dir = format!("{0}/BepInEx/{1}", &self.lc_path, &folder);
                } else {
                    plugin_dir = format!("{0}/BepInEx/plugins/{1}", &self.lc_path, &folder);
                }
                if path_exists(&plugin_dir) {
                    remove_dir_all(&plugin_dir)
                    .with_context(|| format!("Could not remove plugin folder: {0}", &folder))?;
                }
            }
        }

        Ok(())
    }

    // Download and validate server side.
    fn create_plugin(&self, server_plugin: &Plugin) -> Result<()> {
        // Download the corresponding plugin from the.
        let resp = get(format!("{0}/{1}", &self.lc_download, server_plugin.tar_name)).context("Could not get file")?;
        let body = resp.bytes().context("Could not convert file to bytes")?;
        let plugin_tar_file = format!("./lc/{0}", server_plugin.tar_name);
        write(&plugin_tar_file, body)
            .context(format!("Could not write {0}", server_plugin.tar_name))?;

        // Process the sha256sum so as to validate integrity
        let downloaded_plugin_tar_file_sha256 = sha256_sum(&format!("./lc/{0}", server_plugin.tar_name))
                                                                            .with_context(|| format!("Could not compute checksum for {0}", server_plugin.tar_name))?;
        if downloaded_plugin_tar_file_sha256 != server_plugin.sha256 {
            return Err(LCError::CheckSumDiscrepency(format!("Could not write {0} because of sha256sum discrepency:\nServer: {1}\nDownload: {2}\n", 
                server_plugin.tar_name,
                server_plugin.sha256,
                downloaded_plugin_tar_file_sha256
            )).into());
        }

        // Decompress and unpack the plugin archive.
        let dest: File = File::open(&plugin_tar_file)
            .with_context(|| format!("Could not open {0}", server_plugin.tar_name))?;
        let tar = GzDecoder::new(dest);
        let mut archive = Archive::new(tar);
        archive.set_overwrite(true);
        if server_plugin.root {
            archive.unpack(format!("{0}/BepInEx", self.lc_path))
                .with_context(|| format!("Could not unpack {0}", server_plugin.tar_name))?;
        } else {
            archive.unpack(format!("{0}/BepInEx/plugins", self.lc_path))
                .with_context(|| format!("Could not unpack {0}", server_plugin.tar_name))?;
        }

        // Delete the archive.
        remove_file(&plugin_tar_file)
            .with_context(|| format!("Could not remove {0}", server_plugin.tar_name))?;

        Ok(())
    }

    // Method used to sync the server's plugins with the client.
    pub fn update(&mut self) -> Result<()> {
        // Get the plugins.json from the link and store the body as bytes.
        let mut resp = get(format!("{0}/plugins.json", &self.lc_download)).context("Could not get plugins.json")?;
        let mut body = resp.bytes().context("Could not convert plugins.json to bytes")?;
        write("./lc/plugins.json", body).context("Could not write plugins.json")?;

        resp = get(format!("{0}/plugins.sha256", &self.lc_download)).context("Could not get plugins.sha256")?;
        body = resp.bytes().context("Could not convert plugins.sha256 to bytes")?;
        write("./lc/plugins.sha256", body).context("Could not write plugins.sha256")?;

        // Get the server plugins sha256 and compare it to the local, if they aren't a match that is a problem.
        let plugins_sha256_file = read_to_string("./lc/plugins.sha256")
                                            .context("Couldn't read plugins.sha256")?
                                            .trim().to_string();
        let download_plugins_sha256 = sha256_sum("./lc/plugins.json").context("Could not write ./lc/plugins.json because of sha256sum discrepency")?;

        if plugins_sha256_file != download_plugins_sha256 {
            panic!("Plugin sha256 do not match\nServer: {}\nDownload: {}", plugins_sha256_file, download_plugins_sha256);
        }

        // Convert the latest server plugins.json to plugins object.
        let server_plugins_str = read_to_string("./lc/plugins.json").context("Can't read server plugins.json to string")?;
        let server_plugins_temp: Plugins = serde_json::from_str(&server_plugins_str).context("Could not serialize server plugins.json as Plugins")?;
        self.plugins = server_plugins_temp;

        // If, for some reason, plugins doesn't exist, then create it.
        if !path_exists(&format!("{0}/BepInEx/plugins", &self.lc_path)) {
            create_dir(format!("{0}/BepInEx/plugins", &self.lc_path)).context("Could not create plugins")?
        }

        // If we are in wipe mode, delete then recreate.
        else if self.wipe
        {
            remove_dir_all(format!("{0}/BepInEx/plugins", self.lc_path)).context("Could not remove plugins")?;
            create_dir(format!("{0}/BepInEx/plugins", self.lc_path)).context("Could not create plugins in wipe")?
        }

        if path_exists(&format!("{0}/BepInEx/plugins/plugins.json", self.lc_path)) {
            let client_plugins_str = read_to_string(format!("{0}/BepInEx/plugins/plugins.json", self.lc_path))
                                                                .context("Can't read client plugins.json to string")?;
            match serde_json::from_str(&client_plugins_str) {
                Ok(plugins) => {
                    let client_plugins: Plugins = plugins;

                    let mut count: usize = 0;
                    let client_plugins_size: usize = client_plugins.len().clone();

                    // TODO: Need to handle root directory uninstalls (see HookGenPatcher), in the meantime, a reinstallation of bepinex will be adequate
                    // Iterate through the server plugins.
                    for plugin in &self.plugins {
                        // If the plugins on the client are lexographically less they need to be removed (they don't match with the server.)
                        while count < client_plugins_size && client_plugins[count].identifier.to_lowercase() < plugin.identifier.to_lowercase() {
                            self.remove_plugin(&client_plugins[count])?;
                            count += 1;
                        }
                        if count < client_plugins_size {
                            // If the client and server match and their versions or sha256 aren't the same, update it.
                            if client_plugins[count].identifier.to_lowercase() == plugin.identifier.to_lowercase() {
                                if !self.validate(&client_plugins[count]) || client_plugins[count].version != plugin.version ||
                                client_plugins[count].sha256 != plugin.sha256 {
                                    self.remove_plugin(&client_plugins[count])?;
                                    self.create_plugin(&plugin)?;
                                }
                                count += 1;
                            }

                            else if client_plugins[count].identifier.to_lowercase() > plugin.identifier.to_lowercase() {
                                self.create_plugin(&plugin)?;
                            }
                        } else { self.create_plugin(&plugin)?; }
                    }

                    // If there is anything left over on the client side, delete it.
                    while count < client_plugins_size {
                        self.remove_plugin(&client_plugins[count])?;
                        count += 1;
                    }
                }
                // Broken install, clear, reinstall BepInEx, and write everything over to BepInEx.
                Err(_) => {
                    uninstall(&self.lc_path)?;
                    self.steam.install_bepinex()?;
                    for plugin in &self.plugins {
                        self.create_plugin(plugin)?;
                    }
                }
            }            
        // Fresh install, just write everything over to BepInEx
        } else {
            remove_dir_all(format!("{0}/BepInEx/plugins", self.lc_path)).context("Could not remove plugins")?;
            create_dir(format!("{0}/BepInEx/plugins", self.lc_path)).context("Could not create plugins in fresh/broken install")?;
            for plugin in &self.plugins {
                self.create_plugin(plugin)?;
            }
        }

        // Move over plugins.json and plugins.sha256 and remove the directory.
        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.content_only = true;
        move_dir("./lc/", format!("{0}/BepInEx/plugins", self.lc_path), &options).context("Could not move ./lc to plugins folder")?;

        Ok(())
    }
}