use crate::{
    diag::DiagMut,
    mat::{self, As2D, Mat, MatMut},
    row::RowMut,
    unzipped, zipped,
};

use super::*;

#[repr(C)]
pub struct ColMut<'a, E: Entity> {
    pub(super) inner: VecImpl<E>,
    pub(super) __marker: PhantomData<&'a E>,
}

impl<'short, E: Entity> Reborrow<'short> for ColMut<'_, E> {
    type Target = ColRef<'short, E>;

    #[inline]
    fn rb(&'short self) -> Self::Target {
        ColRef {
            inner: self.inner,
            __marker: PhantomData,
        }
    }
}

impl<'short, E: Entity> ReborrowMut<'short> for ColMut<'_, E> {
    type Target = ColMut<'short, E>;

    #[inline]
    fn rb_mut(&'short mut self) -> Self::Target {
        ColMut {
            inner: self.inner,
            __marker: PhantomData,
        }
    }
}

impl<'a, E: Entity> IntoConst for ColMut<'a, E> {
    type Target = ColRef<'a, E>;

    #[inline]
    fn into_const(self) -> Self::Target {
        ColRef {
            inner: self.inner,
            __marker: PhantomData,
        }
    }
}

impl<'a, E: Entity> ColMut<'a, E> {
    #[inline]
    pub(crate) unsafe fn __from_raw_parts(
        ptr: GroupFor<E, *mut E::Unit>,
        nrows: usize,
        row_stride: isize,
    ) -> Self {
        Self {
            inner: VecImpl {
                ptr: into_copy::<E, _>(E::faer_map(
                    ptr,
                    #[inline]
                    |ptr| NonNull::new_unchecked(ptr),
                )),
                len: nrows,
                stride: row_stride,
            },
            __marker: PhantomData,
        }
    }

