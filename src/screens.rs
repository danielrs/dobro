use ncurses::*;

use super::Dobro;
use player::PlayerStatus;
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

            break;

            // choice = choice_string.trim().parse::<i32>().unwrap_or(-1);
            // if choice >= 0 && choice < stations.len() as i32 {
            //     break;
            // }
            // else if choice < 0 {
            //     return Trans::Quit
            // }
        }

        ctx.player_mut().play(stations[1].clone());
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

        if let Some(status) = ctx.player().next_status() {
            match status {
                PlayerStatus::Playing => {
                    if let Some(ref track) = ctx.player().state().lock().unwrap().track {
                        let loved = track.song_rating.unwrap_or(0) > 0;
                        printw(
                            &format!("\nPlaying \"{}\" by {}",
                                     track.song_name.clone().unwrap_or("Unknown".to_owned()),
                                     track.artist_name.clone().unwrap_or("Unknown".to_owned())));
                        attron(COLOR_PAIR(1));
                        printw(
                            &format!("    {}\n", if loved { "<3" } else { "" }));
                        attroff(COLOR_PAIR(1));
                        refresh();
                    }
                },
                _ => (),
            }
        }
        else {
            if let Some((current, total)) = ctx.player().state().lock().unwrap().progress {
                let total_mins = total / 60;
                let total_secs = total % 60;
                let mins = current / 60;
                let secs = current % 60;

                let mut y = 0;
                let mut x = 0;
                getyx(stdscr(), &mut y, &mut x);
                mv(y, 0);
                clrtoeol();
                printw(&format!("{:02}:{:02}/{:02}:{:02}", mins, secs, total_mins, total_secs));
            }
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
