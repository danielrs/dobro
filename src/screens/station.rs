use super::super::Dobro;
use super::StationSelectScreen;

use player::PlayerStatus;
use ui::*;
use state::*;

use pandora::playlist::Track;

use ncurses as nc;

pub struct StationScreen {}

impl StationScreen {
    pub fn new() -> Self {
        StationScreen {}
    }

    fn print_song(status: &str, track: &Track) {
        let loved = track.song_rating.unwrap_or(0) > 0;
        nc::printw(
            &format!("{} \"{}\" by {}",
                     status,
                     track.song_name.clone().unwrap_or("Unknown".to_owned()),
                     track.artist_name.clone().unwrap_or("Unknown".to_owned())));
        nc::printw(
            &format!("  {}\n", if loved { "<3" } else { " " }));
    }

    fn print_progress(ctx: &mut Dobro) {
        if let Some((current, total)) = ctx.player().state().lock().unwrap().progress {
            let total_mins = total / 60;
            let total_secs = total % 60;
            let mins = current / 60;
            let secs = current % 60;

            // Print seconds.
            let mut y = 0;
            let mut x = 0;
            nc::getyx(nc::stdscr(), &mut y, &mut x);
            nc::mv(y, 0);
            nc::clrtoeol();
            nc::printw(&format!("{:02}:{:02}/{:02}:{:02}", mins, secs, total_mins, total_secs));

            // // Progress bar.
            // let mut max_y = 0;
            // let mut max_x = 0;
            // getmaxyx(stdscr(), &mut max_y, &mut max_x);
            // printw("|");
            // let mut total_progress = max_x as i64 - 2;
            // let mut progress = current * total_progress / total;
            // for p in 0..total_progress {
            //     if p < progress {
            //         printw("-");
            //     }
            //     else if p == progress {
            //         printw("=");
            //     }
            //     else {
            //         printw(" ");
            //     }
            // }
            // printw("|\n");
        }
        nc::printw("\n");
    }
}

impl State for StationScreen {
    fn start(&mut self, _ctx: &mut Dobro) {
        nc::printw("Help: press 'p' to toggle pause; 'n' to skip; 's' to change station; 'q' to quit.\n");
    }

    fn resume(&mut self, ctx: &mut Dobro) {
        nc::printw("\n\n");
        ctx.player().report();
    }

    fn update(&mut self, ctx: &mut Dobro) -> Trans {
        if let Some(status) = ctx.player().next_status() {
            match status {
                PlayerStatus::Start(station) => {
                    nc::attron(nc::A_BOLD());
                    nc::printw(&format!("Station \"{}\"\n", station.station_name));
                    nc::attroff(nc::A_BOLD());
                    nc::printw("\n\n");
                },
                PlayerStatus::Playing(track) => {
                    mvrel(-2, 0);
                    Self::print_song("Playing", &track);
                    Self::print_progress(ctx);
                },
                PlayerStatus::Paused(track) => {
                    mvrel(-2, 0);
                    Self::print_song("Paused", &track);
                    Self::print_progress(ctx);
                },
                PlayerStatus::Stopped(track) => {
                    mvrel(-2, 0);
                    Self::print_song("Finished", &track);
                    nc::printw("\n\n");
                },

                _ => (),
            }
        }
        if ctx.player().is_playing() {
            mvrel(-1, 0);
            Self::print_progress(ctx);
        }

        nc::timeout(100);
        let ch = nc::getch();
        nc::timeout(-1);

        match ch as u8 as char {
            'n' => ctx.player_mut().skip(),
            'p' => ctx.player_mut().toggle_pause(),
            's' => return Trans::Push(Box::new(StationSelectScreen::new())),
            'q' => return Trans::Quit,
            rate @ '-' | rate @ '+' => {
                let station = ctx.player().state().lock().unwrap().station.clone();
                let track = ctx.player().state().lock().unwrap().track.clone();
                if let Some(station) = station {
                    if let Some(track) = track {
                        mvrel(-1, 0);
                        nc::printw("Rating track... ");
                        nc::refresh();

                        let res = ctx.pandora().stations()
                                     .playlist(&station).rate(track, rate == '+');
                        match res {
                            Ok(_) => {
                                nc::printw("Done\n");
                                if rate == '-' { ctx.player_mut().skip(); }
                                else { ctx.player().report(); }
                            },
                            e => {
                                nc::printw("Error\n");
                            }
                        };
                        nc::printw("\n\n");
                        nc::refresh();
                    }
                }
            }
            _ => return Trans::None
        };

        Trans::None
    }
}
