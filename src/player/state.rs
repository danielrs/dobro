use pandora::{Station, Track};

/// Player state. It holds the information for the station, track, progress,
/// and status (Playing, Paused, etc).
#[derive(Debug)]
pub struct PlayerState {
    station: Option<Station>,
    track: Option<Track>,
    progress: Option<(i64, i64)>,
    status: PlayerStatus,
}

impl PlayerState {
    /// Returns a new PlayerState.
    pub fn new() -> Self {
        PlayerState {
            station: None,
            track: None,
            progress: None,
            status: PlayerStatus::Shutdown,
        }
    }

    pub fn clear_info(&mut self) {
        self.station = None;
        self.track = None;
        self.progress = None;
    }

    pub fn station(&self) -> Option<Station> {
        self.station.clone()
    }

    pub fn set_station(&mut self, station: Station) {
        self.station = Some(station);
    }

    pub fn track(&self) -> Option<Track> {
        self.track.clone()
    }

    pub fn set_track(&mut self, track: Track) {
        self.track = Some(track);
    }

    pub fn progress(&self) -> Option<(i64, i64)> {
        self.progress.clone()
    }

    pub fn set_progress(&mut self, current: i64, end: i64) {
        self.progress = Some((current, end));
    }

    pub fn status(&self) -> PlayerStatus {
        self.status.clone()
    }

    pub fn set_status(&mut self, status: PlayerStatus) {
        self.status = status;
    }
}

/// Enumeration type for showing player status to the user.
#[derive(Debug, Clone)]
pub enum PlayerStatus {
    // Station-related statuses.
    Standby,

    // Station-related statuses.
    Started(Station),
    Stopped(Station),
    Fetching(Station),

    // Track related statuses.
    Playing(Track),
    Finished(Track),
    Paused(Track),

    // Player not running.
    Shutdown,
}

impl PlayerStatus {
    pub fn is_started(&self) -> bool {
        match *self {
            PlayerStatus::Started(_) => true,
            _ => false,
        }
    }

    pub fn is_stopped(&self) -> bool {
        match *self {
            PlayerStatus::Stopped(_) => true,
            _ => false,
        }
    }

    pub fn is_fetching(&self) -> bool {
        match *self {
            PlayerStatus::Fetching(_) => true,
            _ => false,
        }
    }

    pub fn is_playing(&self) -> bool {
        match *self {
            PlayerStatus::Playing(_) => true,
            _ => false,
        }
    }

    pub fn is_finished(&self) -> bool {
        match *self {
            PlayerStatus::Finished(_) => true,
            _ => false,
        }
    }

    pub fn is_paused(&self) -> bool {
        match *self {
            PlayerStatus::Paused(_) => true,
            _ => false,
        }
    }

    pub fn is_shutdown(&self) -> bool {
        match *self {
            PlayerStatus::Shutdown => true,
            _ => false,
        }
    }
}
