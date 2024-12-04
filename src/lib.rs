#[cfg(feature = "noise")]
use std::path::PathBuf;
#[cfg(feature = "noise")]
use snow::Builder;
#[cfg(feature = "noise")]
use std::fs::{read, write};

pub mod utils;

#[cfg(feature = "noise")]
pub fn prepare_private_key(private_key: Option<&PathBuf>, builder: &Builder) -> Vec<u8> {
    // prepare private static key: make a new one (and write to file) or read from file
    private_key
        .and_then(|path| {
            if !path.is_file() {
                let keypair = builder.generate_keypair().unwrap();
                write(path, &*keypair.private)
                    .expect("Cannot write the private key to the provided path");
                Some(keypair.private)
            } else {
                read(path).map_or_else(|_| None, |key| Some(key))
            }
        })
        .or_else(|| {
            let keypair = builder.generate_keypair().unwrap();
            Some(keypair.private)
        })
        .expect("Should be unreachable - Cannot generate the private key from the provided path")
}
