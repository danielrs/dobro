use super::super::Dobro;
use super::StationSelectScreen;

use player::PlayerStatus;
use state::*;

use pandora::playlist::Track;

use ncurses::*;

pub struct StationScreen {
    track: Option<Track>
}

impl StationScreen {
    pub fn new() -> Self {
        StationScreen {
            track: None,
        }
    }

    fn print_song(ctx: &mut Dobro) {
        if let Some(ref track) = ctx.player().state().lock().unwrap().track {
            let status = if ctx.player().is_paused() { "Paused" } else { "Playing" };
            let loved = track.song_rating.unwrap_or(0) > 0;
            printw(
                &format!("{} \"{}\" by {}",
                         status,
                         track.song_name.clone().unwrap_or("Unknown".to_owned()),
                         track.artist_name.clone().unwrap_or("Unknown".to_owned())));
            attron(COLOR_PAIR(1));
            printw(
                &format!("    {}\n", if loved { "<3" } else { " " }));
            attroff(COLOR_PAIR(1));
            refresh();
        }
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
            getyx(stdscr(), &mut y, &mut x);
            mv(y, 0);
            clrtoeol();
            printw(&format!("{:02}:{:02}/{:02}:{:02}\n", mins, secs, total_mins, total_secs));

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
    }

    fn mv(rel_y: i32, rel_x: i32) {
        let mut y = 0;
        let mut x = 0;
        getyx(stdscr(), &mut y, &mut x);
        mv(y + rel_y, x + rel_x);
    }
}

impl State for StationScreen {
    fn start(&mut self, ctx: &mut Dobro) {
        clear();
        mv(0, 0);

        Self::print_song(ctx);
        Self::print_progress(ctx);
        Self::mv(-1, 0);
    }

    fn update(&mut self, ctx: &mut Dobro) -> Trans {
        halfdelay(1);
        let ch = getch();

        if let Some(status) = ctx.player().next_status() {
            match status {
                PlayerStatus::Playing => {
                    Self::print_song(ctx);
                },
                PlayerStatus::Paused => {
                    Self::mv(-1, 0);
                    Self::print_song(ctx);
                },
                PlayerStatus::Unpaused => {
                    Self::mv(-1, 0);
                    Self::print_song(ctx);
                },
                PlayerStatus::Stopped => Self::mv(2, 0),
                PlayerStatus::Shutdown => return Trans::Pop,
                _ => (),
            }
        }

        if ctx.player().state().lock().unwrap().status == PlayerStatus::Playing {
            Self::print_progress(ctx);
            Self::mv(-1, 0);
        }

        if ch == 'q' as i32 {
            return Trans::Quit;
        }
        else if ch == 'n' as i32 {
            ctx.player_mut().skip();
        }
        else if ch == 'p' as i32 {
            ctx.player_mut().toggle_pause();
        }
        else if ch == 's' as i32 {
            return Trans::Push(Box::new(StationSelectScreen::new()));
        }

        Trans::None
    }
}
