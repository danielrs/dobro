//! Main screen for the application, and initial state for the state machine;
//! Popping this state means the application should end.

use super::super::Dobro;
use super::StationCreateScreen;
use super::StationDeleteScreen;
use super::StationRenameScreen;
use super::StationSelectScreen;
use super::TrackRateScreen;

use player::PlayerStatus;
use ui::*;
use state::*;

use pandora::playlist::Track;

use ncurses as nc;

static HELP_TEXT: &'static str = "Keybindings:
 '?' for help;
 'n' to skip;
 'p' to pause;
 'c' to create station;
 'r' to rename station;
 's' to change station;
 'd' to delete station;
 '+' or '-' to rate the current track;
 'q' to quit.";

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
    fn resume(&mut self, ctx: &mut Dobro) {
        let status = ctx.player().state().lock().unwrap().status.clone();

        match status {
            PlayerStatus::Playing(_) | PlayerStatus::Paused(_) => {
                nc::printw("\n\n");
                ctx.player().report();
            },
            _ => ()
        };
    }

    fn update(&mut self, ctx: &mut Dobro) -> Trans {
        // If status of player is not stopped go to station select screen
        if ctx.player().is_shutdown() {
            return Trans::Push(Box::new(StationSelectScreen::new()));
        }

        if let Some(status) = ctx.player().next_status() {
            match status {
                PlayerStatus::Start(station) => {
                    nc::printw("Type '?' for help.\n");
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
            '?' => {
                nc::printw(&format!("{}\n\n\n", HELP_TEXT));
                ctx.player().report();
            },
            'n' => ctx.player_mut().skip(),
            'p' => ctx.player_mut().toggle_pause(),
            'c' => return Trans::Push(Box::new(StationCreateScreen::new())),
            'r' => return Trans::Push(Box::new(StationRenameScreen::new())),
            's' => return Trans::Push(Box::new(StationSelectScreen::new())),
            'd' => return Trans::Push(Box::new(StationDeleteScreen::new())),
            rate @ '-' | rate @ '+' => return Trans::Push(Box::new(TrackRateScreen::new(rate == '+'))),
            'q' => return Trans::Quit,
            _ => return Trans::None
        };

        Trans::None
    }
}
