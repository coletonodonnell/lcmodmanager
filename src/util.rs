use data_encoding::HEXLOWER;
use ring::digest::{Context, Digest, SHA256};
use std::fs::{File, metadata, remove_dir_all, remove_file};
use std::io::{BufReader, Read};

// Returns true if BepInEx is installed
pub fn check_bepinex(path: &str) -> bool {
    path_exists(&format!("{0}/BepInEx", path))
}

// Delete's BepInEx if it is present
pub fn uninstall(path: &str) {
    let bep_in_ex = &format!("{0}/BepInEx", path);
    if path_exists(&bep_in_ex) {
        remove_dir_all(&bep_in_ex)
            .expect(&format!("Could not remove BepInEx: {0}", bep_in_ex));
    }

    let win_http = &format!("{0}/winhttp.dll", path);
    if path_exists(&win_http) {
        remove_file(&win_http)
            .expect(&format!("Could not remove winhttp.dll: {0}", win_http));
    }

    let doorstop_config = &format!("{0}/doorstop_config.ini", path);
    if path_exists(&doorstop_config) {
        remove_file(&doorstop_config)
            .expect(&format!("Could not remove doorstop_config.ini: {0}", doorstop_config));
    }

    let changelog = &format!("{0}/changelog.txt ", path);
    if path_exists(&changelog) {
        remove_file(&changelog)
            .expect(&format!("Could not remove changelog.txt : {0}", changelog));
    }
}

// Checks if a path exists
pub fn path_exists(path: &str) -> bool {
    metadata(path).is_ok()
}

// Modified from:
// https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html
fn sha256_digest<R: Read>(mut reader: R) -> Digest {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer).expect("Unable to read buffer");
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    context.finish()
}

pub fn sha256_sum(path: &str) -> String {
    let input = File::open(path).expect("sha256_sum could not open path");
    let reader = BufReader::new(input);
    let digest = sha256_digest(reader);

    return HEXLOWER.encode(digest.as_ref());
}