
use std::iter::zip;
use std::ops::{Mul, MulAssign, Add, Sub};
use std::cmp::PartialEq;
use std::fmt::Debug;

#[cfg(test)]
mod tests;
 
pub type F = f32;

pub trait Vector<T>: Debug {
    fn scale(&self, scalar: T) -> impl Vector<T>;
    fn size(&self) -> usize;
    fn elements(&self) -> impl Iterator<Item = T>;
}

// implement Vector properties for array slices so we can treat them like vectors!
// Should probably use macros seeing as these implementations are very similar...

impl<T, const N: usize> Vector<T> for [T; N] 
where T: Copy + MulAssign + Debug
{
    fn scale(&self, scalar: T) -> impl Vector<T> {
        let mut vec = self.to_vec();
        for elem in vec.iter_mut() {
            *elem *= scalar;
        }
        vec
    }

    fn size(&self) -> usize {
        N
    }

    fn elements(&self) -> impl Iterator<Item = T> {
        self.iter().copied()
    }
}

impl<T> Vector<T> for [T] 
where T: Copy + MulAssign + Debug
{
    fn scale(&self, scalar: T) -> impl Vector<T> {
        let mut vec = self.to_vec();
        for elem in vec.iter_mut() {
            *elem *= scalar;
        }
        vec
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn elements(&self) -> impl Iterator<Item = T> {
        self.iter().copied()
    }
}

impl<T> Vector<T> for Vec<T> 
where T: Copy + MulAssign + Debug {
    fn scale(&self, scalar: T) ->  impl Vector<T> {
        let mut vec = self.clone();
        for elem in vec.iter_mut() {
            *elem *= scalar;
        }
        vec
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn elements(&self) -> impl Iterator<Item = T> {
        self.iter().copied()
    }
}


#[derive(Debug, Clone)]
pub struct Matrix {
    values: Vec<F>,
    m: usize,
    n: usize,
}

impl Matrix {
    /// Get a new matrix, all values initialized to 0.
    pub fn new(m: usize, n: usize) -> Self {
        Self {
            values: vec![0.0; n * m],
            n,
            m,
        }
    }

    /// Get a new square matrix with values initialized to 0.
    pub fn new_square(n: usize) -> Self {
        Self {
            values: vec![0.0; n * n],
            n,
            m: n,
        }
    }

    /// Get a new, square identity matrix (i.e. main diagonal has 1s, 0s everywhere else).
    pub fn new_identity(n: usize) -> Self {
        let mut values = vec![0.0; n * n];
        for i in 0..n {
            values[i + n * i] = 1.0;
        }

        Self {
            values,
            n,
            m: n,
        }
    }
    
    /// Create a matrix that is the transpose of the input.
    pub fn to_transpose(&self) -> Self {
        let mut values = vec![0.0; self.n * self.m];
        let n = self.m;
        let m = self.n;
        for i in 0..m {
            for j in 0..n {
                values[j + self.m * i] = self.get_val(j, i).unwrap();
            }
        }

        Self {
            values,
            n: self.m,
            m: self.n,
        }
    }

    /// Create a matrix from raw array/slice/Vec.
    pub fn from_values(input: impl Vector<F>, m: usize, n: usize) -> Self {
        if n * m != input.size() {
            panic!("dimensions do not match array when constructing matrix.");
        }
        Self {
            values: input.elements().collect(),
            n,
            m,
        }
    }

    /// Create a matrix from column vectors.
    pub fn from_cols(input: Vec<impl Vector<F>>) -> Self {
        let n = input.len();
        let m = input[0].size();

        let mut values = vec![0.0; m * n];
            
        for (j, col) in input.iter().enumerate() {
            if col.size() != m {
                panic!("column length mismatch when constructing matrix.");
            }
            for (i, val) in col.elements().enumerate() {
                values[j + i * n] = val;
            }
        }

        Self {
            values,
            n,
            m,
        }
    }

    /// Create a matrix from row vectors.
    pub fn from_rows(input: Vec<impl Vector<F>>) -> Self {
        let m = input.len();
        let n = input[0].size();

        let mut values: Vec<F> = Vec::new();
        values.reserve_exact(m * n);

        for row in input.iter() {
            values.extend(row.elements());
        }

        Self {
            values,
            n,
            m,
        }
    }

    pub fn m(&self) -> usize {
        self.m
    }

    pub fn n(&self) -> usize {
        self.n
    }

    /// Get the value at some position in the matrix. Returns `None` if input is out of bounds.
    pub fn get_val(&self, i: usize, j: usize) -> Option<F> {
        self.values.get(self.n * i + j).copied()
    }

    pub fn iter_row(&self, i: usize) -> impl Iterator<Item = F> + '_ {
        self.values[(self.n * i)..(self.n * (i + 1))].iter().copied()
    }

    pub fn iter_col(&self, j: usize) -> impl Iterator<Item = F> + '_ {
        MatrixColIter::new(&self, j)
    }

    pub fn get_row(&self, i: usize) -> Vec<F> {
        let mut row_vec = Vec::new();
        row_vec.extend_from_slice(&self.values[self.n * i..self.n * (i + 1)]);
        row_vec
    }

    pub fn get_row_slice(&self, i: usize) -> &[F] {
        &self.values[self.n * i..self.n * (i + 1)]
    }

    pub fn get_mut_row_slice(&mut self, i: usize) -> &mut [F] {
        &mut self.values[self.n * i..self.n * (i + 1)]
    }

    pub fn get_col(&self, j: usize) -> Vec<F> {
        (0..self.m).map(|i| self.values[j + i * self.n]).collect()
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = Vec<F>> + use<'_> {
        (0..self.m).map(|i| self.get_row(i))
    }

    pub fn iter_row_slices(&self) -> impl Iterator<Item = &[F]> {
        (0..self.m).map(|i| self.get_row_slice(i))
    }

    pub fn iter_cols(&self) -> impl Iterator<Item = Vec<F>> + use<'_> {
        (0..self.n).map(|j| self.get_col(j))
    }

    /// Gives the raw underlying vector which represents the matrix with contiguous rows.
    pub fn get_raw_values(&self) -> Vec<F> {
        self.values.clone()
    }

    pub fn set_row(&mut self, m: usize, new_row: impl Vector<F>) {
        assert!(new_row.size() == self.n);
        self.values.splice((m * self.n)..((m + 1) * self.n), new_row.elements());
    }

    pub fn set_value(&mut self, val: F, i: usize, j: usize) {
        self.values[i * self.n + j] = val;
    }

    pub fn apply_fn<A>(&mut self, f: A) 
    where A: FnMut(&mut F),
    {
        self.values.iter_mut().for_each(f);
    }

    pub fn to_apply_fn<A>(&self, f: A) -> Self
    where A: FnMut(&F) -> F,
    {
        Self::from_values(self.values.iter().map(f).collect::<Vec<_>>(), self.n, self.m)
    }
}