    #[track_caller]
    #[inline(always)]
    #[doc(hidden)]
    pub fn try_get_contiguous_col_mut(self) -> GroupFor<E, &'a mut [E::Unit]> {
        assert!(self.row_stride() == 1);
        let m = self.nrows();
        E::faer_map(
            self.as_ptr_mut(),
            #[inline(always)]
            |ptr| unsafe { core::slice::from_raw_parts_mut(ptr, m) },
        )
    }

    /// Returns the number of rows of the column.
    #[inline(always)]
    pub fn nrows(&self) -> usize {
        self.inner.len
    }
    /// Returns the number of columns of the column. This is always equal to `1`.
    #[inline(always)]
    pub fn ncols(&self) -> usize {
        1
    }

    /// Returns pointers to the matrix data.
    #[inline(always)]
    pub fn as_ptr_mut(self) -> GroupFor<E, *mut E::Unit> {
        E::faer_map(
            from_copy::<E, _>(self.inner.ptr),
            #[inline(always)]
            |ptr| ptr.as_ptr() as *mut E::Unit,
        )
    }

    /// Returns the row stride of the matrix, specified in number of elements, not in bytes.
    #[inline(always)]
    pub fn row_stride(&self) -> isize {
        self.inner.stride
    }

    /// Returns `self` as a mutable matrix view.
    #[inline(always)]
    pub fn as_2d_mut(self) -> MatMut<'a, E> {
        let nrows = self.nrows();
        let row_stride = self.row_stride();
        unsafe { mat::from_raw_parts_mut(self.as_ptr_mut(), nrows, 1, row_stride, isize::MAX) }
    }

    /// Returns raw pointers to the element at the given index.
    #[inline(always)]
    pub fn ptr_at_mut(self, row: usize) -> GroupFor<E, *mut E::Unit> {
        let offset = (row as isize).wrapping_mul(self.inner.stride);

        E::faer_map(
            self.as_ptr_mut(),
            #[inline(always)]
            |ptr| ptr.wrapping_offset(offset),
        )
    }

    #[inline(always)]
    unsafe fn ptr_at_mut_unchecked(self, row: usize) -> GroupFor<E, *mut E::Unit> {
        let offset = crate::utils::unchecked_mul(row, self.inner.stride);
        E::faer_map(
            self.as_ptr_mut(),
            #[inline(always)]
            |ptr| ptr.offset(offset),
        )
    }

    /// Returns raw pointers to the element at the given index, assuming the provided index
    /// is within the size of the vector.
    ///
    /// # Safety
    /// The behavior is undefined if any of the following conditions are violated:
    /// * `row < self.nrows()`.
    #[inline(always)]
    #[track_caller]
    pub unsafe fn ptr_inbounds_at_mut(self, row: usize) -> GroupFor<E, *mut E::Unit> {
        debug_assert!(row < self.nrows());
        self.ptr_at_mut_unchecked(row)
    }

    /// Splits the column vector at the given index into two parts and
    /// returns an array of each subvector, in the following order:
    /// * top.
    /// * bottom.
    ///
    /// # Safety
    /// The behavior is undefined if any of the following conditions are violated:
    /// * `row <= self.nrows()`.
    #[inline(always)]
    #[track_caller]
    pub unsafe fn split_at_mut_unchecked(self, row: usize) -> (Self, Self) {
        let (top, bot) = self.into_const().split_at_unchecked(row);
        unsafe { (top.const_cast(), bot.const_cast()) }
    }

    /// Splits the column vector at the given index into two parts and
    /// returns an array of each subvector, in the following order:
    /// * top.
    /// * bottom.
    ///
    /// # Panics
    /// The function panics if any of the following conditions are violated:
    /// * `row <= self.nrows()`.
    #[inline(always)]
    #[track_caller]
    pub fn split_at_mut(self, row: usize) -> (Self, Self) {
        assert!(row <= self.nrows());
        unsafe { self.split_at_mut_unchecked(row) }
    }

    /// Returns references to the element at the given index, or subvector if `row` is a
    /// range.
    ///
    /// # Note
    /// The values pointed to by the references are expected to be initialized, even if the
    /// pointed-to value is not read, otherwise the behavior is undefined.
    ///
    /// # Safety
    /// The behavior is undefined if any of the following conditions are violated:
    /// * `row` must be contained in `[0, self.nrows())`.
    #[inline(always)]
    #[track_caller]
    pub unsafe fn get_unchecked_mut<RowRange>(
        self,
        row: RowRange,
    ) -> <Self as ColIndex<RowRange>>::Target
    where
        Self: ColIndex<RowRange>,
    {
        <Self as ColIndex<RowRange>>::get_unchecked(self, row)
    }

    /// Returns references to the element at the given index, or subvector if `row` is a
    /// range, with bound checks.
    ///
    /// # Note
    /// The values pointed to by the references are expected to be initialized, even if the
    /// pointed-to value is not read, otherwise the behavior is undefined.
    ///
    /// # Panics
    /// The function panics if any of the following conditions are violated:
    /// * `row` must be contained in `[0, self.nrows())`.
    #[inline(always)]
    #[track_caller]
    pub fn get_mut<RowRange>(self, row: RowRange) -> <Self as ColIndex<RowRange>>::Target
    where
        Self: ColIndex<RowRange>,
    {
        <Self as ColIndex<RowRange>>::get(self, row)
    }

    /// Reads the value of the element at the given index.
    ///
    /// # Safety
    /// The behavior is undefined if any of the following conditions are violated:
    /// * `row < self.nrows()`.
    #[inline(always)]
    #[track_caller]
    pub unsafe fn read_unchecked(&self, row: usize) -> E {
        self.rb().read_unchecked(row)
    }

    /// Reads the value of the element at the given index, with bound checks.
    ///
    /// # Panics
    /// The function panics if any of the following conditions are violated:
    /// * `row < self.nrows()`.
    #[inline(always)]
    #[track_caller]
    pub fn read(&self, row: usize) -> E {
        self.rb().read(row)
    }

    /// Writes the value to the element at the given index.
    ///
    /// # Safety
    /// The behavior is undefined if any of the following conditions are violated:
    /// * `row < self.nrows()`.
    #[inline(always)]
    #[track_caller]
    pub unsafe fn write_unchecked(&mut self, row: usize, value: E) {
        let units = value.faer_into_units();
        let zipped = E::faer_zip(units, (*self).rb_mut().ptr_inbounds_at_mut(row));
        E::faer_map(
            zipped,
            #[inline(always)]
            |(unit, ptr)| *ptr = unit,
        );
    }

    /// Writes the value to the element at the given index, with bound checks.
    ///
    /// # Panics
    /// The function panics if any of the following conditions are violated:
    /// * `row < self.nrows()`.
    #[inline(always)]
    #[track_caller]
    pub fn write(&mut self, row: usize, value: E) {
        assert!(row < self.nrows());
        unsafe { self.write_unchecked(row, value) };
    }

    /// Copies the values from `other` into `self`.
    ///
    /// # Panics
    /// The function panics if any of the following conditions are violated:
    /// * `self.nrows() == other.nrows()`.
    /// * `self.ncols() == other.ncols()`.
    #[track_caller]
    pub fn copy_from(&mut self, other: impl AsColRef<E>) {
        #[track_caller]
        #[inline(always)]
        fn implementation<E: Entity>(this: ColMut<'_, E>, other: ColRef<'_, E>) {
            zipped!(this.as_2d_mut(), other.as_2d())
                .for_each(|unzipped!(mut dst, src)| dst.write(src.read()));
        }
        implementation(self.rb_mut(), other.as_col_ref())
    }

    /// Fills the elements of `self` with zeros.
    #[track_caller]
    pub fn fill_zero(&mut self)
    where
        E: ComplexField,
    {
        zipped!(self.rb_mut().as_2d_mut()).for_each(
            #[inline(always)]
            |unzipped!(mut x)| x.write(E::faer_zero()),
        );
    }

    /// Fills the elements of `self` with copies of `constant`.
    #[track_caller]
    pub fn fill(&mut self, constant: E) {
        zipped!((*self).rb_mut().as_2d_mut()).for_each(
            #[inline(always)]
            |unzipped!(mut x)| x.write(constant),
        );
    }

    /// Returns a view over the transpose of `self`.
    #[inline(always)]
    #[must_use]
    pub fn transpose_mut(self) -> RowMut<'a, E> {
        unsafe { self.into_const().transpose().const_cast() }
    }

    /// Returns a view over the conjugate of `self`.
    #[inline(always)]
    #[must_use]
    pub fn conjugate_mut(self) -> ColMut<'a, E::Conj>
    where
        E: Conjugate,
    {
        unsafe { self.into_const().conjugate().const_cast() }
    }

    /// Returns a view over the conjugate transpose of `self`.
    #[inline(always)]
    pub fn adjoint_mut(self) -> RowMut<'a, E::Conj>
    where
        E: Conjugate,
    {
        self.conjugate_mut().transpose_mut()
    }

    /// Returns a view over the canonical representation of `self`, as well as a flag declaring
    /// whether `self` is implicitly conjugated or not.
    #[inline(always)]
    pub fn canonicalize_mut(self) -> (ColMut<'a, E::Canonical>, Conj)
    where
        E: Conjugate,
    {
        let (canon, conj) = self.into_const().canonicalize();
        unsafe { (canon.const_cast(), conj) }
    }

    /// Returns a view over the `self`, with the rows in reversed order.
    #[inline(always)]
    #[must_use]
    pub fn reverse_rows_mut(self) -> Self {
        unsafe { self.into_const().reverse_rows().const_cast() }
    }

    /// Returns a view over the subvector starting at row `row_start`, and with number of rows
    /// `nrows`.
    ///
    /// # Safety
    /// The behavior is undefined if any of the following conditions are violated:
    /// * `row_start <= self.nrows()`.
    /// * `nrows <= self.nrows() - row_start`.
    #[track_caller]
    #[inline(always)]
    pub unsafe fn subrows_mut_unchecked(self, row_start: usize, nrows: usize) -> Self {
        self.into_const()
            .subrows_unchecked(row_start, nrows)
            .const_cast()
    }

    /// Returns a view over the subvector starting at row `row_start`, and with number of rows
    /// `nrows`.
    ///
    /// # Safety
    /// The behavior is undefined if any of the following conditions are violated:
    /// * `row_start <= self.nrows()`.
    /// * `nrows <= self.nrows() - row_start`.
    #[track_caller]
    #[inline(always)]
    pub fn subrows_mut(self, row_start: usize, nrows: usize) -> Self {
        unsafe { self.into_const().subrows(row_start, nrows).const_cast() }
    }

    /// Given a matrix with a single column, returns an object that interprets
    /// the column as a diagonal matrix, whoes diagonal elements are values in the column.
    #[track_caller]
    #[inline(always)]
    pub fn column_vector_as_diagonal_mut(self) -> DiagMut<'a, E> {
        DiagMut { inner: self }
    }

    /// Returns an owning [`Col`] of the data.
    #[inline]
    pub fn to_owned(&self) -> Col<E::Canonical>
    where
        E: Conjugate,
    {
        (*self).rb().to_owned()
    }

    /// Returns `true` if any of the elements is NaN, otherwise returns `false`.
    #[inline]
    pub fn has_nan(&self) -> bool
    where
        E: ComplexField,
    {
        (*self).rb().as_2d().has_nan()
    }

    /// Returns `true` if all of the elements are finite, otherwise returns `false`.
    #[inline]
    pub fn is_all_finite(&self) -> bool
    where
        E: ComplexField,
    {
        (*self).rb().as_2d().is_all_finite()
    }

    /// Returns the maximum norm of `self`.
    #[inline]
    pub fn norm_max(&self) -> E::Real
    where
        E: ComplexField,
    {
        self.rb().as_2d().norm_max()
    }
    /// Returns the L2 norm of `self`.
    #[inline]
    pub fn norm_l2(&self) -> E::Real
    where
        E: ComplexField,
    {
        self.rb().as_2d().norm_l2()
    }

    /// Returns the sum of `self`.
    #[inline]
    pub fn sum(&self) -> E
    where
        E: ComplexField,
    {
        self.rb().as_2d().sum()
    }

    /// Kroneckor product of `self` and `rhs`.
    ///
    /// This is an allocating operation; see [`kron`] for the
    /// allocation-free version or more info in general.
    #[inline]
    #[track_caller]
    pub fn kron(&self, rhs: impl As2D<E>) -> Mat<E>
    where
        E: ComplexField,
    {
        self.as_ref().kron(rhs)
    }

    /// Returns a view over the matrix.
    #[inline]
    pub fn as_ref(&self) -> ColRef<'_, E> {
        (*self).rb()
    }

    /// Returns a mutable view over the matrix.
    #[inline]
    pub fn as_mut(&mut self) -> ColMut<'_, E> {
        (*self).rb_mut()
    }
}
