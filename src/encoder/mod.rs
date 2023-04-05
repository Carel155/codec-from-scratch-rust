use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::{io::Write, mem::size_of_val};

pub fn encode_video(video_frames: &mut Vec<Vec<u8>>) -> Vec<u8> {
    let width: usize = 384;
    let height: usize = 216;

    println!("Original video size: {:?}", size(&video_frames));

    for frame in video_frames.iter_mut() {
        let (y, u, v) = convert_frame_to_yuv(width, height, &frame);
        let (u_sub_sampling, v_sub_sampling) = chroma_subsampling_420(width, height, u, v);

        *frame = [y, u_sub_sampling, v_sub_sampling].concat();
    }

    println!("YUV frames size: {:?}", size(&video_frames));

    let pixel_delta_frames = calculate_pixel_deltas(&video_frames);

    // rle_frames are only for RLE demonstration
    let rle_frames = create_run_length_encoding(&pixel_delta_frames);
    let encoded_frames = deflate_frames(&pixel_delta_frames);

    println!("RLE video size: {:?}", size(&rle_frames));
    println!("Final video size: {:?}", size_of_val(&*encoded_frames));

    return encoded_frames;
}

fn convert_frame_to_yuv(
    width: usize,
    height: usize,
    frame: &Vec<u8>,
) -> (Vec<u8>, Vec<f64>, Vec<f64>) {
    let mut y: Vec<u8> = Vec::with_capacity(width * height);
    let mut u: Vec<f64> = Vec::with_capacity(width * height);
    let mut v: Vec<f64> = Vec::with_capacity(width * height);

    for index in 0..width * height {
        let red = frame[index * 3] as f64;
        let green = frame[index * 3 + 1] as f64;
        let blue = frame[index * 3 + 2] as f64;

        let pixel_y = 0.299 * red + 0.587 * green + 0.114 * blue;
        let pixel_u = -0.169 * red - 0.331 * green + 0.449 * blue + 128.0;
        let pixel_v = 0.499 * red - 0.418 * green - 0.0813 * blue + 128.0;

        y.push(pixel_y as u8);
        u.push(pixel_u);
        v.push(pixel_v);
    }

    return (y, u, v);
}

fn chroma_subsampling_420(
    width: usize,
    height: usize,
    u: Vec<f64>,
    v: Vec<f64>,
) -> (Vec<u8>, Vec<u8>) {
    let mut u_downsampled: Vec<u8> = vec![0; width * height / 4];
    let mut v_downsampled: Vec<u8> = vec![0; width * height / 4];

    for x in (0..height).step_by(2) {
        for y in (0..width).step_by(2) {
            let sampled_u = (u[x * width + y]
                + u[x * width + y + 1]
                + u[(x + 1) * width + y]
                + u[(x + 1) * width + y + 1])
                / 4.0;
            let sampled_v = (v[x * width + y]
                + v[x * width + y + 1]
                + v[(x + 1) * width + y]
                + v[(x + 1) * width + y + 1])
                / 4.0;

            u_downsampled[x / 2 * width / 2 + y / 2] = sampled_u as u8;
            v_downsampled[x / 2 * width / 2 + y / 2] = sampled_v as u8;
        }
    }

    return (u_downsampled, v_downsampled);
}

fn calculate_pixel_deltas(frames: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut encoded: Vec<Vec<u8>> = Vec::with_capacity(frames.len());

    for (index, frame) in frames.iter().enumerate() {
        if index == 0 {
            encoded.push(frame.to_vec());
            continue;
        }

        let mut deltas: Vec<u8> = Vec::with_capacity(frame.len());

        for delta_index in 0..frame.len() {
            deltas.push(frame[delta_index].wrapping_sub(frames[index - 1][delta_index]));
        }

        encoded.push(deltas);
    }

    return encoded;
}

fn create_run_length_encoding(frames: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut rle: Vec<Vec<u8>> = Vec::with_capacity(frames.len());

    for frame in frames.iter() {
        let mut frame_rle: Vec<u8> = Vec::new();
        let mut index = 0;

        while index < frame.len() {
            let current_element = frame[index];
            let mut count: u8 = 1;

            for count_index in index..frame.len() - 1 {
                if frame[count_index + 1] == current_element {
                    // to prevent u8 overflow
                    if count == u8::MAX {
                        frame_rle.push(count);
                        frame_rle.push(current_element);
                        index += count as usize;

                        count = 0;
                    }

                    count += 1;
                } else {
                    break;
                }
            }

            frame_rle.push(count);
            frame_rle.push(current_element);

            index += count as usize;
        }

        rle.push(frame_rle);
    }

    return rle;
}

fn deflate_frames(frames: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());

    for frame in frames.iter() {
        encoder.write_all(frame).unwrap();
    }

    return encoder.finish().unwrap();
}

fn size(frames: &Vec<Vec<u8>>) -> usize {
    let mut size = 0;

    for index in 0..frames.len() {
        size += size_of_val(&*frames[index]);
    }

    return size;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_correct_deltas() {
        let original = Vec::from([
            Vec::from([5, 3, 3, 1]),
            Vec::from([6, 1, 2, 1]),
            Vec::from([5, 1, 2, 1]),
        ]);

        let correct_calculation = Vec::from([
            Vec::from([5, 3, 3, 1]),
            Vec::from([1, 254, 255, 0]),
            Vec::from([255, 0, 0, 0]),
        ]);

        let calculated_deltas = calculate_pixel_deltas(&original);
        assert_eq!(calculated_deltas, correct_calculation);
    }
}
