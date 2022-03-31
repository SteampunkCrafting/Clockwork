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
            .and_then(|(png::OutputInfo { width, height, .. }, mut reader)| {
                let buf_size = (width * height * 4) as usize;
                let mut out_img = Vec::with_capacity(buf_size);
                out_img.resize(buf_size, 0);
                reader.next_frame(&mut out_img)?;
                Ok((width as usize, height as usize, Arc::new(out_img)))
            })
            .map(|(width, height, data)| Self {
                width,
                height,
                data,
            })
    }
}
