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
mod screens;
mod state;

use ncurses::*;

use pandora::Pandora;
use pandora::stations::{Stations, StationItem, Station};
use pandora::playlist::Track;

use player::Player;
use state::{Trans, State, Automaton};
use screens::*;

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
            let mut dobro = Dobro::new(pandora);
            let mut automaton = Automaton::new(StationSelectScreen::new());

            automaton.start(&mut dobro);

            loop {
                automaton.update(&mut dobro);
                dobro.update();

                if !automaton.is_running() {
                    break;
                }
            }
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
    use player::{Player, PlayerStatus};

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

                if let &Some(ref receiver) = player.receiver() {
                    if let Ok(action) = receiver.try_recv() {
                        if action == PlayerStatus::Stopped {
                            break;
                        }
                    }
                }
            }
            cbreak();
        }
    }
}

pub struct Dobro {
    pandora: Pandora,
    player: Player,

    station: Option<StationItem>,
    track: Option<Track>,
    tracklist: Vec<Track>,
}

impl Dobro {
    /// Creates a new Dobro instance.
    pub fn new(pandora: Pandora) -> Self {
        Dobro {
            pandora: pandora,
            player: Player::new(),

            station: None,
            track: None,
            tracklist: Vec::with_capacity(5),
        }
    }

    /// Returns a reference to the pandora handler.
    pub fn pandora(&self) -> &Pandora {
        &self.pandora
    }

    /// Returns a reference to the player.
    pub fn player(&self) -> &Player {
        &self.player
    }

    /// Returns a mutable reference to the player.
    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    pub fn station(&self) -> &Option<StationItem> {
        &self.station
    }

    pub fn set_station(&mut self, station: StationItem) {
        self.station = Some(station);
        self.track = None;
        self.tracklist.clear();
        self.player.stop();
    }

    pub fn track(&self) -> &Option<Track> {
        &self.track
    }

    pub fn update(&mut self) {
        if let Some(ref station) = self.station {
            if self.tracklist.len() <= 0 {
                let stations = self.pandora.stations();
                let playlist = stations.playlist(station);
                let mut tracklist = playlist.list().unwrap();
                self.tracklist.append(&mut tracklist);
            }

            if self.player.is_stopped() {
                while let Some(track) = self.tracklist.pop() {
                    if !track.is_ad() {
                        self.track = Some(track.clone());
                        self.player.play(track);
                        break;
                    }
                }
            }
        }
    }
}
