#![feature(fs_try_exists)]

use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Check if there is a log file
    if fs::try_exists("./target/log.txt").unwrap() == true {
        // Delete log file
        let result = fs::remove_file("./target/log.txt");

        match result {
            Ok(_) => (),
            Err(val) => eprintln!("Error occured: {:#?}", val),
        }
    }

    // Get the name of the package.
    let kernel_name = env::var("CARGO_PKG_NAME")?;

    // Tell rustc to pass the linker script to the linker.
    println!("cargo:rustc-link-arg-bin={kernel_name}=--script=conf/linker.ld");

    // Have cargo rerun this script if the linker script or CARGO_PKG_ENV changes.
    println!("cargo:rerun-if-changed=conf/linker.ld");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_NAME");

    Ok(())
}