use flate2::write::DeflateDecoder;
use std::fs::OpenOptions;
use std::io::Write;

pub fn decode_video(data: Vec<u8>) {
    let width: usize = 384;
    let height: usize = 216;

    let decoded_data = decode_deflate(data);
    let mut frames = split_data_to_frames(decoded_data, width, height);

    decode_frame_deltas(&mut frames);
    convert_yuv_to_rgb(&mut frames, width, height);

    write_frames_to_file(frames);
}

fn decode_deflate(data: Vec<u8>) -> Vec<u8> {
    let mut decoder = DeflateDecoder::new(Vec::new());

    decoder.write_all(&data).unwrap();

    return decoder.finish().unwrap();
}

fn split_data_to_frames(data: Vec<u8>, width: usize, height: usize) -> Vec<Vec<u8>> {
    let frame_size = (width * height * 3) / 2;
    let mut result: Vec<Vec<u8>> = Vec::new();

    for frame in data.chunks(frame_size) {
        result.push(frame.to_vec());
    }

    return result;
}

fn decode_frame_deltas(frames: &mut Vec<Vec<u8>>) {
    for index in 0..frames.len() {
        if index == 0 {
            continue;
        }

        for pixel_index in 0..frames[index].len() {
            frames[index][pixel_index] =
                frames[index][pixel_index].wrapping_add(frames[index - 1][pixel_index]);
        }
    }
}

fn convert_yuv_to_rgb(frames: &mut Vec<Vec<u8>>, width: usize, height: usize) {
    for frame in frames.iter_mut() {
        let y = &frame[..width * height];
        let u = &frame[width * height..(width * height) + (width * height / 4)];
        let v = &frame[(width * height) + (width * height / 4)..];

        let mut rgb = Vec::with_capacity(width * height * 3);
        for j in 0..height {
            for k in 0..width {
                let pixel_y = f64::from(y[j * width + k]);
                let pixel_u = f64::from(u[(j / 2) * (width / 2) + (k / 2)]) - 128.0;
                let pixel_v = f64::from(v[(j / 2) * (width / 2) + (k / 2)]) - 128.0;

                let r = clamp(pixel_y + 1.402 * pixel_v, 0.0, 255.0);
                let g = clamp(pixel_y - 0.344 * pixel_u - 0.714 * pixel_v, 0.0, 255.0);
                let b = clamp(pixel_y + 1.772 * pixel_u, 0.0, 255.0);

                rgb.push(r as u8);
                rgb.push(g as u8);
                rgb.push(b as u8);
            }
        }
        *frame = rgb;
    }
}

fn write_frames_to_file(frames: Vec<Vec<u8>>) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("decoded.rgb24")
        .expect("Error creating decoded video file");

    for frame in frames {
        file.write_all(&frame).unwrap();
    }
}

fn clamp(number: f64, min: f64, max: f64) -> f64 {
    if number < min {
        return min;
    }

    if number > max {
        return max;
    }

    return number;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_correct_deltas() {
        let mut original = Vec::from([
            Vec::from([5, 3, 3, 1]),
            Vec::from([1, 254, 255, 0]),
            Vec::from([255, 0, 0, 0]),
        ]);

        let correct_calculation = Vec::from([
            Vec::from([5, 3, 3, 1]),
            Vec::from([6, 1, 2, 1]),
            Vec::from([5, 1, 2, 1]),
        ]);

        decode_frame_deltas(&mut original);
        assert_eq!(original, correct_calculation);
    }
}
