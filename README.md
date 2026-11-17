# Hermit

A small Rust video player with a brief inverted-frame pause effect and a softened time-stop sound.

## Run

Install GTK 4, its GStreamer media backend, and GStreamer codecs. Then run `cargo run -- path/to/video.mp4`, or open a file using the Open button. Format support depends on the installed codecs; no player can guarantee every file or codec.

Space toggles playback. The seek bar and volume slider control the current file.

The pause sound is adapted from [`time-stop-v2.wav`](https://github.com/Fwkkk666/dsh-za-warudo/blob/main/assets/time-stop-v2.wav) in Fwkkk666’s `dsh-za-warudo`. The repository’s MIT license is included at `assets/time-stop-LICENSE.txt`; the recording’s original provenance has not been verified.
