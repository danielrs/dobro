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

use player::Player;
use state::{Automaton};
use screens::*;

use std::sync::{Arc};

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

            while automaton.is_running() {
                automaton.update(&mut dobro);
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

pub struct Dobro {
    pandora: Arc<Pandora>,
    player: Player,
}

impl Dobro {
    /// Creates a new Dobro instance.
    pub fn new(pandora: Pandora) -> Self {
        let pandora = Arc::new(pandora);

        Dobro {
            player: Player::new(pandora.clone()),
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
