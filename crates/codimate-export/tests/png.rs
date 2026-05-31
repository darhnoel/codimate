//! Export test: encode a Bitmap to PNG and round-trip it back, all in memory.

use codimate_export::encode_png;
use codimate_render::Bitmap;

/// Decode PNG bytes back into (width, height, straight RGBA8) using the png crate.
fn decode(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().expect("valid PNG header");
    let mut buf = vec![0; reader.output_buffer_size().expect("known buffer size")];
    let info = reader.next_frame(&mut buf).expect("valid PNG frame");
    buf.truncate(info.buffer_size());
    (info.width, info.height, buf)
}

/// Golden test: encoding then decoding yields the same dimensions and pixels.
/// Pure and in-memory — writes nothing to disk.
#[test]
fn encode_png_round_trips_pixels() {
    let bitmap = Bitmap {
        width: 2,
        height: 1,
        rgba: vec![255, 0, 0, 255, /* red */ 0, 0, 0, 255 /* black */],
    };

    let bytes = encode_png(&bitmap);
    let (width, height, rgba) = decode(&bytes);

    assert_eq!((width, height), (2, 1));
    assert_eq!(rgba, bitmap.rgba);
}
