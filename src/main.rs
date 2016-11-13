//! This example asks for user login information, shows the available stations, and lets the user select which station to play.
//!
//! **Becareful**, this example is still too simple. It doesn't handle reconnection
//! to pandora when credentials expire.

extern crate rpassword;
extern crate ncurses;

extern crate ao;
extern crate earwax;
extern crate pandora;

mod player;

use ncurses::*;

use pandora::Pandora;
use pandora::stations::{Stations, StationItem};

use ao::*;
use earwax::Earwax;

use std::io;
use std::io::{Write};
use std::{thread, time};

fn main() {
    initscr();
    scrollok(stdscr(), true);
    noecho();

    attron(A_BOLD());
    printw("Welcome to simple pandora!\n");
    printw("Please login below\n");
    attroff(A_BOLD());


    let mut email = String::new();
    attron(A_BOLD());
    printw("\nEmail: ");
    attroff(A_BOLD());
    echo();
    getstr(&mut email);
    noecho();

    let mut password = String::new();
    attron(A_BOLD());
    printw("\nPassword: ");
    attroff(A_BOLD());
    getstr(&mut password);

    match Pandora::new(&email.trim(), &password.trim()) {
        Ok(pandora) => {
            let stations = pandora.stations().list().unwrap();
            for (i, station) in stations.iter().enumerate() {
                printw(&format!("\n{} - {}", i, station.station_name));
            }

            let mut choice = 0;
            loop {
                attron(A_BOLD());
                printw("\nStation choice: ");
                attroff(A_BOLD());
                echo();
                let mut choice_string = String::new();
                getstr(&mut choice_string);
                noecho();

                choice = choice_string.trim().parse::<i32>().unwrap_or(-1);
                if choice >= 0 && choice < stations.stations().len() as i32 {
                    break;
                }
            }

            play(pandora.stations(), &stations.stations()[(choice)as usize]);
        },
        Err(_) => {
            attron(A_BLINK());
            printw("\nUnable to connect to pandora using the provided credentials!");
            attroff(A_BLINK());
            getch();
        }
    }

    endwin();
}

fn play(stations: Stations, station: &StationItem) {
    use std::sync::mpsc::channel;
    use player::{Player, PlayerAction};

    let mut player = Player::new();

    attron(A_BOLD());
    printw(&format!("\nPlaying station \"{}\"", station.station_name));
    attroff(A_BOLD());

    loop {
        let playlist = stations.playlist(station);
        let tracklist = playlist.list().unwrap();

        for track in tracklist {
            if track.is_ad() { continue }

            printw(&format!("\nNow playing \"{}\" by {}",
                track.song_name.clone().unwrap_or("Unknown".to_owned()),
                track.artist_name.clone().unwrap_or("Unknown".to_owned())
            ));

            player.stop();
            player.play(track);

            halfdelay(1);
            loop {
                // Send
                let ch = getch();
                if ch == 'n' as i32 {
                    attron(A_BOLD());
                    printw("  Skipping song...");
                    attroff(A_BOLD());
                    refresh();
                    player.stop();
                    break;
                }
                else if ch == 'p' as i32 {
                    player.toggle_pause();
                }
                else if ch == 'q' as i32 {
                    player.stop();
                    return;
                }

                // Receive
                if let &Some(ref receiver) = player.receiver() {
                    if let Ok(action) = receiver.try_recv() {
                        match action {
                            PlayerAction::Stop => break,
                            _ => ()
                        }
                    }
                }
            }
            cbreak();
        }
    }
}

// pub struct Dobro {
//     pandora: Pandora,
//     player: Player,
// }

// impl Dobro {
//     /// Creates a new Dobro instance.
//     pub fn new(pandora: Pandora) -> Self {
//         Dobro {
//             pandora: pandora,
//             player: Player::new();
//         }
//     }

//     /// Returns a reference to the player.
//     pub fn player(&self) -> &Player {
//         &self.player
//     }

//     /// Returns a mutable reference to the player.
//     pub fn player_mut(&mut self) -> &mut Player {
//         &mut self.player
//     }
// }
