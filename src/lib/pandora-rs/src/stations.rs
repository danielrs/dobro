//! Management of stations such as creation, seeding, deletion and listing.

use super::Pandora;
use error::Result;
use method::Method;
use music::{ToMusicToken, MusicType};
use playlist::Playlist;

use serde_json;

/// Handler for stations.
pub struct Stations<'a> {
    pandora: &'a Pandora,
}

impl<'a> Stations<'a> {
    /// Creates a new Stations handler.
    pub fn new(pandora: &'a Pandora) -> Stations<'a> {
        Stations { pandora: pandora }
    }

    /// Lists the user stations.
    pub fn list(&self) -> Result<Vec<Station>> {
        let stations = try!(self.pandora.post::<StationList>(
            Method::UserGetStationList,
            None
        ));
        Ok(stations.stations)
    }

    /// Creates a new station.
    pub fn create<T>(&self, music_token: &T) -> Result<Station> where T: ToMusicToken {
        self.pandora.post(
            Method::StationCreateStation,
            Some(serde_json::to_value(CreateStationRequest {
                track_token: None,
                music_type: None,
                music_token: Some(music_token.to_music_token()),
            }))
        )
    }

    /// Renames a station.
    pub fn rename<T>(&self, station: &T, station_name: &str) -> Result<Station>
    where T: ToStationToken {
        self.pandora.post(
            Method::StationRenameStation,
            Some(serde_json::to_value(RenameStationRequest {
                station_token: station.to_station_token(),
                station_name: station_name.to_owned(),
            }))
        )
    }

    /// Deletes a station.
    pub fn delete<T>(&self, station: &T) -> Result<()> where T: ToStationToken {
        self.pandora.post_noop(
            Method::StationDeleteStation,
            Some(serde_json::to_value(DeleteStationRequest {
                station_token: station.to_station_token(),
            }))
        )
    }

    /// Adds a seed to a station.
    pub fn add_seed<S, T>(&self, station: &S, music_token: &T) -> Result<Seed>
    where S: ToStationToken, T: ToMusicToken {
        self.pandora.post(
            Method::StationAddMusic,
            Some(serde_json::to_value(AddSeedRequest {
                station_token: station.to_station_token(),
                music_token: music_token.to_music_token(),
            }))
        )
    }

    /// Removes a seed from a station.
    pub fn remove_seed(&self, seed: &Seed) -> Result<()> {
        self.pandora.post(
            Method::StationDeleteMusic,
            Some(serde_json::to_value(RemoveSeedRequest {
                seed_id: seed.seed_id.clone(),
            }))
        )
    }

    /// Gets extended station information.
    pub fn station<T>(&self, station: &T) -> Result<Station>
    where T: ToStationToken {
        self.pandora.post(
            Method::StationGetStation,
            Some(serde_json::to_value(GetStationRequest {
                station_token: station.to_station_token(),
                include_extended_attributes: true,
            }))
        )
    }

    /// Gets the current checksum of the station; useful if you need
    /// to check for changes.
    pub fn checksum(&self) -> Result<StationListChecksum> {
        self.pandora.post(Method::UserGetStationListChecksum, None)
    }

    /// Returns a Playlist handler for the given station.
    pub fn playlist<T>(&self, station: &T) -> Playlist where T: ToStationToken {
        Playlist::new(self.pandora, station)
    }
}

/// Trait for types that return a station token.
pub trait ToStationToken {
    fn to_station_token(&self) -> String;
}

/// Single item for StationList.
#[derive(Debug, Clone, Deserialize)]
pub struct Station {
    #[serde(rename="stationId")]
    pub station_id: String,
    #[serde(rename="stationName")]
    pub station_name: String,
}

impl ToStationToken for Station {
    fn to_station_token(&self) -> String {
        self.station_id.clone()
    }
}

/// List of stations.
#[derive(Debug, Deserialize)]
struct StationList {
    pub stations: Vec<Station>,
    pub checksum: String,
}

/// Result type for a Pandora checksum request.
#[derive(Deserialize)]
pub struct StationListChecksum {
    pub checksum: String,
}

/// Extended station information.
#[derive(Debug, Deserialize)]
pub struct ExtendedStation {
    #[serde(rename="stationId")]
    pub station_id: String,
    #[serde(rename="stationName")]
    pub station_name: String,
    #[serde(rename="artUrl")]
    pub art_url: Option<String>,
    // Some stations don't allow adding music (e.g. QuickMix).
    pub music: Option<StationMusic>,
}

/// Seed information for a station.
#[derive(Debug, Deserialize)]
pub struct StationMusic {
    pub songs: Vec<SongSeed>,
    pub artists: Vec<ArtistSeed>,
    pub genre: Option<Vec<GenreSeed>>,
}

/// Generic seed.
#[derive(Debug, Deserialize)]
pub struct Seed {
    #[serde(rename="seedId")]
    pub seed_id: String,
}

/// Song seed.
#[derive(Debug, Deserialize)]
pub struct SongSeed {
    #[serde(rename="seedId")]
    pub seed_id: String,
    #[serde(rename="artistName")]
    pub artist_name: String,
    #[serde(rename="artUrl")]
    pub art_url: String,
    #[serde(rename="songName")]
    pub song_name: String,
    #[serde(rename="musicToken")]
    pub music_token: String,
}

/// Artist seed.
#[derive(Debug, Deserialize)]
pub struct ArtistSeed {
    #[serde(rename="seedId")]
    pub seed_id: String,
    #[serde(rename="artistName")]
    pub artist_name: String,
    #[serde(rename="artUrl")]
    pub art_url: String,
    #[serde(rename="musicToken")]
    pub music_token: String,
}

/// Genre seed.
#[derive(Debug, Deserialize)]
pub struct GenreSeed {
    #[serde(rename="seedId")]
    pub seed_id: String,
    #[serde(rename="artistName")]
    pub genre_name: String,
    #[serde(rename="musicToken")]
    pub music_token: String,
}

////////////////////
// Request structs
////////////////////

#[derive(Serialize)]
struct CreateStationRequest {
    #[serde(rename="trackToken")]
    track_token: Option<String>,
    #[serde(rename="musicType")]
    music_type: Option<MusicType>,
    #[serde(rename="musicToken")]
    music_token: Option<String>,
}

#[derive(Serialize)]
struct RenameStationRequest {
    #[serde(rename="stationToken")]
    station_token: String,
    #[serde(rename="stationName")]
    station_name: String,
}

#[derive(Serialize)]
struct DeleteStationRequest {
    #[serde(rename="stationToken")]
    station_token: String,
}

#[derive(Serialize)]
struct GetStationRequest {
    #[serde(rename="stationToken")]
    station_token: String,
    #[serde(rename="includeExtendedAttributes")]
    include_extended_attributes: bool,
}

#[derive(Serialize)]
struct AddSeedRequest {
    #[serde(rename="stationToken")]
    station_token: String,
    #[serde(rename="musicToken")]
    music_token: String,
}

#[derive(Serialize)]
struct RemoveSeedRequest {
    #[serde(rename="seedId")]
    seed_id: String,
}
