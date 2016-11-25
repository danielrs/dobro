//! Playlists for a [Station](stations/struct.Stations.html).

use super::Pandora;
use error::Result;
use method::Method;
use stations::ToStationToken;

use serde_json;

/// Handler for Playlists.
#[derive(Debug)]
pub struct Playlist<'a> {
    pandora: &'a Pandora,
    station_token: String,
}

impl<'a> Playlist<'a> {
    /// Creates a new Playlist handler.
    pub fn new<T>(pandora: &'a Pandora, station: &T) -> Playlist<'a>
    where T: ToStationToken {
        Playlist { pandora: pandora, station_token: station.to_station_token() }
    }

    /// Gets the current tracklist from Pandora.
    pub fn list(&self) -> Result<Vec<Track>> {
        let tracklist = try!(self.pandora.post::<Tracklist>(
            Method::StationGetPlaylist,
            Some(serde_json::to_value(TracklistRequest {
                station_token: self.station_token.clone()
            }))
        ));
        Ok(tracklist.items)
    }

    // TODO: Result should not be empty
    /// Rates a track.
    pub fn rate<T>(&self, track: T, is_positive: bool) -> Result<()>
    where T: ToTrackToken {
        self.pandora.post_noop(
            Method::StationAddFeedback,
            Some(serde_json::to_value(RateTrackRequest {
                station_token: self.station_token.clone(),
                track_token: track.to_track_token().unwrap_or("".to_owned()),
                is_positive: is_positive,
            }))
        )
    }
}

/// Trait for types that return a track token.
pub trait ToTrackToken {
    fn to_track_token(&self) -> Option<String>;
}

/// List of tracks.
#[derive(Debug, Deserialize)]
struct Tracklist {
    pub items: Vec<Track>,
}

/// Track information. Most fields are optional since
/// the tracklist can include ads.
#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    #[serde(rename="trackToken")]
    pub track_token: Option<String>,
    #[serde(rename="artistName")]
    pub artist_name: Option<String>,
    #[serde(rename="albumName")]
    pub album_name: Option<String>,
    #[serde(rename="songName")]
    pub song_name: Option<String>,
    #[serde(rename="songRating")]
    pub song_rating: Option<u32>,

    #[serde(rename="audioUrlMap")]
    pub track_audio: Option<TrackAudio>,

    #[serde(rename="adToken")]
    pub ad_token: Option<String>,
}

impl Track {
    pub fn is_ad(&self) -> bool {
        self.ad_token.is_some()
    }
}

impl ToTrackToken for Track {
    fn to_track_token(&self) -> Option<String> {
        match self.track_token {
            Some(ref track_token) => Some(track_token.clone()),
            None => None
        }
    }
}

impl<'a> ToTrackToken for &'a Track {
    fn to_track_token(&self) -> Option<String> {
        match self.track_token {
            Some(ref track_token) => Some(track_token.clone()),
            None => None
        }
    }
}

/// Struct for deserializing audio types for a track.
#[derive(Debug, Clone, Deserialize)]
pub struct TrackAudio {
    #[serde(rename="lowQuality")]
    pub low_quality: Audio,
    #[serde(rename="mediumQuality")]
    pub medium_quality: Audio,
    #[serde(rename="highQuality")]
    pub high_quality: Audio,
}

/// Audio information for a track.
#[derive(Debug, Clone, Deserialize)]
pub struct Audio {
    pub bitrate: String,
    pub encoding: String,
    #[serde(rename="audioUrl")]
    pub audio_url: String,
    pub protocol: String,
}

////////////////////
// Request structs
////////////////////

#[derive(Serialize)]
struct TracklistRequest {
    #[serde(rename="stationToken")]
    station_token: String,
}

#[derive(Serialize)]
struct RateTrackRequest {
    #[serde(rename="stationToken")]
    station_token: String,
    #[serde(rename="trackToken")]
    track_token: String,
    #[serde(rename="isPositive")]
    is_positive: bool,
}
