use super::super::Dobro;

use ui::*;
use state::*;

use ncurses as nc;

pub struct StationRenameScreen {}

impl StationRenameScreen {
    pub fn new() -> Self {
        StationRenameScreen {}
    }
}

impl State for StationRenameScreen {
    fn start(&mut self, ctx: &mut Dobro) {
        let station = ctx.player().state().lock().unwrap().station.clone();
        if let Some(station) = station {
            nc::attron(nc::A_BOLD());
            nc::printw(&format!("Renaming station \"{}\"\n", station.station_name));
            nc::attroff(nc::A_BOLD());

            nc::printw("New name (blank to cancel): ");
            let new_name = getstring();
            nc::printw("\n");

            if new_name.len() > 0 {
                nc::printw("Renaming... ");
                nc::refresh();

                if let Ok(_) = ctx.pandora().stations().rename(&station, &new_name) {
                    nc::printw(&format!("Renamed to \"{}\"\n", new_name));
                }
                else {
                    nc::printw("Unable to use that name\n");
                }
            }
            else {
                nc::printw("Leaving old name\n");
            }
        }
    }

    fn update(&mut self, _ctx: &mut Dobro) -> Trans {
        Trans::Pop
    }
}
