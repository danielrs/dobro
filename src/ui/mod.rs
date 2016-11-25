use ncurses as nc;
use std::char;

pub fn mvrel(rel_y: i32, rel_x: i32) {
    wmvrel(nc::stdscr(), rel_y, rel_x);
}

pub fn wmvrel(window: nc::WINDOW, rel_y: i32, rel_x: i32) {
    let mut y = 0;
    let mut x = 0;
    nc::getyx(window, &mut y, &mut x);
    nc::wmove(window, y + rel_y, x + rel_x);
}

pub fn getstring() -> String {
    wgetstring(nc::stdscr())
}

pub fn wgetstring(window: nc::WINDOW) -> String {
    nc::noecho();

    let mut string = String::with_capacity(32);
    let mut pos_history = Vec::new();

    let mut y = 0;
    let mut x = 0;

    nc::getyx(window, &mut y, &mut x);
    pos_history.push((y, x));

    let mut ch = nc::wgetch(window);
    while ch != '\n' as i32 && ch != '\r' as i32 {
        if ch == 127 {
            if let Some((y, x)) = pos_history.pop() {
                nc::mvwdelch(window, y, x);
                string.pop();
            }
        }
        else if let Some(c) = char::from_u32(ch as u32) {
            nc::getyx(window, &mut y, &mut x);
            pos_history.push((y, x));
            string.push(c);
            nc::wechochar(window, ch as u32);
        }
        ch = nc::wgetch(window);
    }

    string.shrink_to_fit();
    string
}

pub fn getsecretstring() -> String {
    wgetsecretstring(nc::stdscr())
}

pub fn wgetsecretstring(window: nc::WINDOW) -> String {
    nc::noecho();
    let mut string = String::with_capacity(32);

    let mut ch = nc::wgetch(window);
    while ch != '\n' as i32 && ch != '\r' as i32 {
        if ch == 127 {
            string.pop();
        }
        else if let Some(c) = char::from_u32(ch as u32) {
            string.push(c);
        }
        ch = nc::wgetch(window);
    }

    string.shrink_to_fit();
    string
}
