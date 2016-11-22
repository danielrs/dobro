use super::super::Dobro;
use super::StationScreen;

use ui::*;
use state::*;

use ncurses as nc;

pub struct StationSelectScreen {}

impl StationSelectScreen {
    pub fn new() -> Self {
        StationSelectScreen {}
    }
}

impl State for StationSelectScreen {
    fn start(&mut self, _ctx: &mut Dobro) {
        nc::attron(nc::A_BOLD());
        nc::printw("Stations ");
        nc::attroff(nc::A_BOLD());
    }

    fn update(&mut self, ctx: &mut Dobro) -> Trans {
        let stations = ctx.pandora().stations().list().unwrap();
        for (index, station) in stations.iter().enumerate() {
            nc::printw(&format!("\n{} - {}", index, station.station_name));
        }

        let mut choice = 0;
        loop {
            nc::attron(nc::A_BOLD());
            nc::printw("\nStation choice (negative to quit): ");
            nc::attroff(nc::A_BOLD());
            let choice_string = getstring();
            nc::printw("\n");

            choice = choice_string.trim().parse::<i32>().unwrap_or(-1);
            if choice >= 0 && choice < stations.len() as i32 {
                break;
            }
            else if choice < 0 {
                return Trans::Pop
            }
        }

        ctx.player_mut().play(stations[choice as usize].clone());
        Trans::Replace(Box::new(StationScreen::new()))
    }
}

