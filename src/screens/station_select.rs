use super::super::Dobro;
use super::StationScreen;

use state::*;

use ncurses::*;

pub struct StationSelectScreen {}

impl StationSelectScreen {
    pub fn new() -> Self {
        StationSelectScreen {}
    }
}

impl State for StationSelectScreen {
    fn update(&mut self, ctx: &mut Dobro) -> Trans {
        let stations = ctx.pandora().stations().list().unwrap();
        for (i, station) in stations.iter().enumerate() {
            printw(&format!("\n{} - {}", i, station.station_name));
        }

        let mut choice = 0;
        loop {
            attron(A_BOLD());
            printw("\nStation choice (negative to quit): ");
            attroff(A_BOLD());
            echo();
            let mut choice_string = String::new();
            getstr(&mut choice_string);
            noecho();

            break;

            // choice = choice_string.trim().parse::<i32>().unwrap_or(-1);
            // if choice >= 0 && choice < stations.len() as i32 {
            //     break;
            // }
            // else if choice < 0 {
            //     return Trans::Quit
            // }
        }

        ctx.player_mut().play(stations[1].clone());
        Trans::Replace(Box::new(StationScreen::new()))
    }
}

