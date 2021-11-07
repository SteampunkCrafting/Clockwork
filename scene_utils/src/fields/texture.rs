use kernel::sync::ReadLock;

/// A representation of an immutable 2D RGBA Texture
#[derive(Clone)]
pub struct Texture2D {
    data: ReadLock<Vec<u8>>,
    width: usize,
    height: usize,
}

impl Texture2D {
    pub fn new(width: usize, height: usize, data: impl Into<Vec<u8>>) -> Self {
        Self {
            data: ReadLock::from({
                let data: Vec<u8> = data.into();
                let mut data_buf = Vec::<u8>::default();
                data_buf.reserve(width * height * 4);

                for i in (0..height).rev() {
                    let row = &data[(i * width * 4)..((i + 1) * width * 4)];
                    for byte in row.iter() {
                        data_buf.push(*byte);
                    }
                }

                data_buf
            }),
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data_lock(&self) -> ReadLock<Vec<u8>> {
        self.data.clone()
    }
}
