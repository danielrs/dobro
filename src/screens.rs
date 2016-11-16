use ncurses::*;

use super::Dobro;
use state::*;

use pandora::playlist::Track;

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
            printw(&choice_string);
            refresh();

            break;
            // choice = choice_string.trim().parse::<i32>().unwrap_or(-1);
            // if choice >= 0 && choice < stations.len() as i32 {
            //     break;
            // }
            // else if choice < 0 {
            //     return Trans::Quit
            // }

        }

        // ctx.set_station(stations[choice as usize].clone());
        ctx.set_station(stations[1].clone());
        Trans::Replace(Box::new(StationScreen::new()))
    }
}

pub struct StationScreen {
    track: Option<Track>
}

impl StationScreen {
    pub fn new() -> Self {
        StationScreen {
            track: None,
        }
    }
}

impl State for StationScreen {
    fn update(&mut self, ctx: &mut Dobro) -> Trans {
        halfdelay(1);
        let ch = getch();

        if ch == 'q' as i32 {
            return Trans::Pop;
        }
        else if ch == 'n' as i32 {
            ctx.player.stop();
        }
        else if ch == 'p' as i32 {
            ctx.player.toggle_pause();
        }
        else if ch == 's' as i32 {
            return Trans::Push(Box::new(StationSelectScreen::new()));
        }

        Trans::None
    }
}
