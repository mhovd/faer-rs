use super::{AsColRef, ColIndex, ColView};
use crate::internal_prelude::*;
use crate::mat::matmut::SyncCell;
use crate::utils::bound::{Array, Dim, Partition};
use crate::{ContiguousFwd, Idx, IdxInc};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::ptr::NonNull;
use equator::assert;
use faer_traits::Real;
use generativity::Guard;

pub struct ColMut<'a, T, Rows = usize, RStride = isize> {
	pub(super) imp: ColView<T, Rows, RStride>,
	pub(super) __marker: PhantomData<(&'a mut T, &'a Rows)>,
}

impl<'short, T, Rows: Copy, RStride: Copy> Reborrow<'short> for ColMut<'_, T, Rows, RStride> {
	type Target = ColRef<'short, T, Rows, RStride>;

	#[inline]
	fn rb(&'short self) -> Self::Target {
		ColRef { imp: self.imp, __marker: PhantomData }
	}
}
impl<'short, T, Rows: Copy, RStride: Copy> ReborrowMut<'short> for ColMut<'_, T, Rows, RStride> {
	type Target = ColMut<'short, T, Rows, RStride>;

	#[inline]
	fn rb_mut(&'short mut self) -> Self::Target {
		ColMut { imp: self.imp, __marker: PhantomData }
	}
}
impl<'a, T, Rows: Copy, RStride: Copy> IntoConst for ColMut<'a, T, Rows, RStride> {
	type Target = ColRef<'a, T, Rows, RStride>;

	#[inline]
	fn into_const(self) -> Self::Target {
		ColRef { imp: self.imp, __marker: PhantomData }
	}
}

unsafe impl<T: Sync, Rows: Sync, RStride: Sync> Sync for ColMut<'_, T, Rows, RStride> {}
unsafe impl<T: Send, Rows: Send, RStride: Send> Send for ColMut<'_, T, Rows, RStride> {}

impl<'a, T> ColMut<'a, T> {
	#[inline]
	pub fn from_slice_mut(slice: &'a mut [T]) -> Self {
		let len = slice.len();
		unsafe { Self::from_raw_parts_mut(slice.as_mut_ptr(), len, 1) }
	}
}

impl<'a, T, Rows: Shape, RStride: Stride> ColMut<'a, T, Rows, RStride> {
	#[inline(always)]
	#[track_caller]
	pub unsafe fn from_raw_parts_mut(ptr: *mut T, nrows: Rows, row_stride: RStride) -> Self {
		Self {
			imp: ColView {
				ptr: NonNull::new_unchecked(ptr),
				nrows,
				row_stride,
			},
			__marker: PhantomData,
		}
	}

	#[inline(always)]
	pub fn as_ptr(&self) -> *const T {
		self.rb().as_ptr()
	}

	#[inline(always)]
	pub fn nrows(&self) -> Rows {
		self.imp.nrows
	}

	#[inline(always)]
	pub fn ncols(&self) -> usize {
		1
	}

	#[inline(always)]
	pub fn shape(&self) -> (Rows, usize) {
		(self.nrows(), self.ncols())
	}

	#[inline(always)]
	pub fn row_stride(&self) -> RStride {
		self.imp.row_stride
	}

	#[inline(always)]
	pub fn ptr_at(&self, row: IdxInc<Rows>) -> *const T {
		self.rb().ptr_at(row)
	}

	#[inline(always)]
	#[track_caller]
	pub unsafe fn ptr_inbounds_at(&self, row: Idx<Rows>) -> *const T {
		self.rb().ptr_inbounds_at(row)
	}

	#[inline]
	#[track_caller]
	pub fn split_at_row(self, row: IdxInc<Rows>) -> (ColRef<'a, T, usize, RStride>, ColRef<'a, T, usize, RStride>) {
		self.into_const().split_at_row(row)
	}

