#[macro_use]
extern crate clap;

extern crate ratd;

use std::process;

use clap::App;

use ratd::server::{
    Server,
    config::Config,
};

fn main() {
    let yml = load_yaml!("cli_en.yml");
    let args = App::from_yaml(yml).get_matches();
    let config = Config::from_clap(args).unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        process::exit(e as i32);
    });

    match Server::new(config) {
        Ok(server) => server.run(),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(e as i32);
        }
    }

    println!("Shutting down...");
}
