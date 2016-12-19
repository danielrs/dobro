use super::super::Dobro;
use super::StationScreen;

use ui::*;
use state::*;

use ncurses as nc;

pub struct StationCreateScreen {
    search_string: String,
}

impl StationCreateScreen {
    pub fn new() -> Self {
        StationCreateScreen {
            search_string: "".to_owned(),
        }
    }
}

impl State for StationCreateScreen {
    fn start(&mut self, _ctx: &mut Dobro) {
        nc::attron(nc::A_BOLD());
        nc::printw("Create station from artist or song: ");
        nc::attroff(nc::A_BOLD());
        self.search_string = getstring();
        nc::printw("\n");
    }

    fn update(&mut self, ctx: &mut Dobro) -> Trans {
        let music = ctx.pandora().music();

        let mut no_results = true;
        if let Ok(results) = music.search(&self.search_string) {
            if  results.nearMatchesAvailable() {
                no_results = false;

                // TODO: Display results here.
            }
        }

        if no_results {
            nc::printw("No results!\n");
            nc::getch();
        }

        Trans::Pop
    }
}

