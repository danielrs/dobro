//! Management of stations such as creation, seeding, deletion and listing.

use super::Pandora;
use error::Result;
use method::Method;
use music::{ToMusicToken, MusicType};

use std::slice::Iter;
use std::slice::IterMut;
use std::vec::IntoIter;

use serde_json;

#[derive(Debug, Deserialize)]
pub struct Seed {
    #[serde(rename="seedId")]
    pub seed_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Station {
    #[serde(rename="stationId")]
    pub station_id: String,
    #[serde(rename="stationName")]
    pub station_name: String,
}

#[derive(Debug, Deserialize)]
pub struct StationList {
    stations: Vec<Station>,
    checksum: String,
}

impl StationList {
    /// Returns the stations.
    pub fn stations(&self) -> &[Station] {
        &self.stations
    }

    /// Returns the checksum for this list.
    pub fn checksum<'a>(&'a self) -> &'a str {
        &self.checksum
    }

    /// Returns an immutable iterator for the
    /// stations.
    pub fn iter(&self) -> Iter<Station> {
        self.stations.iter()
    }

    /// Returns an mutable iterator for the
    /// stations.
    pub fn iter_mut(&mut self) -> IterMut<Station> {
        self.stations.iter_mut()
    }
}

impl<'a> IntoIterator for StationList {
    type Item = Station;
    type IntoIter = IntoIter<Station>;
    fn into_iter(self) -> IntoIter<Station> {
        self.stations.into_iter()
    }
}

impl<'a> IntoIterator for &'a StationList {
    type Item = &'a Station;
    type IntoIter = Iter<'a, Station>;
    fn into_iter(self) -> Iter<'a, Station> {
        self.iter()
    }
}

pub struct Stations<'a> {
    pandora: &'a Pandora<'a>,
}

impl<'a> Stations<'a> {
    /// Creates a new Stations handler.
    pub fn new(pandora: &'a Pandora<'a>) -> Stations<'a> {
        Stations { pandora: pandora }
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
    pub fn rename(&self, station: &Station, station_name: &str) -> Result<Station> {
        self.pandora.post(
            Method::StationRenameStation,
            Some(serde_json::to_value(RenameStationRequest {
                station_token: station.station_id.clone(),
                station_name: station_name.to_owned(),
            }))
        )
    }

    /// Deletes a station.
    pub fn delete(&self, station: &Station) -> Result<()> {
        self.pandora.post(
            Method::StationDeleteStation,
            Some(serde_json::to_value(DeleteStationRequest {
                station_token: station.station_id.clone(),
            }))
        )
    }

    /// Adds a seed to a station.
    pub fn add_seed<T>(&self, station: &Station, music_token: &T) -> Result<Seed>
    where T: ToMusicToken {
        self.pandora.post(
            Method::StationAddMusic,
            Some(serde_json::to_value(AddSeedRequest {
                station_token: station.station_id.clone(),
                music_token: music_token.to_music_token(),
            }))
        )
    }

    pub fn remove_seed(&self, seed: &Seed) -> Result<()> {
        self.pandora.post(
            Method::StationDeleteMusic,
            Some(serde_json::to_value(RemoveSeedRequest {
                seed_id: seed.seed_id.clone(),
            }))
        )
    }

    /// Lists the user stations.
    pub fn list(&self) -> Result<StationList> {
        self.pandora.post(Method::UserGetStationList, None)
    }

    pub fn checksum(&self) -> Result<StationListChecksum> {
        self.pandora.post(Method::UserGetStationListChecksum, None)
    }
}

#[derive(Deserialize)]
pub struct StationListChecksum {
    pub checksum: String,
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
