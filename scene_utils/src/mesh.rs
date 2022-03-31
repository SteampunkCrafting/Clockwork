use crate::prelude::TexturedMesh;
use kernel::util::getset::Getters;
use std::io::{BufReader, Read};

/// A triangulated mesh, which is stored in a Vertex-Index list form.
///
/// The structure is generic over vertex type T, and is immutable due to
/// the fact that its field content are dependent on each other.
#[derive(Getters)]
pub struct Mesh<T> {
    #[getset(get = "pub")]
    indices: Vec<usize>,
    #[getset(get = "pub")]
    vertices: Vec<T>,
}

pub struct TriangleIterator<'a, T> {
    counter: usize,
    mesh: &'a Mesh<T>,
}

#[derive(kernel::util::thiserror::Error, Debug)]
pub enum MeshCreationError {
    #[error("Failed parsing .obj source {0:?}")]
    ObjError(obj::ObjError),
}

/// Generic Mesh implementation
impl<T> Mesh<T> {
    /// Gets an iterator over triangles of that vertex.
    pub fn triangle_iter(&self) -> TriangleIterator<'_, T> {
        TriangleIterator {
            counter: 0,
            mesh: self,
        }
    }
}

/// Textured Mesh implementation
impl TexturedMesh {
    /// Loads mesh from the WaveFront obj source
    pub fn from_obj_src(obj_src: impl Read) -> Result<Self, MeshCreationError> {
        obj::load_obj::<obj::TexturedVertex, _, usize>(BufReader::new(obj_src))
            .map_err(MeshCreationError::ObjError)
            .map(
                |obj::Obj {
                     vertices, indices, ..
                 }| Self {
                    indices,
                    vertices: vertices.into_iter().map(Into::into).collect(),
                },
            )
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
