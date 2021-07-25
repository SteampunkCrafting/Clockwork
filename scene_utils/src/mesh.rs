use getset::Getters;

/// A triangulated mesh, which is stored in a Vertex-Index list form.
///
/// The structure is generic over vertex type T.
#[derive(Getters)]
pub struct Mesh<T> {
    pub indices: Vec<usize>,
    pub vertices: Vec<T>,
}

pub struct TriangleIterator<'a, T> {
    counter: usize,
    mesh: &'a Mesh<T>,
}

impl<T> Mesh<T> {
    pub fn triangle_iter(&self) -> TriangleIterator<'_, T> {
        TriangleIterator {
            counter: 0,
            mesh: self,
        }
    }
}

impl<'a, T> Iterator for TriangleIterator<'a, T> {
    type Item = [&'a T; 3];

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            counter,
            mesh: Mesh {
                indices: idx,
                vertices: vtx,
            },
        } = self;
        let ptr = *counter * 3usize;
        *counter += 1;
        match ptr {
            i if i + 3 > idx.len() => None,
            i => {
                let t = (i..i + 3)
                    .map(|i| idx[i])
                    .map(|v| &vtx[v])
                    .collect::<Vec<_>>();
                Some([t[0], t[1], t[2]])
            }
        }
    }
}