	#[inline(always)]
	pub fn transpose(self) -> RowRef<'a, T, Rows, RStride> {
		self.into_const().transpose()
	}

	#[inline(always)]
	pub fn conjugate(self) -> ColRef<'a, T::Conj, Rows, RStride>
	where
		T: Conjugate,
	{
		self.into_const().conjugate()
	}

	#[inline(always)]
	pub fn canonical(self) -> ColRef<'a, T::Canonical, Rows, RStride>
	where
		T: Conjugate,
	{
		self.into_const().canonical()
	}

	#[inline(always)]
	pub fn adjoint(self) -> RowRef<'a, T::Conj, Rows, RStride>
	where
		T: Conjugate,
	{
		self.into_const().adjoint()
	}

	#[track_caller]
	#[inline(always)]
	pub fn get<RowRange>(self, row: RowRange) -> <ColRef<'a, T, Rows, RStride> as ColIndex<RowRange>>::Target
	where
		ColRef<'a, T, Rows, RStride>: ColIndex<RowRange>,
	{
		<ColRef<'a, T, Rows, RStride> as ColIndex<RowRange>>::get(self.into_const(), row)
	}

	#[track_caller]
	#[inline(always)]
	pub unsafe fn get_unchecked<RowRange>(self, row: RowRange) -> <ColRef<'a, T, Rows, RStride> as ColIndex<RowRange>>::Target
	where
		ColRef<'a, T, Rows, RStride>: ColIndex<RowRange>,
	{
		unsafe { <ColRef<'a, T, Rows, RStride> as ColIndex<RowRange>>::get_unchecked(self.into_const(), row) }
	}

	#[inline]
	pub fn reverse_rows(self) -> ColRef<'a, T, Rows, RStride::Rev> {
		self.into_const().reverse_rows()
	}

	#[inline]
	pub fn subrows<V: Shape>(self, row_start: IdxInc<Rows>, nrows: V) -> ColRef<'a, T, V, RStride> {
		self.into_const().subrows(row_start, nrows)
	}

	#[inline]
	#[track_caller]
	pub fn as_row_shape<V: Shape>(self, nrows: V) -> ColRef<'a, T, V, RStride> {
		self.into_const().as_row_shape(nrows)
	}

	#[inline]
	pub fn as_dyn_rows(self) -> ColRef<'a, T, usize, RStride> {
		self.into_const().as_dyn_rows()
	}

	#[inline]
	pub fn as_dyn_stride(self) -> ColRef<'a, T, Rows, isize> {
		self.into_const().as_dyn_stride()
	}

	#[inline]
	pub fn iter(self) -> impl 'a + ExactSizeIterator + DoubleEndedIterator<Item = &'a T> {
		self.into_const().iter()
	}

	#[inline]
	#[cfg(feature = "rayon")]
	pub fn par_iter(self) -> impl 'a + rayon::iter::IndexedParallelIterator<Item = &'a T>
	where
		T: Sync,
	{
		self.into_const().par_iter()
	}

	#[inline]
	#[track_caller]
	#[cfg(feature = "rayon")]
	pub fn par_partition(self, count: usize) -> impl 'a + rayon::iter::IndexedParallelIterator<Item = ColRef<'a, T, usize, RStride>>
	where
		T: Sync,
	{
		self.into_const().par_partition(count)
	}

	#[inline]
	pub fn try_as_col_major(self) -> Option<ColRef<'a, T, Rows, ContiguousFwd>> {
		self.into_const().try_as_col_major()
	}

	#[inline]
	pub fn try_as_col_major_mut(self) -> Option<ColMut<'a, T, Rows, ContiguousFwd>> {
		self.into_const().try_as_col_major().map(|x| unsafe { x.const_cast() })
	}

	#[inline(always)]
	pub unsafe fn const_cast(self) -> ColMut<'a, T, Rows, RStride> {
		self
	}

	#[inline]
	pub fn as_ref(&self) -> ColRef<'_, T, Rows, RStride> {
		self.rb()
	}

	#[inline]
	pub fn as_mut(&mut self) -> ColMut<'_, T, Rows, RStride> {
		self.rb_mut()
	}

	#[inline]
	pub fn bind_r<'N>(self, row: Guard<'N>) -> ColMut<'a, T, Dim<'N>, RStride> {
		unsafe { ColMut::from_raw_parts_mut(self.as_ptr_mut(), self.nrows().bind(row), self.row_stride()) }
	}

	#[inline]
	pub fn as_mat(self) -> MatRef<'a, T, Rows, usize, RStride, isize> {
		self.into_const().as_mat()
	}

	#[inline]
	pub fn as_mat_mut(self) -> MatMut<'a, T, Rows, usize, RStride, isize> {
		unsafe { self.into_const().as_mat().const_cast() }
	}

	#[inline]
	pub fn as_diagonal(self) -> DiagRef<'a, T, Rows, RStride> {
		DiagRef { inner: self.into_const() }
	}

	#[inline]
	pub fn norm_max(&self) -> Real<T>
	where
		T: Conjugate,
	{
		self.rb().norm_max()
	}

	#[inline]
	pub fn norm_l2(&self) -> Real<T>
	where
		T: Conjugate,
	{
		self.rb().norm_l2()
	}
}

