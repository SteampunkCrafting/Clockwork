use kernel::util::getset::Getters;
use std::{io::Cursor, sync::Arc};

/// A representation of an immutable 2D RGBA Texture
#[derive(Clone, Getters)]
pub struct Texture2D {
    #[getset(get = "pub")]
    data: Arc<Vec<u8>>,

    #[getset(get = "pub")]
    width: usize,

    #[getset(get = "pub")]
    height: usize,
}

impl Texture2D {
    pub fn from_png_src(png: impl AsRef<[u8]>) -> Result<Self, png::DecodingError> {
        png::Decoder::new(Cursor::new(png))
            .read_info()
            // DECODING IMAGE
            .and_then(|(png::OutputInfo { width, height, .. }, mut reader)| {
                let buf_size = (width * height * 4) as usize;
                let mut data_buf = Vec::with_capacity(buf_size);

                data_buf.resize(buf_size, 0);
                reader.next_frame(&mut data_buf)?;

                Ok((width as usize, height as usize, data_buf))
            })
            // TURNING THE IMAGE UPSIDE DOWN
            .map(|(width, height, data_buf)| {
                let mut out_img = Vec::with_capacity(width * height * 4);
                for i in (0..height).rev() {
                    let row = &data_buf[((i * width * 4)..((i + 1) * width * 4))];
                    for byte in row.iter() {
                        out_img.push(*byte)
                    }
                }
                (width, height, Arc::new(out_img))
            })
            // GENERATING STRUCT
            .map(|(width, height, data)| Self {
                width,
                height,
                data,
            })
    }
}
