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
For decoding audio files from pandora I decided to use [ffmpeg][ffmpeg]. For playing the decoded PCM data I'll use
[libao][libao] with ffi bindings.

#### TUI
After audio playback.

#### User settings
After everything else is done.

### API
The pandora-rs module interacts with the API found [here](https://6xq.net/pandora-apidoc/json/).

[hyper]: https://github.com/hyperium/hyper
[serde]: https://github.com/serde-rs/serde
[serde_json]: https://github.com/serde-rs/json

[ffmpeg]: https://www.ffmpeg.org/
[libao]: https://www.xiph.org/ao/
