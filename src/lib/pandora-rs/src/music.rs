//! Traits and structs for Songs, Artists, and Searches.

use super::Pandora;
use error::Result;
use method::Method;

use serde_json;

/// Trait for types that can return a music token for seeding.
pub trait ToMusicToken {
    fn to_music_token(&self) -> String;
}

impl ToMusicToken for String {
    fn to_music_token(&self) -> String {
        self.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub enum MusicType {
    #[serde(rename="song")]
    Song,
    #[serde(rename="artist")]
    Artist,
}

/// Song information.
#[derive(Debug, Deserialize)]
pub struct Song {
    #[serde(rename="artistName")]
    pub artist_name: String,
    #[serde(rename="musicToken")]
    pub music_token: String,
    #[serde(rename="songName")]
    pub song_name: String,
    pub score: u32,
}

impl ToMusicToken for Song {
    fn to_music_token(&self) -> String {
        self.music_token.clone()
    }
}

/// Artist information.
#[derive(Debug, Deserialize)]
pub struct Artist {
    #[serde(rename="artistName")]
    pub artist_name: String,
    #[serde(rename="musicToken")]
    pub music_token: String,
    #[serde(rename="likelyMatch")]
    pub likely_match: bool,
    pub score: u32,
}

impl ToMusicToken for Artist {
    fn to_music_token(&self) -> String {
        self.music_token.clone()
    }
}

/// Private struct for sending a search request.
#[derive(Serialize)]
struct Search {
    #[serde(rename="searchText")]
    search_text: String,
    #[serde(rename="includeNearMatches")]
    include_near_matches: bool,
}

/// Search results with both the songs and the artists that matched
/// the search string.
#[derive(Debug, Deserialize)]
pub struct SearchResults {
    #[serde(rename="nearMatchesAvailable")]
    near_matches_available: bool,
    songs: Vec<Song>,
    artists: Vec<Artist>,
}

impl SearchResults {
    /// Returns true if near matches are available.
    pub fn near_matches_available(&self) -> bool {
        self.near_matches_available
    }

    /// Returns the songs in the search results.
    pub fn songs<'a>(&'a self) -> &'a [Song] {
        &self.songs
    }

    /// Returns the artists in the search results.
    pub fn artists<'a>(&'a self) -> &'a [Artist] {
        &self.artists
    }
}

////////////////////
// Main struct
////////////////////

/// Music struct for searching songs and artists.
pub struct Music<'a> {
    pandora: &'a Pandora,
}

impl<'a> Music<'a> {
    /// Creates a new Music handler.
    pub fn new(pandora: &'a Pandora) -> Music<'a> {
        Music { pandora: pandora }
    }

    /// Searches for music using the given search string.
    pub fn search(&self, search_text: &str) -> Result<SearchResults> {
        self.pandora.post(
            Method::MusicSearch,
            Some(serde_json::to_value(Search {
                search_text: search_text.to_owned(),
                include_near_matches: true,
            }))
        )
    }
}
