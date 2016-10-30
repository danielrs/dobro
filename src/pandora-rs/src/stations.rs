use super::Pandora;
use error::Result;
use method::Method;

use std::slice::Iter;
use std::slice::IterMut;
use std::vec::IntoIter;

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
    pub fn stations(&self) -> &Vec<Station> {
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

    /// Lists the user stations.
    pub fn list(&self) -> Result<StationList> {
        self.pandora.post(Method::UserGetStationList, None)
    }

    pub fn checksum(&self) -> Result<StationListChecksum> {
        unimplemented!()
    }
}

// General structs.

pub struct StationListChecksum {
    pub checksum: String,
}
