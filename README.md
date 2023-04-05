# Video Encoding from Scratch in Rust

Rust implementation of [Video Encoding from Scratch](https://github.com/kevmo314/codec-from-scratch) made by kevmo314.

## Run Locally

Clone the project

```bash
cargo run
```

Play the decoded video

```bash
ffplay -f rawvideo -pixel_format rgb24 -video_size 384x216 -framerate 25 decoded.rgb24
```
