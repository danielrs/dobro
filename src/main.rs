//! This example asks for user login information, shows the available stations, and lets the user select which station to play.
//!
//! **Becareful**, this example is still too simple. It doesn't handle reconnection
//! to pandora when credentials expire.

extern crate ncurses;

extern crate ao_rs as ao;
extern crate earwax;
extern crate pandora;

mod player;
mod screens;
mod ui;
mod state;

use ncurses as nc;

use pandora::Pandora;

use player::Player;
use state::{Automaton};
use screens::StationScreen;

use std::sync::{Arc};

use ui::*;

fn main() {
    nc::initscr();
    nc::scrollok(nc::stdscr(), true);
    nc::noecho();

    nc::attron(nc::A_BOLD());
    nc::printw("Welcome to Dobro! The unofficial pandora terminal client.");
    nc::printw("\nPlease login below");
    nc::attroff(nc::A_BOLD());

    nc::attron(nc::A_BOLD());
    nc::printw("\nEmail: ");
    nc::attroff(nc::A_BOLD());
    let email = getstring();

    nc::attron(nc::A_BOLD());
    nc::printw("\nPassword: ");
    nc::attroff(nc::A_BOLD());
    let password = getsecretstring();

    nc::printw("\nLogging in... ");
    nc::refresh();

    match Pandora::new(&email.trim(), &password.trim()) {
        Ok(pandora) => {
            nc::printw("Done\n");
            let mut dobro = Dobro::new(pandora);
            let mut automaton = Automaton::new(StationScreen::new());

            automaton.start(&mut dobro);

            while automaton.is_running() {
                automaton.update(&mut dobro);
                nc::refresh();
            }
        },
        Err(_) => {
            nc::attron(nc::A_BLINK());
            nc::printw("Unable to connect to pandora using the provided credentials\n");
            nc::attroff(nc::A_BLINK());
            nc::getch();
        }
    }

    nc::endwin();
}

pub struct Dobro {
    pandora: Arc<Pandora>,
    player: Player,
}

impl Dobro {
    /// Creates a new Dobro instance.
    pub fn new(pandora: Pandora) -> Self {
        let pandora = Arc::new(pandora);

        Dobro {
            player: Player::new(&pandora),
            pandora: pandora,
        }
    }

    /// Returns a reference to the pandora handler.
    pub fn pandora(&self) -> &Arc<Pandora> {
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
}
