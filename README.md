# Hermit

A small Rust video player with a brief inverted-frame pause effect and an original pause sound.

## Run

Install GTK 4, its GStreamer media backend, and GStreamer codecs. Then run `cargo run -- path/to/video.mp4`, or open a file using the Open button. Format support depends on the installed codecs; no player can guarantee every file or codec.

Space toggles playback. The seek bar and volume slider control the current file.
