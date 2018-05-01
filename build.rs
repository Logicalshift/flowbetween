extern crate pkg_config;

use pkg_config::*;

use std::io::prelude::*;
use std::io;

fn detect_cairo() -> Result<(), Error> {
    let mut config = pkg_config::Config::new();

    config.atleast_version("1.10");
    config.probe("cairo")?;

    Ok(())
}

fn detect_glib() -> Result<(), Error> {
    let mut config = pkg_config::Config::new();

    config.atleast_version("2.32");
    config.probe("glib-2.0")?;

    Ok(())
}

fn detect_gdk_pixbuf() -> Result<(), Error> {
    let mut config = pkg_config::Config::new();

    config.atleast_version("2.26");
    config.probe("gdk-pixbuf-2.0")?;

    Ok(())
}

fn detect_gio() -> Result<(), Error> {
    let mut config = pkg_config::Config::new();

    config.atleast_version("2.34");
    config.probe("gio-2.0")?;

    Ok(())
}

fn detect_gdk() -> Result<(), Error> {
    let mut config = pkg_config::Config::new();

    config.atleast_version("3.12");
    config.probe("gdk-3.0")?;

    Ok(())
}

fn detect_gtk() -> Result<(), Error> {
    let mut config = pkg_config::Config::new();

    config.atleast_version("3.20");
    config.probe("gtk+-3.0")?;

    Ok(())
}

fn main() {
    let can_use_gtk = detect_glib()
        .and_then(|_| detect_gio())
        .and_then(|_| detect_gdk())
        .and_then(|_| detect_gdk_pixbuf())
        .and_then(|_| detect_cairo())
        .and_then(|_| detect_gtk());

    if can_use_gtk.is_ok() {
        println!("cargo:rustc-cfg=auto_gtk");
        writeln!(io::stderr(), "build.rs: picking GTK+ primary user interface").unwrap();
    } else {
        println!("cargo:rustc-cfg=auto_no_gtk");
        writeln!(io::stderr(), "build.rs: disabling GTK support: {:?}", can_use_gtk).unwrap();
    }
}
