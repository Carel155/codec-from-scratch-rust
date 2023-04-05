use std::fs::File;
use std::io::Read;

mod decoder;
mod encoder;

fn main() {
    let width: usize = 384;
    let height: usize = 216;

    let mut video_frames = read_video_frames(width, height);

    let encoded_frames = encoder::encode_video(&mut video_frames);

    decoder::decode_video(encoded_frames);
}

fn read_video_frames(width: usize, height: usize) -> Vec<Vec<u8>> {
    let mut result: Vec<Vec<u8>> = Vec::new();

    let mut file = File::open("video.rgb24").expect("Error reading image file");

    let chunk_size = width * height * 3;

    loop {
        let mut frame = Vec::with_capacity(width * height * 3);

        let n = file
            .by_ref()
            .take(chunk_size as u64)
            .read_to_end(&mut frame)
            .expect("Error reading frame data from rbg24 file");

        if n == 0 {
            break;
        }

        result.push(frame);
    }

    return result;
}
