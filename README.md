# lcmodmanager
A centralized mod manager for Lethal Comapny that manages plugins via a remote server and syncs them with a client, as well as install BepInEx.

## Rationale

There exists already many mod managers that support Lethal Company. This mod manager is definitely inferior to these alternatives, but it does serve a centralized source. The goal for this mod manager is to support the following:

* Linux clients that might use interesting/non-default path setups.
* Modified plugins or plugins that have configurations that need to be shared by all clients.
* Allows a tech savvy person in a friend group to act as a centralized mod/plugin distributor.

In addition to these features, I also did this as a Rust teaching experience and a project I can continually improve.

## Client Usage

**Note: using lcmodmanager will (most likely) delete your current mods installed. If you'd like to make sure your current mods are safe, back them up.**

To use, just run the lcmodmanager executable. It will for a default install just run perfectly! If you do have a nonstandard configuration, though, please look at the commands below:

| Long Command          | Short Command | Description                                                       |
| --------------------- | ------------- | ----------------------------------------------------------------- |
| --windows             | -w            | Use Windows paths.                                                |
| --linux               | -l            | Use Linux paths.                                                  |
| --flatpak             | -f            | Use Flatpak paths                                                 |
| --wipe                | -i            | Wipe the plugins directory.                                       |
| --lethal-company-path | N/A           | Specify a Lethal Company path encapsulated by strings to utilize. |
| --steam-path          | N/A           | Specify the path to the steam executable encapsulated by strings. |
| --help                | -h            | Print the help message.                                           |


## Server Usage
The general idea of lcmodmanager is that there exists a server with mods that exist as tarballs. Here is an example layout for such a folder setup that you would host:

```
/path/to/
├── BepInEx.tar.gz
├── LateCompany.tar.gz
├── MoreCompany.tar.gz
├── plugins.json
└── plugins.sha256
```

In this example, two plugins are in this path, as well as `BepInEx.tar.gz` for BepInEx installs. The `plugins.json` specifies specific details about these plugins for the client. Here is the `plugins.json`

```json
[
  {
    "tar_name": "LateCompany.tar.gz",
    "version": "1.0.6",
    "dll_name": "LateCompany.dll",
    "folders": null,
    "sha256": "e6dd667b2ba011e9dfd7373762f6cb37a1a4719d153dbc59d7cad2df011665b1"
  },
  {
    "tar_name": "MoreCompany.tar.gz",
    "version": "1.7.2",
    "dll_name": "MoreCompany.dll",
    "folders": ["MoreCompanyCosmetics"],
    "sha256": "54d6413de5848fff9d65aa5c17c4a49c260f1f961f2a62c292a14623f7bac973"
  }
]
```

Here we see the things that must be specified:

* `tar_name`: The name of the tar archive that will be downloaded.
* `version`: The current version of the mod/plugin.
* `dll_name`: All plugins utilize dll files, point to that main dll.
* `folders`: An array of folders within the tar archive, in the event that there is no folders (which is common) just set this to null.
* `sha256`: This is the lowercase checksum for the tar archive. You can get this with `sha256sum`.

Finally, `plugins.sha256` is the checksum of the `plugins.json`. Whenever the client grabs `plugins.json`, they will verify that the checksums match before proceeding. The idea behind this is that there is now a low risk for possible transfer issues.

## Distribution

If you are interested in distributing your own instance of this mod manager, the process is pretty simple.

1. Edit the `.env` to your liking. `LCDOWNLOAD` is the link to the folder containing the files outlined in [Server Usage](#server-usage). For example, `https://example.com/lc`. `BEPINEXDOWNLOAD` points specifically to the BepInEx tar archive you're using for clients. For example, `https://example.com/lc/BepInEx.tar.gz`.
2. Run `cargo build --release`.
3. Distribute the executables to your friends, or, if they are paranoid (rightfully so) send them this source code with your modified `.env` for them to compile on their system.