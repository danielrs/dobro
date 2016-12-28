use super::super::Dobro;
use super::StationCreateScreen;

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

        if stations.len() <= 0 {
            return Trans::Push(Box::new(StationCreateScreen::new()));
        }
        else {
            for (index, station) in stations.iter().enumerate() {
                nc::printw(&format!("\n{} - {}", index, station.station_name));
            }

            let mut choice;
            loop {
                nc::attron(nc::A_BOLD());
                nc::printw("\nStation choice (blank to cancel): ");
                nc::attroff(nc::A_BOLD());
                choice = getchoice();
                nc::printw("\n");

                if choice >= 0 && choice < stations.len() as i32 {
                    break;
                }
                else if choice < 0 {
                    return Trans::Pop
                }
            }

            ctx.player_mut().play(stations[choice as usize].clone());
        }

        Trans::Pop
    }
}
