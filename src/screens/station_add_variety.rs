use super::super::Dobro;

use state::*;

use screens::station_create::StationMusicScreen;
use pandora::music::ToMusicToken;

use ncurses as nc;

pub struct StationAddVarietyScreen {}

impl StationAddVarietyScreen {
    pub fn new() -> Self {
        StationAddVarietyScreen {}
    }
}

impl StationMusicScreen for StationAddVarietyScreen {
    fn message(&self) -> &'static str {
        "Add variety from artist or song: "
    }

    fn on_choice<T>(&mut self, ctx: &mut Dobro, music_token: &T) where T: ToMusicToken {
        let station = ctx.player().state().lock().unwrap().station.clone();
        if let Some(ref station) = station {
            nc::printw("Adding variety to station... ");
            nc::refresh();
            if let Ok(_) = ctx.pandora().stations().add_seed(station, music_token) {
                nc::printw("Done\n");
            }
            else {
                nc::printw("Unable to add variety to station\n");
            }
        }
    }
}

impl State for StationAddVarietyScreen {
    fn start(&mut self, ctx: &mut Dobro) {
        StationMusicScreen::start(self, ctx);
    }

    fn update(&mut self, _ctx: &mut Dobro) -> Trans {
        Trans::Pop
    }
}
