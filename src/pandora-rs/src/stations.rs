use super::Pandora;
use error::Result;
use method::Method;

#[derive(Debug, Deserialize)]
pub struct Station;

pub struct Stations<'a> {
    pandora: &'a Pandora<'a>,
}

impl<'a> Stations<'a> {
    /// Creates a new Stations handler.
    pub fn new(pandora: &'a Pandora<'a>) -> Stations<'a> {
        Stations { pandora: pandora }
    }

    /// Lists the user stations.
    pub fn list(&self) -> Result<Vec<Station>> {
        self.pandora.post(Method::UserGetStationList, None)
    }
}

struct StationsRequest {}
