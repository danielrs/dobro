use super::super::Dobro;

use ui::*;
use state::*;

use ncurses as nc;

pub struct TrackRateScreen {
    is_positive: bool,
}

impl TrackRateScreen {
    pub fn new(is_positive: bool) -> Self {
        TrackRateScreen {
            is_positive: is_positive
        }
    }
}

impl State for TrackRateScreen {
    fn start(&mut self, ctx: &mut Dobro) {
        let station = ctx.player().state().station();
        let track = ctx.player().state().track();
        if let Some(station) = station {
            if let Some(track) = track {
                mvrel(-1, 0);
                nc::printw("Rating track... ");
                nc::refresh();

                let res = ctx.pandora().stations()
                             .playlist(&station).rate(track, self.is_positive);
                match res {
                    Ok(_) => {
                        nc::printw("Done\n");
                        if !self.is_positive { ctx.player_mut().skip(); }
                        else { ctx.player().report(); }
                    },
                    _ => {
                        nc::printw("Error\n");
                    }
                };
            }
        }
    }

    fn update(&mut self, _ctx: &mut Dobro) -> Trans {
        Trans::Pop
    }
}