impl<'a, T, Rows: Shape, RStride: Stride> ColMut<'a, T, Rows, RStride> {
	#[inline(always)]
	pub fn as_ptr_mut(&self) -> *mut T {
		self.rb().as_ptr() as *mut T
	}

	#[inline(always)]
	pub fn ptr_at_mut(&self, row: IdxInc<Rows>) -> *mut T {
		self.rb().ptr_at(row) as *mut T
	}

	#[inline(always)]
	#[track_caller]
	pub unsafe fn ptr_inbounds_at_mut(&self, row: Idx<Rows>) -> *mut T {
		self.rb().ptr_inbounds_at(row) as *mut T
	}

	#[inline]
	#[track_caller]
	pub fn split_at_row_mut(self, row: IdxInc<Rows>) -> (ColMut<'a, T, usize, RStride>, ColMut<'a, T, usize, RStride>) {
		let (a, b) = self.into_const().split_at_row(row);
		unsafe { (a.const_cast(), b.const_cast()) }
	}

	#[inline(always)]
	pub fn transpose_mut(self) -> RowMut<'a, T, Rows, RStride> {
		unsafe { self.into_const().transpose().const_cast() }
	}

	#[inline(always)]
	pub fn conjugate_mut(self) -> ColMut<'a, T::Conj, Rows, RStride>
	where
		T: Conjugate,
	{
		unsafe { self.into_const().conjugate().const_cast() }
	}

	#[inline(always)]
	pub fn canonical_mut(self) -> ColMut<'a, T::Canonical, Rows, RStride>
	where
		T: Conjugate,
	{
		unsafe { self.into_const().canonical().const_cast() }
	}

	#[inline(always)]
	pub fn adjoint_mut(self) -> RowMut<'a, T::Conj, Rows, RStride>
	where
		T: Conjugate,
	{
		unsafe { self.into_const().adjoint().const_cast() }
	}

	#[inline(always)]
	#[track_caller]
	pub(crate) fn at_mut(self, row: Idx<Rows>) -> &'a mut T {
		assert!(all(row < self.nrows()));
		unsafe { self.at_mut_unchecked(row) }
	}

	#[inline(always)]
	#[track_caller]
	pub(crate) unsafe fn at_mut_unchecked(self, row: Idx<Rows>) -> &'a mut T {
		&mut *self.ptr_inbounds_at_mut(row)
	}

	#[track_caller]
	#[inline(always)]
	pub fn get_mut<RowRange>(self, row: RowRange) -> <ColMut<'a, T, Rows, RStride> as ColIndex<RowRange>>::Target
	where
		ColMut<'a, T, Rows, RStride>: ColIndex<RowRange>,
	{
		<ColMut<'a, T, Rows, RStride> as ColIndex<RowRange>>::get(self, row)
	}

	#[track_caller]
	#[inline(always)]
	pub unsafe fn get_mut_unchecked<RowRange>(self, row: RowRange) -> <ColMut<'a, T, Rows, RStride> as ColIndex<RowRange>>::Target
	where
		ColMut<'a, T, Rows, RStride>: ColIndex<RowRange>,
	{
		unsafe { <ColMut<'a, T, Rows, RStride> as ColIndex<RowRange>>::get_unchecked(self, row) }
	}

	#[inline]
	pub fn reverse_rows_mut(self) -> ColMut<'a, T, Rows, RStride::Rev> {
		unsafe { self.into_const().reverse_rows().const_cast() }
	}

	#[inline]
	#[track_caller]
	pub fn subrows_mut<V: Shape>(self, row_start: IdxInc<Rows>, nrows: V) -> ColMut<'a, T, V, RStride> {
		unsafe { self.into_const().subrows(row_start, nrows).const_cast() }
	}

	#[inline]
	#[track_caller]
	pub fn as_row_shape_mut<V: Shape>(self, nrows: V) -> ColMut<'a, T, V, RStride> {
		unsafe { self.into_const().as_row_shape(nrows).const_cast() }
	}

	#[inline]
	pub fn as_dyn_rows_mut(self) -> ColMut<'a, T, usize, RStride> {
		unsafe { self.into_const().as_dyn_rows().const_cast() }
	}

	#[inline]
	pub fn as_dyn_stride_mut(self) -> ColMut<'a, T, Rows, isize> {
		unsafe { self.into_const().as_dyn_stride().const_cast() }
	}

	#[inline]
	pub fn iter_mut(self) -> impl 'a + ExactSizeIterator + DoubleEndedIterator<Item = &'a mut T> {
		let this = self.into_const();
		Rows::indices(Rows::start(), this.nrows().end()).map(move |j| unsafe { this.const_cast().at_mut_unchecked(j) })
	}

	#[inline]
	#[cfg(feature = "rayon")]
	pub fn par_iter_mut(self) -> impl 'a + rayon::iter::IndexedParallelIterator<Item = &'a mut T>
	where
		T: Send,
	{
		unsafe {
			let this = self.as_type::<SyncCell<T>>().into_const();

			use rayon::prelude::*;
			(0..this.nrows().unbound()).into_par_iter().map(move |j| {
				let ptr = this.const_cast().at_mut_unchecked(Idx::<Rows>::new_unbound(j));
				&mut *(ptr as *mut SyncCell<T> as *mut T)
			})
		}
	}

	#[inline]
	#[track_caller]
	#[cfg(feature = "rayon")]
	pub fn par_partition_mut(self, count: usize) -> impl 'a + rayon::iter::IndexedParallelIterator<Item = ColMut<'a, T, usize, RStride>>
	where
		T: Send,
	{
		use rayon::prelude::*;
		unsafe { self.as_type::<SyncCell<T>>().into_const().par_partition(count).map(|col| col.const_cast().as_type::<T>()) }
	}

	pub(crate) unsafe fn as_type<U>(self) -> ColMut<'a, U, Rows, RStride> {
		ColMut::from_raw_parts_mut(self.as_ptr_mut() as *mut U, self.nrows(), self.row_stride())
	}

	#[inline]
	pub fn as_diagonal_mut(self) -> DiagMut<'a, T, Rows, RStride> {
		DiagMut { inner: self }
	}

	#[inline]
	pub fn copy_from<RhsT: Conjugate<Canonical = T>>(&mut self, other: impl AsColRef<T = RhsT, Rows = Rows>)
	where
		T: ComplexField,
	{
		let other = other.as_col_ref();

		assert!(all(self.nrows() == other.nrows(), self.ncols() == other.ncols(),));
		let m = self.nrows();

		with_dim!(M, m.unbound());
		imp(self.rb_mut().as_row_shape_mut(M).as_dyn_stride_mut(), other.as_row_shape(M).canonical(), Conj::get::<RhsT>());

		pub fn imp<'M, 'N, T: ComplexField>(this: ColMut<'_, T, Dim<'M>>, other: ColRef<'_, T, Dim<'M>>, conj_: Conj) {
			match conj_ {
				Conj::No => {
					zipped!(this, other).for_each(|unzipped!(dst, src)| *dst = copy(&src));
				},
				Conj::Yes => {
					zipped!(this, other).for_each(|unzipped!(dst, src)| *dst = conj(&src));
				},
			}
		}
	}

	#[inline]
	pub fn fill(&mut self, value: T)
	where
		T: Clone,
	{
		fn cloner<T: Clone>(value: T) -> impl for<'a> FnMut(crate::linalg::zip::Last<&'a mut T>) {
			#[inline(always)]
			move |x| *x.0 = value.clone()
		}
		z!(self.rb_mut().as_dyn_rows_mut()).for_each(cloner::<T>(value));
	}

	#[inline(always)]
	#[track_caller]
	pub fn read(&self, row: Idx<Rows>) -> T
	where
		T: Clone,
	{
		self.rb().read(row)
	}

	#[inline]
	pub fn write(&mut self, i: Idx<Rows>, value: T) {
		*self.rb_mut().at_mut(i) = value;
	}

	#[inline]
	#[track_caller]
	pub(crate) fn __at_mut(self, i: Idx<Rows>) -> &'a mut T {
		self.at_mut(i)
	}
}

