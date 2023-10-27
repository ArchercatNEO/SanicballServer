pub mod buffer;
pub mod stream;

pub mod data;

pub mod commands;
pub mod game;
pub mod headers;
pub mod server;
pub mod version;

use serde::de::DeserializeOwned;
use server::Server;
use std::{error::Error, fs::File, io::BufReader};

//TODO Make this into an executable app
//TODO Turn into CLI
fn main() {
    //env::set_var("RUST_BACKTRACE", "full");
    let server_config = load_file("config.json").unwrap();
    let match_config = load_file("match.json").unwrap();
    let motd = load_file("motd.json").unwrap();

    let mut server = Server::new(server_config, match_config, motd);

    loop {
        server.update();
        //sleep(Duration::from_secs(1))
    }
}

pub fn to_byte(num: usize) -> u8 {
    (num & 0xFF).try_into().unwrap()
}

///Turn a C# JSON into a Rust JSON
pub fn oxidize(mut json: String) -> String {
    json = json.replace("SanicballCore.MatchMessages.", "");
    json.replace(", SanicballCore", "")
}

fn load_file<T>(filename: &str) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned,
{
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let json = serde_json::from_reader(reader)?;

    Ok(json)
}
