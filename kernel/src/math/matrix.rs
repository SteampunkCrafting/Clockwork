use itertools::iproduct;
use std::{
    borrow::{Borrow, BorrowMut},
    mem::MaybeUninit,
    ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Sub, SubAssign},
};

/// A statically sized 2D array, whose elements are allocated on the stack.
///
/// This structure is used primarily for linear algebra.
pub struct Matrix<T, const N: usize, const M: usize>([[T; M]; N]);

/// Everything, which is convertible to a 2D array is also convertible to
/// the Matrix.
impl<U, T, const N: usize, const M: usize> From<U> for Matrix<T, N, M>
where
    U: Into<[[T; M]; N]>,
{
    fn from(x: U) -> Self {
        Self(x.into())
    }
}

/// Some matrices (of equal shape) support additions
impl<T, U, const N: usize, const M: usize> Add<Matrix<U, N, M>> for Matrix<T, N, M>
where
    T: Clone + Add<U>,
    U: Clone,
{
    type Output = Matrix<<T as Add<U>>::Output, N, M>;

    fn add(self, rhs: Matrix<U, N, M>) -> Self::Output {
        let mut out = [[MaybeUninit::<()>::uninit(); M]; N]
            .map(|row| row.map(|_| MaybeUninit::<<T as Add<U>>::Output>::uninit()));
        for (i, j) in iproduct!(0..N, 0..M) {
            out[i][j].write(self[(i, j)].clone() + rhs[(i, j)].clone());
        }
        out.map(|row| row.map(|element| unsafe { element.assume_init() }))
            .into()
    }
}

/// Some matrices (of equal shape) support additions
impl<T, U, const N: usize, const M: usize> AddAssign<Matrix<U, N, M>> for Matrix<T, N, M>
where
    T: AddAssign<U>,
    U: Clone,
{
    fn add_assign(&mut self, rhs: Matrix<U, N, M>) {
        for (i, j) in iproduct!(0..N, 0..M) {
            self[(i, j)] += rhs[(i, j)].clone()
        }
    }
}

/// Some matrices (of equal shape) support subtractions
impl<T, U, const N: usize, const M: usize> Sub<Matrix<U, N, M>> for Matrix<T, N, M>
where
    T: Clone + Sub<U>,
    U: Clone,
{
    type Output = Matrix<<T as Sub<U>>::Output, N, M>;

    fn sub(self, rhs: Matrix<U, N, M>) -> Self::Output {
        let mut out = [[MaybeUninit::<()>::uninit(); M]; N]
            .map(|row| row.map(|_| MaybeUninit::<<T as Sub<U>>::Output>::uninit()));
        for (i, j) in iproduct!(0..N, 0..M) {
            out[i][j].write(self[(i, j)].clone() - rhs[(i, j)].clone());
        }
        out.map(|row| row.map(|element| unsafe { element.assume_init() }))
            .into()
    }
}

/// Some matrices (of equal shape) support subtractions
impl<T, U, const N: usize, const M: usize> SubAssign<Matrix<U, N, M>> for Matrix<T, N, M>
where
    T: SubAssign<U>,
    U: Clone,
{
    fn sub_assign(&mut self, rhs: Matrix<U, N, M>) {
        for (i, j) in iproduct!(0..N, 0..M) {
            self[(i, j)] -= rhs[(i, j)].clone()
        }
    }
}

/// Some matrices can be negated
impl<T, const N: usize, const M: usize> Neg for Matrix<T, N, M>
where
    T: Neg,
{
    type Output = Matrix<<T as Neg>::Output, N, M>;

    fn neg(self) -> Self::Output {
        self.0.map(|row| row.map(Neg::neg)).into()
    }
}

/// Some matrices support dot products with other matrices
impl<T, U, const N: usize, const K: usize, const M: usize> Mul<Matrix<U, K, M>> for Matrix<T, N, K>
where
    <T as Mul<U>>::Output: Default + AddAssign<<T as Mul<U>>::Output>,
    T: Clone + Mul<U>,
    U: Clone,
{
    type Output = Matrix<<T as Mul<U>>::Output, N, M>;

    fn mul(self, rhs: Matrix<U, K, M>) -> Self::Output {
        let mut out = Matrix::<<T as Mul<U>>::Output, N, M>::default();
        for (i, j, k) in iproduct!(0..N, 0..M, 0..K) {
            out[(i, j)] += self[(i, k)].clone() * rhs[(k, j)].clone()
        }
        out
    }
}

/// A matrix with an elements, which supports Default, also supports Default
impl<T, const N: usize, const M: usize> Default for Matrix<T, N, M>
where
    T: Default,
{
    fn default() -> Self {
        [[MaybeUninit::<()>::uninit(); M]; N]
            .map(|row| row.map(|_| Default::default()))
            .into()
    }
}

/// A matrix with Copy elements is a Copy matrix
impl<T, const N: usize, const M: usize> Copy for Matrix<T, N, M>
where
    Self: Clone,
    T: Copy,
{
}

/// A matrix with Clone elements is a clone matrix
impl<T, const N: usize, const M: usize> Clone for Matrix<T, N, M>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.0.clone().into()
    }
}

/// A Matrix is indexed by a pair of unsigned numbers
impl<T, const N: usize, const M: usize> Index<(usize, usize)> for Matrix<T, N, M> {
    type Output = T;

    fn index(&self, (i, j): (usize, usize)) -> &Self::Output {
        &self.0[i][j]
    }
}

/// A column vector can be indexed by a single number
impl<T, const N: usize> Index<usize> for Matrix<T, N, 1> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i][0]
    }
}

/// A Matrix is indexed by a pair of unsigned numbers
impl<T, const N: usize, const M: usize> IndexMut<(usize, usize)> for Matrix<T, N, M> {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut Self::Output {
        &mut self.0[i][j]
    }
}

/// A Matrix can be borrowed as a 2D array
impl<T, const N: usize, const M: usize> Borrow<[[T; M]; N]> for Matrix<T, N, M> {
    fn borrow(&self) -> &[[T; M]; N] {
        &self.0
    }
}

/// A Matrix can be borrowed as a 2D array
impl<T, const N: usize, const M: usize> BorrowMut<[[T; M]; N]> for Matrix<T, N, M> {
    fn borrow_mut(&mut self) -> &mut [[T; M]; N] {
        &mut self.0
    }
}

/// A Matrix can be cast into a 2D array
impl<T, const N: usize, const M: usize> AsRef<[[T; M]; N]> for Matrix<T, N, M> {
    fn as_ref(&self) -> &[[T; M]; N] {
        &self.0
    }
}

/// A Matrix can be cast into a 2D array
impl<T, const N: usize, const M: usize> AsMut<[[T; M]; N]> for Matrix<T, N, M> {
    fn as_mut(&mut self) -> &mut [[T; M]; N] {
        &mut self.0
    }
}
