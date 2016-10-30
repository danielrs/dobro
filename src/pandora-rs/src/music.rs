use super::Pandora;
use error::Result;
use method::Method;

#[derive(Debug, Deserialize)]
pub struct Song {
    #[serde(rename="artistName")]
    artist_name: String,
    #[serde(rename="musicToken")]
    music_token: String,
    #[serde(rename="songName")]
    song_name: String,
    score: u32,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    #[serde(rename="artistName")]
    artist_name: String,
    #[serde(rename="musicToken")]
    music_token: String,
    #[serde(rename="likelyMatch")]
    likely_match: bool,
    score: u32,
}

#[derive(Serialize)]
struct Search {
    #[serde(rename="searchText")]
    search_text: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchResults {
    #[serde(rename="nearMatchesAvailble")]
    near_matches_available: bool,
    songs: Vec<Song>,
    artists: Vec<Artist>,
}

////////////////////
// Main struct
////////////////////

pub struct Music<'a> {
    pandora: &'a Pandora<'a>,
}

impl<'a> Music<'a> {
    /// Creates a new Music handler.
    pub fn new(pandora: &'a pandora<'a>) -> Music<'a> {
        Music { pandora: pandora }
    }

    /// Searches for music using the given search string.
    pub fn search(&self, search_text: String) -> Result<SearchResults> {
        // self.pandora.post(Method::MusicSearch,
        unimplemented!()
    }
}
