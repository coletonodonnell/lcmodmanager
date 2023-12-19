use data_encoding::HEXLOWER;
use ring::digest::{Context, Digest, SHA256};
use std::fs::{File, metadata};
use std::io::{BufReader, Read};

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