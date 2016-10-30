//! Methods defined by the unofficial Pandora API.

/// The Pandora method.
#[derive(Debug)]
pub enum Method {
    TestCheckLicensing,

    AuthPartnerLogin,
    AuthUserLogin,

    UserGetStationList,

    StationCreateStation,
    StationAddMusic,
    StationDeleteMusic,
    StationRenameStation,
    StationGetPlaylist,
}

impl ToString for Method {
    fn to_string(&self) -> String {
        match *self {
            Method::TestCheckLicensing   => "test.checkLicensing".to_owned(),

            Method::AuthPartnerLogin     => "auth.partnerLogin".to_owned(),
            Method::AuthUserLogin        => "auth.userLogin".to_owned(),

            Method::UserGetStationList   => "user.getStationList".to_owned(),

            Method::StationCreateStation => "station.createStation".to_owned(),
            Method::StationAddMusic      => "station.addMusic".to_owned(),
            Method::StationDeleteMusic   => "station.deleteMusic".to_owned(),
            Method::StationRenameStation => "station.renameStation".to_owned(),
            Method::StationGetPlaylist   => "station.getPlaylist".to_owned(),
        }
    }
}

impl Method {
    /// Returns true if the method needs the request body to be encrypted with Blowfish ECB.
    /// The [crypt](../crypt/index.html) module takes care of encryption and decryption.
    pub fn is_encrypted(&self) -> bool {
        match *self {
            Method::TestCheckLicensing | Method::AuthPartnerLogin => false,
            _ => true
        }
    }
}
