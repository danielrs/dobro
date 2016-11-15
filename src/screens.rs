use ncurses::*;

use super::Dobro;
use state::*;

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

            choice = choice_string.trim().parse::<i32>().unwrap_or(-1);
            if choice >= 0 && choice < stations.stations().len() as i32 {
                break;
            }
            else if choice < 0 {
                return Trans::Quit
            }
        }

        ctx.set_station(stations.stations()[choice as usize].clone());
        Trans::Push(Box::new(StationScreen::new()))
    }
}

pub struct StationScreen {}

impl StationScreen {
    pub fn new() -> Self {
        StationScreen {}
    }
}

impl State for StationScreen {
    fn update(&mut self, ctx: &mut Dobro) -> Trans {
        if let &Some(ref track) = ctx.track() {
            printw(&format!("Playing \"{}\" by {}",
                            track.song_name.clone().unwrap_or("Unknown".to_owned()),
                            track.artist_name.clone().unwrap_or("Unknown".to_owned())));
        }

        Trans::None
    }
}
