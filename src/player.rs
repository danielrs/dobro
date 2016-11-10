use ao;
use earwax::Earwax;
use pandora::Track;

pub struct Player {
    ao: ao::Ao,
    driver: ao::Driver,
}

impl Player {
    pub fn new() -> Self {
        Player {
            ao: ao::Ao::new(),
            driver: ao::Driver::new().unwrap(),
        }
    }

    pub fn play(&self, track: Track) {
        if let Some(audio) = track.track_audio {
            if let Ok(mut earwax) = Earwax::new(&audio.high_quality.audio_url) {
                // TODO: Format should replicate earwax format.
                let format = ao::Format::new();
                let device = ao::Device::new(&self.driver, &format, None).unwrap();

                println!(
                    "Playing \"{}\" by {}",
                    track.song_name.unwrap_or("Unknown".to_owned()),
                    track.artist_name.unwrap_or("Unknown".to_owned()),
                );

                while let Some(chunk) = earwax.spit() {
                    device.play(chunk.data);
                }
            }
        }
    }
}
