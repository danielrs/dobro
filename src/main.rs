//! This example asks for user login information, shows the available stations, and lets
//! the user select which station to play.
//!
//! **Becareful**, this example is still too simple. It doesn't handle reconnection
//! to pandora when credentials expire.

extern crate rpassword;
extern crate ao;
extern crate earwax;
extern crate pandora;

mod player;

use pandora::Pandora;
use pandora::stations::{Stations, StationItem};

use ao::*;
use earwax::Earwax;

use std::io;
use std::io::{Write};
use std::thread;

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("Welcome to simple pandora!");
    println!("Please login.");

    let mut email = String::new();
    let mut password = String::new();

    loop {
        print!("Email: ");
        stdout.flush().unwrap();
        if stdin.read_line(&mut email).unwrap() > 1 {
            break;
        }
    }

    loop {
        print!("Password: ");
        stdout.flush().unwrap();
        if let Ok(pass) = rpassword::read_password() {
            if pass.len() > 0 {
                password = pass;
                break;
            }
        }
    }

    match Pandora::new(&email.trim(), &password.trim()) {
        Ok(pandora) => {
            let stations = pandora.stations().list().unwrap();
            for (i, station) in stations.iter().enumerate() {
                println!("{} - {}", i, station.station_name);
            }

            let choice = 0;
            loop {
                print!("Station number: ");
                stdout.flush().unwrap();

                let mut choice_string = String::new();
                stdin.read_line(&mut choice_string).unwrap();
                let choice = choice_string.trim().parse::<i32>().unwrap_or(-1);
                println!("Choice: {}", choice);
                if choice >= 0 && choice < stations.stations().len() as i32 {
                    break;
                }
            }

            play(pandora.stations(), &stations.stations()[(choice + 1)as usize]);
        },
        Err(e) => {
            println!("Unable to connect to pandora: {:?}", e);
        }
    }
}

fn play(stations: Stations, station: &StationItem) {
    use player::Player;
    let player = Player::new();

    println!("Station \"{}\"", station.station_name);
    loop {
        let playlist = stations.playlist(station);
        let tracklist = playlist.list().unwrap();

        for track in tracklist {
            player.play(track);
        }
    }
}