impl Mul<&Matrix> for &Matrix {
    type Output = Matrix;

    fn mul(self, rhs: &Matrix) -> Matrix {
        if self.n != rhs.m {
            panic!("Dimension mismatch when trying to multiply matrices!");
        }

        let mut out = Matrix::new(self.m, rhs.n);
        for (i, self_row) in self.iter_rows().enumerate() {
            for (j, rhs_col) in rhs.iter_cols().enumerate() {
                out.set_value(dot(&self_row, &rhs_col).unwrap(), i, j);
            }
        }
        out
    }
}


impl Sub<&Matrix> for &Matrix {
    type Output = Matrix;

    fn sub(self, rhs: &Matrix) -> Matrix {
        if self.m != rhs.m || self.n != rhs.n {
            panic!("Dimensinon mismatch when trying to multiply matrices!");
        }

        let mut values = Vec::new();
        values.reserve_exact(self.values.len());

        for (e1, e2) in zip(self.values.iter(), rhs.values.iter()) {
            values.push(e1 - e2);
        }

        Matrix {
            values,
            m: self.m,
            n: self.n,
        }
    }
}

impl Add<&Matrix> for &Matrix {
    type Output = Matrix;

    fn add(self, rhs: &Matrix) -> Matrix {
        if self.m != rhs.m || self.n != rhs.n {
            panic!("Dimensinon mismatch when trying to multiply matrices!");
        }

        let mut values = Vec::new();
        values.reserve_exact(self.values.len());

        for (e1, e2) in zip(self.values.iter(), rhs.values.iter()) {
            values.push(e1 + e2);
        }

        Matrix {
            values,
            m: self.m,
            n: self.n,
        }
    }
}

impl<T: Vector<F>> Mul<&T> for &Matrix {
    type Output = Vec<F>;

    fn mul(self, rhs: &T) -> Vec<F> { 
        let mut output = Vec::new();
        output.reserve_exact(self.m);

        for row in self.iter_row_slices() {
            output.push(dot(row, rhs).unwrap());
        }

        output
    }
}

impl Mul<F> for &Matrix {
    type Output = Matrix;

    fn mul(self, rhs: F) -> Matrix {
        Matrix {
            values: self.values.iter().map(|x| x * rhs).collect(),
            m: self.m,
            n: self.n,
        }
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        if self.m != other.m || self.n != other.n {
            return false;
        }
        for (i, s_val) in self.values.iter().enumerate() {
            if *s_val != other.values[i] {
                return false;
            }
        }

        true
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

// iterates over the values of a matrix column.
pub struct MatrixColIter<'a> {
    matrix_ref: &'a Matrix,
    col: usize,
    current_row: usize,
}

impl<'a> MatrixColIter<'a> {
    pub fn new(matrix_ref: &'a Matrix, col: usize) -> Self {
        Self {
            matrix_ref,
            col,
            current_row: 0,
        }
    }
}

impl Iterator for MatrixColIter<'_> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.matrix_ref.get_val(self.current_row, self.col);
        self.current_row += 1;
        out
    }
}



pub fn sigmoid(x: F) -> F {
    1.0 / (1.0 + (-x).exp())
}

pub fn dot<U, V>(u: &U, v: &V) -> Option<F>
where V: Vector<F> + ?Sized,
      U: Vector<F> + ?Sized, {

    if u.size() != v.size() {
        return None;
    }

    let mut sum: F = 0.0;

    for (u_val, v_val) in zip(u.elements(), v.elements()) {
        sum += u_val * v_val;
    }

    Some(sum)
}