impl<'a, T, Rows: Shape> ColMut<'a, T, Rows, ContiguousFwd> {
	#[inline]
	pub fn as_slice_mut(self) -> &'a mut [T] {
		unsafe { core::slice::from_raw_parts_mut(self.as_ptr_mut(), self.nrows().unbound()) }
	}
}

impl<'a, 'ROWS, T> ColMut<'a, T, Dim<'ROWS>, ContiguousFwd> {
	#[inline]
	pub fn as_array_mut(self) -> &'a mut Array<'ROWS, T> {
		unsafe { &mut *(self.as_slice_mut() as *mut [_] as *mut Array<'ROWS, T>) }
	}
}

impl<'ROWS, 'a, T, RStride: Stride> ColMut<'a, T, Dim<'ROWS>, RStride> {
	#[inline]
	pub fn split_rows_with<'TOP, 'BOT>(self, row: Partition<'TOP, 'BOT, 'ROWS>) -> (ColRef<'a, T, Dim<'TOP>, RStride>, ColRef<'a, T, Dim<'BOT>, RStride>) {
		let (a, b) = self.split_at_row(row.midpoint());
		(a.as_row_shape(row.head), b.as_row_shape(row.tail))
	}
}

impl<'ROWS, 'a, T, RStride: Stride> ColMut<'a, T, Dim<'ROWS>, RStride> {
	#[inline]
	pub fn split_rows_with_mut<'TOP, 'BOT>(self, row: Partition<'TOP, 'BOT, 'ROWS>) -> (ColMut<'a, T, Dim<'TOP>, RStride>, ColMut<'a, T, Dim<'BOT>, RStride>) {
		let (a, b) = self.split_at_row_mut(row.midpoint());
		(a.as_row_shape_mut(row.head), b.as_row_shape_mut(row.tail))
	}
}

impl<T: core::fmt::Debug, Rows: Shape, RStride: Stride> core::fmt::Debug for ColMut<'_, T, Rows, RStride> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		self.rb().fmt(f)
	}
}

impl<T, Rows: Shape, CStride: Stride> Index<Idx<Rows>> for ColRef<'_, T, Rows, CStride> {
	type Output = T;

	#[inline]
	#[track_caller]
	fn index(&self, row: Idx<Rows>) -> &Self::Output {
		self.at(row)
	}
}

impl<T, Rows: Shape, CStride: Stride> Index<Idx<Rows>> for ColMut<'_, T, Rows, CStride> {
	type Output = T;

	#[inline]
	#[track_caller]
	fn index(&self, row: Idx<Rows>) -> &Self::Output {
		self.rb().at(row)
	}
}

impl<T, Rows: Shape, CStride: Stride> IndexMut<Idx<Rows>> for ColMut<'_, T, Rows, CStride> {
	#[inline]
	#[track_caller]
	fn index_mut(&mut self, row: Idx<Rows>) -> &mut Self::Output {
		self.rb_mut().at_mut(row)
	}
}
