# dobro
Unofficial Pandora client written in Rust.

### What's going on right now?

This an app that I'm building during my free time. It will consist of the following main components (most to least important):

- API interaction (pandora-rs).
- Audio playback.
- Text-based user interface (TUI).
- User Settings.

#### API Interaction
Most of the work for this module is already done. It interacts with the API in a very rusty way using [hyper][hyper]; all
the requests/responses are serializing/deserializing is done using [serde][serde] and [serde_json][serde_json].

#### Audio playback
Not done, not started yet. The audio tools for rust is lacking a little bit as of November, 2016. I have been thinking about
using tools such as [rust-media][rust-media]. Or a combination of something like [cpal][cpal] and [ffmpeg][ffmpeg] to keep
the size down. I haven't done any audio-related programming in forever, so getting this done might take a lot of reading
and time.

#### TUI
After audio playback.

#### User settings
After everything else is done.

### API
The pandora-rs module interacts with the API found [here](https://6xq.net/pandora-apidoc/json/).

[hyper]: https://github.com/hyperium/hyper
[serde]: https://github.com/serde-rs/serde
[serde_json]: https://github.com/serde-rs/json

[rust-media]: https://github.com/pcwalton/rust-media
[cpal]: https://github.com/tomaka/cpal
[ffmpeg]: https://www.ffmpeg.org/
