use crate::assert;
use crate::col::*;
use crate::diag::*;
use crate::mat::*;
use crate::perm::*;
use crate::row::*;
use crate::sparse::*;
use crate::*;
use faer_entity::*;

use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

macro_rules! impl_partial_eq {
    ($lhs: ty, $rhs: ty) => {
        impl<LhsE: Conjugate, RhsE: Conjugate<Canonical = LhsE::Canonical>> PartialEq<$rhs>
            for $lhs
        {
            fn eq(&self, other: &$rhs) -> bool {
                self.as_ref().eq(&other.as_ref())
            }
        }
    };
}

macro_rules! impl_add_sub {
    ($lhs: ty, $rhs: ty, $out: ty) => {
        impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
            Add<$rhs> for $lhs
        {
            type Output = $out;
            #[track_caller]
            fn add(self, other: $rhs) -> Self::Output {
                self.as_ref().add(other.as_ref())
            }
        }

        impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
            Sub<$rhs> for $lhs
        {
            type Output = $out;
            #[track_caller]
            fn sub(self, other: $rhs) -> Self::Output {
                self.as_ref().sub(other.as_ref())
            }
        }
    };
}

macro_rules! impl_add_sub_assign {
    ($lhs: ty, $rhs: ty) => {
        impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> AddAssign<$rhs> for $lhs {
            #[track_caller]
            fn add_assign(&mut self, other: $rhs) {
                self.as_mut().add_assign(other.as_ref())
            }
        }

        impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> SubAssign<$rhs> for $lhs {
            #[track_caller]
            fn sub_assign(&mut self, other: $rhs) {
                self.as_mut().sub_assign(other.as_ref())
            }
        }
    };
}

macro_rules! impl_neg {
    ($mat: ty, $out: ty) => {
        impl<E: Conjugate> Neg for $mat
        where
            E::Canonical: ComplexField,
        {
            type Output = $out;
            #[track_caller]
            fn neg(self) -> Self::Output {
                self.as_ref().neg()
            }
        }
    };
}

macro_rules! impl_mul {
    ($lhs: ty, $rhs: ty, $out: ty) => {
        impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
            Mul<$rhs> for $lhs
        {
            type Output = $out;
            #[track_caller]
            fn mul(self, other: $rhs) -> Self::Output {
                self.as_ref().mul(other.as_ref())
            }
        }
    };
}

macro_rules! impl_perm {
    ($lhs: ty, $rhs: ty, $out: ty) => {
        impl<I: Index, E: Conjugate> Mul<$rhs> for $lhs
        where
            E::Canonical: ComplexField,
        {
            type Output = $out;
            #[track_caller]
            fn mul(self, other: $rhs) -> Self::Output {
                self.as_ref().mul(other.as_ref())
            }
        }
    };
}

macro_rules! impl_perm_perm {
    ($lhs: ty, $rhs: ty, $out: ty) => {
        impl<I: Index> Mul<$rhs> for $lhs {
            type Output = $out;
            #[track_caller]
            fn mul(self, other: $rhs) -> Self::Output {
                self.as_ref().mul(other.as_ref())
            }
        }
    };
}

macro_rules! impl_scalar_mul {
    ($lhs: ty, $rhs: ty, $out: ty) => {
        impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
            Mul<$rhs> for $lhs
        {
            type Output = $out;
            #[track_caller]
            fn mul(self, other: $rhs) -> Self::Output {
                self.mul(other.as_ref())
            }
        }
    };
}

macro_rules! impl_mul_scalar {
    ($lhs: ty, $rhs: ty, $out: ty) => {
        impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
            Mul<$rhs> for $lhs
        {
            type Output = $out;
            #[track_caller]
            fn mul(self, other: $rhs) -> Self::Output {
                self.as_ref().mul(other)
            }
        }
    };
}

macro_rules! impl_mul_assign_scalar {
    ($lhs: ty, $rhs: ty) => {
        impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> MulAssign<$rhs> for $lhs {
            #[track_caller]
            fn mul_assign(&mut self, other: $rhs) {
                self.as_mut().mul_assign(other)
            }
        }
    };
}

macro_rules! impl_sparse_mul {
    ($lhs: ty, $rhs: ty, $out: ty) => {
        impl<
                I: Index,
                E: ComplexField,
                LhsE: Conjugate<Canonical = E>,
                RhsE: Conjugate<Canonical = E>,
            > Mul<$rhs> for $lhs
        where
            E::Canonical: ComplexField,
        {
            type Output = $out;
            #[track_caller]
            fn mul(self, other: $rhs) -> Self::Output {
                self.as_ref().mul(other.as_ref())
            }
        }
    };
}

impl<LhsE: Conjugate, RhsE: Conjugate<Canonical = LhsE::Canonical>> PartialEq<MatRef<'_, RhsE>>
    for MatRef<'_, LhsE>
{
    fn eq(&self, other: &MatRef<'_, RhsE>) -> bool {
        let lhs = *self;
        let rhs = *other;

        if (lhs.nrows(), lhs.ncols()) != (rhs.nrows(), rhs.ncols()) {
            return false;
        }
        let m = lhs.nrows();
        let n = lhs.ncols();
        for j in 0..n {
            for i in 0..m {
                if !(lhs.read(i, j).canonicalize() == rhs.read(i, j).canonicalize()) {
                    return false;
                }
            }
        }

        true
    }
}

// impl_partial_eq!(MatRef<'_, LhsE>, MatRef<'_, RhsE>);
impl_partial_eq!(MatRef<'_, LhsE>, MatMut<'_, RhsE>);
impl_partial_eq!(MatRef<'_, LhsE>, Mat<RhsE>);

impl_partial_eq!(MatMut<'_, LhsE>, MatRef<'_, RhsE>);
impl_partial_eq!(MatMut<'_, LhsE>, MatMut<'_, RhsE>);
impl_partial_eq!(MatMut<'_, LhsE>, Mat<RhsE>);

impl_partial_eq!(Mat<LhsE>, MatRef<'_, RhsE>);
impl_partial_eq!(Mat<LhsE>, MatMut<'_, RhsE>);
impl_partial_eq!(Mat<LhsE>, Mat<RhsE>);

impl<LhsE: Conjugate, RhsE: Conjugate<Canonical = LhsE::Canonical>> PartialEq<ColRef<'_, RhsE>>
    for ColRef<'_, LhsE>
{
    fn eq(&self, other: &ColRef<'_, RhsE>) -> bool {
        self.as_2d().eq(&other.as_2d())
    }
}

// impl_partial_eq!(ColRef<'_, LhsE>, ColRef<'_, RhsE>);
impl_partial_eq!(ColRef<'_, LhsE>, ColMut<'_, RhsE>);
impl_partial_eq!(ColRef<'_, LhsE>, Col<RhsE>);

impl_partial_eq!(ColMut<'_, LhsE>, ColRef<'_, RhsE>);
impl_partial_eq!(ColMut<'_, LhsE>, ColMut<'_, RhsE>);
impl_partial_eq!(ColMut<'_, LhsE>, Col<RhsE>);

impl_partial_eq!(Col<LhsE>, ColRef<'_, RhsE>);
impl_partial_eq!(Col<LhsE>, ColMut<'_, RhsE>);
impl_partial_eq!(Col<LhsE>, Col<RhsE>);

impl<LhsE: Conjugate, RhsE: Conjugate<Canonical = LhsE::Canonical>> PartialEq<RowRef<'_, RhsE>>
    for RowRef<'_, LhsE>
{
    fn eq(&self, other: &RowRef<'_, RhsE>) -> bool {
        self.as_2d().eq(&other.as_2d())
    }
}

// impl_partial_eq!(RowRef<'_, LhsE>, RowRef<'_, RhsE>);
impl_partial_eq!(RowRef<'_, LhsE>, RowMut<'_, RhsE>);
impl_partial_eq!(RowRef<'_, LhsE>, Row<RhsE>);

impl_partial_eq!(RowMut<'_, LhsE>, RowRef<'_, RhsE>);
impl_partial_eq!(RowMut<'_, LhsE>, RowMut<'_, RhsE>);
impl_partial_eq!(RowMut<'_, LhsE>, Row<RhsE>);

impl_partial_eq!(Row<LhsE>, RowRef<'_, RhsE>);
impl_partial_eq!(Row<LhsE>, RowMut<'_, RhsE>);
impl_partial_eq!(Row<LhsE>, Row<RhsE>);

impl<LhsE: Conjugate, RhsE: Conjugate<Canonical = LhsE::Canonical>> PartialEq<DiagRef<'_, RhsE>>
    for DiagRef<'_, LhsE>
{
    fn eq(&self, other: &DiagRef<'_, RhsE>) -> bool {
        self.column_vector().eq(&other.column_vector())
    }
}

// impl_partial_eq!(DiagRef<'_, LhsE>, DiagRef<'_, RhsE>);
impl_partial_eq!(DiagRef<'_, LhsE>, DiagMut<'_, RhsE>);
impl_partial_eq!(DiagRef<'_, LhsE>, Diag<RhsE>);

impl_partial_eq!(DiagMut<'_, LhsE>, DiagRef<'_, RhsE>);
impl_partial_eq!(DiagMut<'_, LhsE>, DiagMut<'_, RhsE>);
impl_partial_eq!(DiagMut<'_, LhsE>, Diag<RhsE>);

impl_partial_eq!(Diag<LhsE>, DiagRef<'_, RhsE>);
impl_partial_eq!(Diag<LhsE>, DiagMut<'_, RhsE>);
impl_partial_eq!(Diag<LhsE>, Diag<RhsE>);

impl<I: Index> PartialEq<PermRef<'_, I>> for PermRef<'_, I> {
    #[inline]
    fn eq(&self, other: &PermRef<'_, I>) -> bool {
        self.arrays().0 == other.arrays().0
    }
}
impl<I: Index> PartialEq<PermRef<'_, I>> for Perm<I> {
    #[inline]
    fn eq(&self, other: &PermRef<'_, I>) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl<I: Index> PartialEq<Perm<I>> for PermRef<'_, I> {
    #[inline]
    fn eq(&self, other: &Perm<I>) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl<I: Index> PartialEq<Perm<I>> for Perm<I> {
    #[inline]
    fn eq(&self, other: &Perm<I>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Add<MatRef<'_, RhsE>> for MatRef<'_, LhsE>
{
    type Output = Mat<E>;

    #[track_caller]
    fn add(self, rhs: MatRef<'_, RhsE>) -> Self::Output {
        zipped!(self, rhs).map(|unzipped!(lhs, rhs)| {
            lhs.read()
                .canonicalize()
                .faer_add(rhs.read().canonicalize())
        })
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Sub<MatRef<'_, RhsE>> for MatRef<'_, LhsE>
{
    type Output = Mat<E>;

    #[track_caller]
    fn sub(self, rhs: MatRef<'_, RhsE>) -> Self::Output {
        zipped!(self, rhs).map(|unzipped!(lhs, rhs)| {
            lhs.read()
                .canonicalize()
                .faer_sub(rhs.read().canonicalize())
        })
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> AddAssign<MatRef<'_, RhsE>>
    for MatMut<'_, LhsE>
{
    #[track_caller]
    fn add_assign(&mut self, rhs: MatRef<'_, RhsE>) {
        zipped!(self.as_mut(), rhs).for_each(|unzipped!(mut lhs, rhs)| {
            lhs.write(lhs.read().faer_add(rhs.read().canonicalize()))
        })
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> SubAssign<MatRef<'_, RhsE>>
    for MatMut<'_, LhsE>
{
    #[track_caller]
    fn sub_assign(&mut self, rhs: MatRef<'_, RhsE>) {
        zipped!(self.as_mut(), rhs).for_each(|unzipped!(mut lhs, rhs)| {
            lhs.write(lhs.read().faer_sub(rhs.read().canonicalize()))
        })
    }
}

impl<E: Conjugate> Neg for MatRef<'_, E>
where
    E::Canonical: ComplexField,
{
    type Output = Mat<E::Canonical>;

    fn neg(self) -> Self::Output {
        zipped!(self).map(|unzipped!(x)| x.read().canonicalize().faer_neg())
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Add<ColRef<'_, RhsE>> for ColRef<'_, LhsE>
{
    type Output = Col<E>;

    #[track_caller]
    fn add(self, rhs: ColRef<'_, RhsE>) -> Self::Output {
        zipped!(self, rhs).map(|unzipped!(lhs, rhs)| {
            lhs.read()
                .canonicalize()
                .faer_add(rhs.read().canonicalize())
        })
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Sub<ColRef<'_, RhsE>> for ColRef<'_, LhsE>
{
    type Output = Col<E>;

    #[track_caller]
    fn sub(self, rhs: ColRef<'_, RhsE>) -> Self::Output {
        zipped!(self, rhs).map(|unzipped!(lhs, rhs)| {
            lhs.read()
                .canonicalize()
                .faer_sub(rhs.read().canonicalize())
        })
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> AddAssign<ColRef<'_, RhsE>>
    for ColMut<'_, LhsE>
{
    #[track_caller]
    fn add_assign(&mut self, rhs: ColRef<'_, RhsE>) {
        zipped!(self.as_mut(), rhs).for_each(|unzipped!(mut lhs, rhs)| {
            lhs.write(lhs.read().faer_add(rhs.read().canonicalize()))
        })
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> SubAssign<ColRef<'_, RhsE>>
    for ColMut<'_, LhsE>
{
    #[track_caller]
    fn sub_assign(&mut self, rhs: ColRef<'_, RhsE>) {
        zipped!(self.as_mut(), rhs).for_each(|unzipped!(mut lhs, rhs)| {
            lhs.write(lhs.read().faer_sub(rhs.read().canonicalize()))
        })
    }
}

impl<E: Conjugate> Neg for ColRef<'_, E>
where
    E::Canonical: ComplexField,
{
    type Output = Col<E::Canonical>;

    fn neg(self) -> Self::Output {
        zipped!(self).map(|unzipped!(x)| x.read().canonicalize().faer_neg())
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Add<RowRef<'_, RhsE>> for RowRef<'_, LhsE>
{
    type Output = Row<E>;

    #[track_caller]
    fn add(self, rhs: RowRef<'_, RhsE>) -> Self::Output {
        zipped!(self, rhs).map(|unzipped!(lhs, rhs)| {
            lhs.read()
                .canonicalize()
                .faer_add(rhs.read().canonicalize())
        })
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Sub<RowRef<'_, RhsE>> for RowRef<'_, LhsE>
{
    type Output = Row<E>;

    #[track_caller]
    fn sub(self, rhs: RowRef<'_, RhsE>) -> Self::Output {
        zipped!(self, rhs).map(|unzipped!(lhs, rhs)| {
            lhs.read()
                .canonicalize()
                .faer_sub(rhs.read().canonicalize())
        })
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> AddAssign<RowRef<'_, RhsE>>
    for RowMut<'_, LhsE>
{
    #[track_caller]
    fn add_assign(&mut self, rhs: RowRef<'_, RhsE>) {
        zipped!(self.as_mut(), rhs).for_each(|unzipped!(mut lhs, rhs)| {
            lhs.write(lhs.read().faer_add(rhs.read().canonicalize()))
        })
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> SubAssign<RowRef<'_, RhsE>>
    for RowMut<'_, LhsE>
{
    #[track_caller]
    fn sub_assign(&mut self, rhs: RowRef<'_, RhsE>) {
        zipped!(self.as_mut(), rhs).for_each(|unzipped!(mut lhs, rhs)| {
            lhs.write(lhs.read().faer_sub(rhs.read().canonicalize()))
        })
    }
}

impl<E: Conjugate> Neg for RowRef<'_, E>
where
    E::Canonical: ComplexField,
{
    type Output = Row<E::Canonical>;

    fn neg(self) -> Self::Output {
        zipped!(self).map(|unzipped!(x)| x.read().canonicalize().faer_neg())
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Add<DiagRef<'_, RhsE>> for DiagRef<'_, LhsE>
{
    type Output = Diag<E>;

    #[track_caller]
    fn add(self, rhs: DiagRef<'_, RhsE>) -> Self::Output {
        zipped!(self.column_vector(), rhs.column_vector())
            .map(|unzipped!(lhs, rhs)| {
                lhs.read()
                    .canonicalize()
                    .faer_add(rhs.read().canonicalize())
            })
            .column_vector_into_diagonal()
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Sub<DiagRef<'_, RhsE>> for DiagRef<'_, LhsE>
{
    type Output = Diag<E>;

    #[track_caller]
    fn sub(self, rhs: DiagRef<'_, RhsE>) -> Self::Output {
        zipped!(self.column_vector(), rhs.column_vector())
            .map(|unzipped!(lhs, rhs)| {
                lhs.read()
                    .canonicalize()
                    .faer_sub(rhs.read().canonicalize())
            })
            .column_vector_into_diagonal()
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> AddAssign<DiagRef<'_, RhsE>>
    for DiagMut<'_, LhsE>
{
    #[track_caller]
    fn add_assign(&mut self, rhs: DiagRef<'_, RhsE>) {
        zipped!(self.as_mut().column_vector_mut(), rhs.column_vector()).for_each(
            |unzipped!(mut lhs, rhs)| lhs.write(lhs.read().faer_add(rhs.read().canonicalize())),
        )
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> SubAssign<DiagRef<'_, RhsE>>
    for DiagMut<'_, LhsE>
{
    #[track_caller]
    fn sub_assign(&mut self, rhs: DiagRef<'_, RhsE>) {
        zipped!(self.as_mut().column_vector_mut(), rhs.column_vector()).for_each(
            |unzipped!(mut lhs, rhs)| lhs.write(lhs.read().faer_sub(rhs.read().canonicalize())),
        )
    }
}

impl<E: Conjugate> Neg for DiagRef<'_, E>
where
    E::Canonical: ComplexField,
{
    type Output = Diag<E::Canonical>;

    fn neg(self) -> Self::Output {
        zipped!(self.column_vector())
            .map(|unzipped!(x)| x.read().canonicalize().faer_neg())
            .column_vector_into_diagonal()
    }
}

// impl_add_sub!(MatRef<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(MatRef<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(MatRef<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_add_sub!(MatRef<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(MatRef<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(MatRef<'_, LhsE>, &Mat<RhsE>, Mat<E>);
impl_add_sub!(&MatRef<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(&MatRef<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(&MatRef<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_add_sub!(&MatRef<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(&MatRef<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(&MatRef<'_, LhsE>, &Mat<RhsE>, Mat<E>);

impl_add_sub!(MatMut<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(MatMut<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(MatMut<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_add_sub!(MatMut<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(MatMut<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(MatMut<'_, LhsE>, &Mat<RhsE>, Mat<E>);
impl_add_sub!(&MatMut<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(&MatMut<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(&MatMut<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_add_sub!(&MatMut<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(&MatMut<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(&MatMut<'_, LhsE>, &Mat<RhsE>, Mat<E>);

impl_add_sub!(Mat<LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(Mat<LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(Mat<LhsE>, Mat<RhsE>, Mat<E>);
impl_add_sub!(Mat<LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(Mat<LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(Mat<LhsE>, &Mat<RhsE>, Mat<E>);
impl_add_sub!(&Mat<LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(&Mat<LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(&Mat<LhsE>, Mat<RhsE>, Mat<E>);
impl_add_sub!(&Mat<LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_add_sub!(&Mat<LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_add_sub!(&Mat<LhsE>, &Mat<RhsE>, Mat<E>);

// impl_add_sub_assign!(MatMut<'_, LhsE>, MatRef<'_, RhsE>);
impl_add_sub_assign!(MatMut<'_, LhsE>, MatMut<'_, RhsE>);
impl_add_sub_assign!(MatMut<'_, LhsE>, Mat<RhsE>);
impl_add_sub_assign!(MatMut<'_, LhsE>, &MatRef<'_, RhsE>);
impl_add_sub_assign!(MatMut<'_, LhsE>, &MatMut<'_, RhsE>);
impl_add_sub_assign!(MatMut<'_, LhsE>, &Mat<RhsE>);

impl_add_sub_assign!(Mat<LhsE>, MatRef<'_, RhsE>);
impl_add_sub_assign!(Mat<LhsE>, MatMut<'_, RhsE>);
impl_add_sub_assign!(Mat<LhsE>, Mat<RhsE>);
impl_add_sub_assign!(Mat<LhsE>, &MatRef<'_, RhsE>);
impl_add_sub_assign!(Mat<LhsE>, &MatMut<'_, RhsE>);
impl_add_sub_assign!(Mat<LhsE>, &Mat<RhsE>);

// impl_neg!(MatRef<'_, E>, Mat<E::Canonical>);
impl_neg!(MatMut<'_, E>, Mat<E::Canonical>);
impl_neg!(Mat<E>, Mat<E::Canonical>);
impl_neg!(&MatRef<'_, E>, Mat<E::Canonical>);
impl_neg!(&MatMut<'_, E>, Mat<E::Canonical>);
impl_neg!(&Mat<E>, Mat<E::Canonical>);

// impl_add_sub!(ColRef<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(ColRef<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(ColRef<'_, LhsE>, Col<RhsE>, Col<E>);
impl_add_sub!(ColRef<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(ColRef<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(ColRef<'_, LhsE>, &Col<RhsE>, Col<E>);
impl_add_sub!(&ColRef<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(&ColRef<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(&ColRef<'_, LhsE>, Col<RhsE>, Col<E>);
impl_add_sub!(&ColRef<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(&ColRef<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(&ColRef<'_, LhsE>, &Col<RhsE>, Col<E>);

impl_add_sub!(ColMut<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(ColMut<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(ColMut<'_, LhsE>, Col<RhsE>, Col<E>);
impl_add_sub!(ColMut<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(ColMut<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(ColMut<'_, LhsE>, &Col<RhsE>, Col<E>);
impl_add_sub!(&ColMut<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(&ColMut<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(&ColMut<'_, LhsE>, Col<RhsE>, Col<E>);
impl_add_sub!(&ColMut<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(&ColMut<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(&ColMut<'_, LhsE>, &Col<RhsE>, Col<E>);

impl_add_sub!(Col<LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(Col<LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(Col<LhsE>, Col<RhsE>, Col<E>);
impl_add_sub!(Col<LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(Col<LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(Col<LhsE>, &Col<RhsE>, Col<E>);
impl_add_sub!(&Col<LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(&Col<LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(&Col<LhsE>, Col<RhsE>, Col<E>);
impl_add_sub!(&Col<LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_add_sub!(&Col<LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_add_sub!(&Col<LhsE>, &Col<RhsE>, Col<E>);

// impl_add_sub_assign!(ColMut<'_, LhsE>, ColRef<'_, RhsE>);
impl_add_sub_assign!(ColMut<'_, LhsE>, ColMut<'_, RhsE>);
impl_add_sub_assign!(ColMut<'_, LhsE>, Col<RhsE>);
impl_add_sub_assign!(ColMut<'_, LhsE>, &ColRef<'_, RhsE>);
impl_add_sub_assign!(ColMut<'_, LhsE>, &ColMut<'_, RhsE>);
impl_add_sub_assign!(ColMut<'_, LhsE>, &Col<RhsE>);

impl_add_sub_assign!(Col<LhsE>, ColRef<'_, RhsE>);
impl_add_sub_assign!(Col<LhsE>, ColMut<'_, RhsE>);
impl_add_sub_assign!(Col<LhsE>, Col<RhsE>);
impl_add_sub_assign!(Col<LhsE>, &ColRef<'_, RhsE>);
impl_add_sub_assign!(Col<LhsE>, &ColMut<'_, RhsE>);
impl_add_sub_assign!(Col<LhsE>, &Col<RhsE>);

// impl_neg!(ColRef<'_, E>, Col<E::Canonical>);
impl_neg!(ColMut<'_, E>, Col<E::Canonical>);
impl_neg!(Col<E>, Col<E::Canonical>);
impl_neg!(&ColRef<'_, E>, Col<E::Canonical>);
impl_neg!(&ColMut<'_, E>, Col<E::Canonical>);
impl_neg!(&Col<E>, Col<E::Canonical>);

// impl_add_sub!(RowRef<'_, LhsE>, RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(RowRef<'_, LhsE>, RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(RowRef<'_, LhsE>, Row<RhsE>, Row<E>);
impl_add_sub!(RowRef<'_, LhsE>, &RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(RowRef<'_, LhsE>, &RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(RowRef<'_, LhsE>, &Row<RhsE>, Row<E>);
impl_add_sub!(&RowRef<'_, LhsE>, RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(&RowRef<'_, LhsE>, RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(&RowRef<'_, LhsE>, Row<RhsE>, Row<E>);
impl_add_sub!(&RowRef<'_, LhsE>, &RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(&RowRef<'_, LhsE>, &RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(&RowRef<'_, LhsE>, &Row<RhsE>, Row<E>);

impl_add_sub!(RowMut<'_, LhsE>, RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(RowMut<'_, LhsE>, RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(RowMut<'_, LhsE>, Row<RhsE>, Row<E>);
impl_add_sub!(RowMut<'_, LhsE>, &RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(RowMut<'_, LhsE>, &RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(RowMut<'_, LhsE>, &Row<RhsE>, Row<E>);
impl_add_sub!(&RowMut<'_, LhsE>, RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(&RowMut<'_, LhsE>, RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(&RowMut<'_, LhsE>, Row<RhsE>, Row<E>);
impl_add_sub!(&RowMut<'_, LhsE>, &RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(&RowMut<'_, LhsE>, &RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(&RowMut<'_, LhsE>, &Row<RhsE>, Row<E>);

impl_add_sub!(Row<LhsE>, RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(Row<LhsE>, RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(Row<LhsE>, Row<RhsE>, Row<E>);
impl_add_sub!(Row<LhsE>, &RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(Row<LhsE>, &RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(Row<LhsE>, &Row<RhsE>, Row<E>);
impl_add_sub!(&Row<LhsE>, RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(&Row<LhsE>, RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(&Row<LhsE>, Row<RhsE>, Row<E>);
impl_add_sub!(&Row<LhsE>, &RowRef<'_, RhsE>, Row<E>);
impl_add_sub!(&Row<LhsE>, &RowMut<'_, RhsE>, Row<E>);
impl_add_sub!(&Row<LhsE>, &Row<RhsE>, Row<E>);

// impl_add_sub_assign!(RowMut<'_, LhsE>, RowRef<'_, RhsE>);
impl_add_sub_assign!(RowMut<'_, LhsE>, RowMut<'_, RhsE>);
impl_add_sub_assign!(RowMut<'_, LhsE>, Row<RhsE>);
impl_add_sub_assign!(RowMut<'_, LhsE>, &RowRef<'_, RhsE>);
impl_add_sub_assign!(RowMut<'_, LhsE>, &RowMut<'_, RhsE>);
impl_add_sub_assign!(RowMut<'_, LhsE>, &Row<RhsE>);

impl_add_sub_assign!(Row<LhsE>, RowRef<'_, RhsE>);
impl_add_sub_assign!(Row<LhsE>, RowMut<'_, RhsE>);
impl_add_sub_assign!(Row<LhsE>, Row<RhsE>);
impl_add_sub_assign!(Row<LhsE>, &RowRef<'_, RhsE>);
impl_add_sub_assign!(Row<LhsE>, &RowMut<'_, RhsE>);
impl_add_sub_assign!(Row<LhsE>, &Row<RhsE>);

// impl_neg!(RowRef<'_, E>, Row<E::Canonical>);
impl_neg!(RowMut<'_, E>, Row<E::Canonical>);
impl_neg!(Row<E>, Row<E::Canonical>);
impl_neg!(&RowRef<'_, E>, Row<E::Canonical>);
impl_neg!(&RowMut<'_, E>, Row<E::Canonical>);
impl_neg!(&Row<E>, Row<E::Canonical>);

// impl_add_sub!(DiagRef<'_, LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(DiagRef<'_, LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(DiagRef<'_, LhsE>, Diag<RhsE>, Diag<E>);
impl_add_sub!(DiagRef<'_, LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(DiagRef<'_, LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(DiagRef<'_, LhsE>, &Diag<RhsE>, Diag<E>);
impl_add_sub!(&DiagRef<'_, LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(&DiagRef<'_, LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(&DiagRef<'_, LhsE>, Diag<RhsE>, Diag<E>);
impl_add_sub!(&DiagRef<'_, LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(&DiagRef<'_, LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(&DiagRef<'_, LhsE>, &Diag<RhsE>, Diag<E>);

impl_add_sub!(DiagMut<'_, LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(DiagMut<'_, LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(DiagMut<'_, LhsE>, Diag<RhsE>, Diag<E>);
impl_add_sub!(DiagMut<'_, LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(DiagMut<'_, LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(DiagMut<'_, LhsE>, &Diag<RhsE>, Diag<E>);
impl_add_sub!(&DiagMut<'_, LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(&DiagMut<'_, LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(&DiagMut<'_, LhsE>, Diag<RhsE>, Diag<E>);
impl_add_sub!(&DiagMut<'_, LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(&DiagMut<'_, LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(&DiagMut<'_, LhsE>, &Diag<RhsE>, Diag<E>);

impl_add_sub!(Diag<LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(Diag<LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(Diag<LhsE>, Diag<RhsE>, Diag<E>);
impl_add_sub!(Diag<LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(Diag<LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(Diag<LhsE>, &Diag<RhsE>, Diag<E>);
impl_add_sub!(&Diag<LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(&Diag<LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(&Diag<LhsE>, Diag<RhsE>, Diag<E>);
impl_add_sub!(&Diag<LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_add_sub!(&Diag<LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_add_sub!(&Diag<LhsE>, &Diag<RhsE>, Diag<E>);

// impl_add_sub_assign!(DiagMut<'_, LhsE>, DiagRef<'_, RhsE>);
impl_add_sub_assign!(DiagMut<'_, LhsE>, DiagMut<'_, RhsE>);
impl_add_sub_assign!(DiagMut<'_, LhsE>, Diag<RhsE>);
impl_add_sub_assign!(DiagMut<'_, LhsE>, &DiagRef<'_, RhsE>);
impl_add_sub_assign!(DiagMut<'_, LhsE>, &DiagMut<'_, RhsE>);
impl_add_sub_assign!(DiagMut<'_, LhsE>, &Diag<RhsE>);

impl_add_sub_assign!(Diag<LhsE>, DiagRef<'_, RhsE>);
impl_add_sub_assign!(Diag<LhsE>, DiagMut<'_, RhsE>);
impl_add_sub_assign!(Diag<LhsE>, Diag<RhsE>);
impl_add_sub_assign!(Diag<LhsE>, &DiagRef<'_, RhsE>);
impl_add_sub_assign!(Diag<LhsE>, &DiagMut<'_, RhsE>);
impl_add_sub_assign!(Diag<LhsE>, &Diag<RhsE>);

// impl_neg!(DiagRef<'_, E>, Diag<E::Canonical>);
impl_neg!(DiagMut<'_, E>, Diag<E::Canonical>);
impl_neg!(Diag<E>, Diag<E::Canonical>);
impl_neg!(&DiagRef<'_, E>, Diag<E::Canonical>);
impl_neg!(&DiagMut<'_, E>, Diag<E::Canonical>);
impl_neg!(&Diag<E>, Diag<E::Canonical>);

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<Scale<RhsE>> for Scale<LhsE>
{
    type Output = Scale<E>;

    #[inline]
    fn mul(self, rhs: Scale<RhsE>) -> Self::Output {
        Scale(self.0.canonicalize().faer_mul(rhs.0.canonicalize()))
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Add<Scale<RhsE>> for Scale<LhsE>
{
    type Output = Scale<E>;

    #[inline]
    fn add(self, rhs: Scale<RhsE>) -> Self::Output {
        Scale(self.0.canonicalize().faer_add(rhs.0.canonicalize()))
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Sub<Scale<RhsE>> for Scale<LhsE>
{
    type Output = Scale<E>;

    #[inline]
    fn sub(self, rhs: Scale<RhsE>) -> Self::Output {
        Scale(self.0.canonicalize().faer_add(rhs.0.canonicalize()))
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> MulAssign<Scale<RhsE>> for Scale<LhsE> {
    #[inline]
    fn mul_assign(&mut self, rhs: Scale<RhsE>) {
        self.0 = self.0.faer_mul(rhs.0.canonicalize())
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> AddAssign<Scale<RhsE>> for Scale<LhsE> {
    #[inline]
    fn add_assign(&mut self, rhs: Scale<RhsE>) {
        self.0 = self.0.faer_add(rhs.0.canonicalize())
    }
}

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> SubAssign<Scale<RhsE>> for Scale<LhsE> {
    #[inline]
    fn sub_assign(&mut self, rhs: Scale<RhsE>) {
        self.0 = self.0.faer_sub(rhs.0.canonicalize())
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<MatRef<'_, RhsE>> for MatRef<'_, LhsE>
{
    type Output = Mat<E>;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: MatRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        assert!(lhs.ncols() == rhs.nrows());
        let mut out = Mat::zeros(lhs.nrows(), rhs.ncols());
        crate::linalg::matmul::matmul(
            out.as_mut(),
            lhs,
            rhs,
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<ColRef<'_, RhsE>> for MatRef<'_, LhsE>
{
    type Output = Col<E>;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: ColRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        assert!(lhs.ncols() == rhs.nrows());
        let mut out = Col::zeros(lhs.nrows());
        crate::linalg::matmul::matmul(
            out.as_mut().as_2d_mut(),
            lhs,
            rhs.as_2d(),
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<MatRef<'_, RhsE>> for RowRef<'_, LhsE>
{
    type Output = Row<E>;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: MatRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        assert!(lhs.ncols() == rhs.nrows());
        let mut out = Row::zeros(rhs.ncols());
        crate::linalg::matmul::matmul(
            out.as_mut().as_2d_mut(),
            lhs.as_2d(),
            rhs,
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<ColRef<'_, RhsE>> for RowRef<'_, LhsE>
{
    type Output = E;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: ColRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        assert!(lhs.ncols() == rhs.nrows());
        let (lhs, conj_lhs) = lhs.as_2d().transpose().canonicalize();
        let (rhs, conj_rhs) = rhs.as_2d().canonicalize();
        crate::linalg::matmul::inner_prod::inner_prod_with_conj(lhs, conj_lhs, rhs, conj_rhs)
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<RowRef<'_, RhsE>> for ColRef<'_, LhsE>
{
    type Output = Mat<E>;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: RowRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        assert!(lhs.ncols() == rhs.nrows());
        let mut out = Mat::zeros(lhs.nrows(), rhs.ncols());
        crate::linalg::matmul::matmul(
            out.as_mut(),
            lhs.as_2d(),
            rhs.as_2d(),
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

// impl_mul!(MatRef<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, &Mat<RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, &Mat<RhsE>, Mat<E>);

impl_mul!(MatMut<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, &Mat<RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, &Mat<RhsE>, Mat<E>);

impl_mul!(Mat<LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, &Mat<RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, &Mat<RhsE>, Mat<E>);

// impl_mul!(MatRef<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(MatRef<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(MatRef<'_, LhsE>, Col<RhsE>, Col<E>);
impl_mul!(MatRef<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(MatRef<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(MatRef<'_, LhsE>, &Col<RhsE>, Col<E>);
impl_mul!(&MatRef<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(&MatRef<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(&MatRef<'_, LhsE>, Col<RhsE>, Col<E>);
impl_mul!(&MatRef<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(&MatRef<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(&MatRef<'_, LhsE>, &Col<RhsE>, Col<E>);

impl_mul!(MatMut<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(MatMut<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(MatMut<'_, LhsE>, Col<RhsE>, Col<E>);
impl_mul!(MatMut<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(MatMut<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(MatMut<'_, LhsE>, &Col<RhsE>, Col<E>);
impl_mul!(&MatMut<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(&MatMut<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(&MatMut<'_, LhsE>, Col<RhsE>, Col<E>);
impl_mul!(&MatMut<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(&MatMut<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(&MatMut<'_, LhsE>, &Col<RhsE>, Col<E>);

impl_mul!(Mat<LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(Mat<LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(Mat<LhsE>, Col<RhsE>, Col<E>);
impl_mul!(Mat<LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(Mat<LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(Mat<LhsE>, &Col<RhsE>, Col<E>);
impl_mul!(&Mat<LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(&Mat<LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(&Mat<LhsE>, Col<RhsE>, Col<E>);
impl_mul!(&Mat<LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(&Mat<LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(&Mat<LhsE>, &Col<RhsE>, Col<E>);

// impl_mul!(RowRef<'_, LhsE>, MatRef<'_, RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, MatMut<'_, RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, Mat<RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, &MatRef<'_, RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, &MatMut<'_, RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, &Mat<RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, MatRef<'_, RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, MatMut<'_, RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, Mat<RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, &MatRef<'_, RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, &MatMut<'_, RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, &Mat<RhsE>, Row<E>);

impl_mul!(RowMut<'_, LhsE>, MatRef<'_, RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, MatMut<'_, RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, Mat<RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, &MatRef<'_, RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, &MatMut<'_, RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, &Mat<RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, MatRef<'_, RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, MatMut<'_, RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, Mat<RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, &MatRef<'_, RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, &MatMut<'_, RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, &Mat<RhsE>, Row<E>);

impl_mul!(Row<LhsE>, MatRef<'_, RhsE>, Row<E>);
impl_mul!(Row<LhsE>, MatMut<'_, RhsE>, Row<E>);
impl_mul!(Row<LhsE>, Mat<RhsE>, Row<E>);
impl_mul!(Row<LhsE>, &MatRef<'_, RhsE>, Row<E>);
impl_mul!(Row<LhsE>, &MatMut<'_, RhsE>, Row<E>);
impl_mul!(Row<LhsE>, &Mat<RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, MatRef<'_, RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, MatMut<'_, RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, Mat<RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, &MatRef<'_, RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, &MatMut<'_, RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, &Mat<RhsE>, Row<E>);

// impl_mul!(RowRef<'_, LhsE>, ColRef<'_, RhsE>, E);
impl_mul!(RowRef<'_, LhsE>, ColMut<'_, RhsE>, E);
impl_mul!(RowRef<'_, LhsE>, Col<RhsE>, E);
impl_mul!(RowRef<'_, LhsE>, &ColRef<'_, RhsE>, E);
impl_mul!(RowRef<'_, LhsE>, &ColMut<'_, RhsE>, E);
impl_mul!(RowRef<'_, LhsE>, &Col<RhsE>, E);
impl_mul!(&RowRef<'_, LhsE>, ColRef<'_, RhsE>, E);
impl_mul!(&RowRef<'_, LhsE>, ColMut<'_, RhsE>, E);
impl_mul!(&RowRef<'_, LhsE>, Col<RhsE>, E);
impl_mul!(&RowRef<'_, LhsE>, &ColRef<'_, RhsE>, E);
impl_mul!(&RowRef<'_, LhsE>, &ColMut<'_, RhsE>, E);
impl_mul!(&RowRef<'_, LhsE>, &Col<RhsE>, E);

impl_mul!(RowMut<'_, LhsE>, ColRef<'_, RhsE>, E);
impl_mul!(RowMut<'_, LhsE>, ColMut<'_, RhsE>, E);
impl_mul!(RowMut<'_, LhsE>, Col<RhsE>, E);
impl_mul!(RowMut<'_, LhsE>, &ColRef<'_, RhsE>, E);
impl_mul!(RowMut<'_, LhsE>, &ColMut<'_, RhsE>, E);
impl_mul!(RowMut<'_, LhsE>, &Col<RhsE>, E);
impl_mul!(&RowMut<'_, LhsE>, ColRef<'_, RhsE>, E);
impl_mul!(&RowMut<'_, LhsE>, ColMut<'_, RhsE>, E);
impl_mul!(&RowMut<'_, LhsE>, Col<RhsE>, E);
impl_mul!(&RowMut<'_, LhsE>, &ColRef<'_, RhsE>, E);
impl_mul!(&RowMut<'_, LhsE>, &ColMut<'_, RhsE>, E);
impl_mul!(&RowMut<'_, LhsE>, &Col<RhsE>, E);

impl_mul!(Row<LhsE>, ColRef<'_, RhsE>, E);
impl_mul!(Row<LhsE>, ColMut<'_, RhsE>, E);
impl_mul!(Row<LhsE>, Col<RhsE>, E);
impl_mul!(Row<LhsE>, &ColRef<'_, RhsE>, E);
impl_mul!(Row<LhsE>, &ColMut<'_, RhsE>, E);
impl_mul!(Row<LhsE>, &Col<RhsE>, E);
impl_mul!(&Row<LhsE>, ColRef<'_, RhsE>, E);
impl_mul!(&Row<LhsE>, ColMut<'_, RhsE>, E);
impl_mul!(&Row<LhsE>, Col<RhsE>, E);
impl_mul!(&Row<LhsE>, &ColRef<'_, RhsE>, E);
impl_mul!(&Row<LhsE>, &ColMut<'_, RhsE>, E);
impl_mul!(&Row<LhsE>, &Col<RhsE>, E);

// impl_mul!(ColRef<'_, LhsE>, RowRef<'_, RhsE>, Mat<E>);
impl_mul!(ColRef<'_, LhsE>, RowMut<'_, RhsE>, Mat<E>);
impl_mul!(ColRef<'_, LhsE>, Row<RhsE>, Mat<E>);
impl_mul!(ColRef<'_, LhsE>, &RowRef<'_, RhsE>, Mat<E>);
impl_mul!(ColRef<'_, LhsE>, &RowMut<'_, RhsE>, Mat<E>);
impl_mul!(ColRef<'_, LhsE>, &Row<RhsE>, Mat<E>);
impl_mul!(&ColRef<'_, LhsE>, RowRef<'_, RhsE>, Mat<E>);
impl_mul!(&ColRef<'_, LhsE>, RowMut<'_, RhsE>, Mat<E>);
impl_mul!(&ColRef<'_, LhsE>, Row<RhsE>, Mat<E>);
impl_mul!(&ColRef<'_, LhsE>, &RowRef<'_, RhsE>, Mat<E>);
impl_mul!(&ColRef<'_, LhsE>, &RowMut<'_, RhsE>, Mat<E>);
impl_mul!(&ColRef<'_, LhsE>, &Row<RhsE>, Mat<E>);

impl_mul!(ColMut<'_, LhsE>, RowRef<'_, RhsE>, Mat<E>);
impl_mul!(ColMut<'_, LhsE>, RowMut<'_, RhsE>, Mat<E>);
impl_mul!(ColMut<'_, LhsE>, Row<RhsE>, Mat<E>);
impl_mul!(ColMut<'_, LhsE>, &RowRef<'_, RhsE>, Mat<E>);
impl_mul!(ColMut<'_, LhsE>, &RowMut<'_, RhsE>, Mat<E>);
impl_mul!(ColMut<'_, LhsE>, &Row<RhsE>, Mat<E>);
impl_mul!(&ColMut<'_, LhsE>, RowRef<'_, RhsE>, Mat<E>);
impl_mul!(&ColMut<'_, LhsE>, RowMut<'_, RhsE>, Mat<E>);
impl_mul!(&ColMut<'_, LhsE>, Row<RhsE>, Mat<E>);
impl_mul!(&ColMut<'_, LhsE>, &RowRef<'_, RhsE>, Mat<E>);
impl_mul!(&ColMut<'_, LhsE>, &RowMut<'_, RhsE>, Mat<E>);
impl_mul!(&ColMut<'_, LhsE>, &Row<RhsE>, Mat<E>);

impl_mul!(Col<LhsE>, RowRef<'_, RhsE>, Mat<E>);
impl_mul!(Col<LhsE>, RowMut<'_, RhsE>, Mat<E>);
impl_mul!(Col<LhsE>, Row<RhsE>, Mat<E>);
impl_mul!(Col<LhsE>, &RowRef<'_, RhsE>, Mat<E>);
impl_mul!(Col<LhsE>, &RowMut<'_, RhsE>, Mat<E>);
impl_mul!(Col<LhsE>, &Row<RhsE>, Mat<E>);
impl_mul!(&Col<LhsE>, RowRef<'_, RhsE>, Mat<E>);
impl_mul!(&Col<LhsE>, RowMut<'_, RhsE>, Mat<E>);
impl_mul!(&Col<LhsE>, Row<RhsE>, Mat<E>);
impl_mul!(&Col<LhsE>, &RowRef<'_, RhsE>, Mat<E>);
impl_mul!(&Col<LhsE>, &RowMut<'_, RhsE>, Mat<E>);
impl_mul!(&Col<LhsE>, &Row<RhsE>, Mat<E>);

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<MatRef<'_, RhsE>> for DiagRef<'_, LhsE>
{
    type Output = Mat<E>;

    #[track_caller]
    fn mul(self, rhs: MatRef<'_, RhsE>) -> Self::Output {
        let lhs = self.column_vector();
        let lhs_dim = lhs.nrows();
        let rhs_nrows = rhs.nrows();
        assert!(lhs_dim == rhs_nrows);

        Mat::from_fn(rhs.nrows(), rhs.ncols(), |i, j| unsafe {
            E::faer_mul(
                lhs.read_unchecked(i).canonicalize(),
                rhs.read_unchecked(i, j).canonicalize(),
            )
        })
    }
}

// impl_mul!(DiagRef<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(DiagRef<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(DiagRef<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(DiagRef<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(DiagRef<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(DiagRef<'_, LhsE>, &Mat<RhsE>, Mat<E>);
impl_mul!(&DiagRef<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&DiagRef<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&DiagRef<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(&DiagRef<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&DiagRef<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&DiagRef<'_, LhsE>, &Mat<RhsE>, Mat<E>);

impl_mul!(DiagMut<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(DiagMut<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(DiagMut<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(DiagMut<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(DiagMut<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(DiagMut<'_, LhsE>, &Mat<RhsE>, Mat<E>);
impl_mul!(&DiagMut<'_, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&DiagMut<'_, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&DiagMut<'_, LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(&DiagMut<'_, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&DiagMut<'_, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&DiagMut<'_, LhsE>, &Mat<RhsE>, Mat<E>);

impl_mul!(Diag<LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(Diag<LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(Diag<LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(Diag<LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(Diag<LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(Diag<LhsE>, &Mat<RhsE>, Mat<E>);
impl_mul!(&Diag<LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&Diag<LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&Diag<LhsE>, Mat<RhsE>, Mat<E>);
impl_mul!(&Diag<LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_mul!(&Diag<LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_mul!(&Diag<LhsE>, &Mat<RhsE>, Mat<E>);

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<ColRef<'_, RhsE>> for DiagRef<'_, LhsE>
{
    type Output = Col<E>;

    #[track_caller]
    fn mul(self, rhs: ColRef<'_, RhsE>) -> Self::Output {
        let lhs = self.column_vector();
        let lhs_dim = lhs.nrows();
        let rhs_nrows = rhs.nrows();
        assert!(lhs_dim == rhs_nrows);

        Col::from_fn(rhs.nrows(), |i| unsafe {
            E::faer_mul(
                lhs.read_unchecked(i).canonicalize(),
                rhs.read_unchecked(i).canonicalize(),
            )
        })
    }
}

// impl_mul!(DiagRef<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(DiagRef<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(DiagRef<'_, LhsE>, Col<RhsE>, Col<E>);
impl_mul!(DiagRef<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(DiagRef<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(DiagRef<'_, LhsE>, &Col<RhsE>, Col<E>);
impl_mul!(&DiagRef<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(&DiagRef<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(&DiagRef<'_, LhsE>, Col<RhsE>, Col<E>);
impl_mul!(&DiagRef<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(&DiagRef<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(&DiagRef<'_, LhsE>, &Col<RhsE>, Col<E>);

impl_mul!(DiagMut<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(DiagMut<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(DiagMut<'_, LhsE>, Col<RhsE>, Col<E>);
impl_mul!(DiagMut<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(DiagMut<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(DiagMut<'_, LhsE>, &Col<RhsE>, Col<E>);
impl_mul!(&DiagMut<'_, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(&DiagMut<'_, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(&DiagMut<'_, LhsE>, Col<RhsE>, Col<E>);
impl_mul!(&DiagMut<'_, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(&DiagMut<'_, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(&DiagMut<'_, LhsE>, &Col<RhsE>, Col<E>);

impl_mul!(Diag<LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(Diag<LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(Diag<LhsE>, Col<RhsE>, Col<E>);
impl_mul!(Diag<LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(Diag<LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(Diag<LhsE>, &Col<RhsE>, Col<E>);
impl_mul!(&Diag<LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_mul!(&Diag<LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_mul!(&Diag<LhsE>, Col<RhsE>, Col<E>);
impl_mul!(&Diag<LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_mul!(&Diag<LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_mul!(&Diag<LhsE>, &Col<RhsE>, Col<E>);

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<DiagRef<'_, RhsE>> for MatRef<'_, LhsE>
{
    type Output = Mat<E>;

    #[track_caller]
    fn mul(self, rhs: DiagRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        let rhs = rhs.column_vector();
        let lhs_ncols = lhs.ncols();
        let rhs_dim = rhs.nrows();
        assert!(lhs_ncols == rhs_dim);

        Mat::from_fn(lhs.nrows(), lhs.ncols(), |i, j| unsafe {
            E::faer_mul(
                lhs.read_unchecked(i, j).canonicalize(),
                rhs.read_unchecked(j).canonicalize(),
            )
        })
    }
}

// impl_mul!(MatRef<'_, LhsE>, DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, Diag<RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, &DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, &DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(MatRef<'_, LhsE>, &Diag<RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, Diag<RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, &DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, &DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(&MatRef<'_, LhsE>, &Diag<RhsE>, Mat<E>);

impl_mul!(MatMut<'_, LhsE>, DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, Diag<RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, &DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, &DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(MatMut<'_, LhsE>, &Diag<RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, Diag<RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, &DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, &DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(&MatMut<'_, LhsE>, &Diag<RhsE>, Mat<E>);

impl_mul!(Mat<LhsE>, DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, Diag<RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, &DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, &DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(Mat<LhsE>, &Diag<RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, Diag<RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, &DiagRef<'_, RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, &DiagMut<'_, RhsE>, Mat<E>);
impl_mul!(&Mat<LhsE>, &Diag<RhsE>, Mat<E>);

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<DiagRef<'_, RhsE>> for RowRef<'_, LhsE>
{
    type Output = Row<E>;

    #[track_caller]
    fn mul(self, rhs: DiagRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        let rhs = rhs.column_vector();
        let lhs_ncols = lhs.ncols();
        let rhs_dim = rhs.nrows();
        assert!(lhs_ncols == rhs_dim);

        Row::from_fn(lhs.ncols(), |j| unsafe {
            E::faer_mul(
                lhs.read_unchecked(j).canonicalize(),
                rhs.read_unchecked(j).canonicalize(),
            )
        })
    }
}

// impl_mul!(RowRef<'_, LhsE>, DiagRef<'_, RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, DiagMut<'_, RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, Diag<RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, &DiagRef<'_, RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, &DiagMut<'_, RhsE>, Row<E>);
impl_mul!(RowRef<'_, LhsE>, &Diag<RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, DiagRef<'_, RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, DiagMut<'_, RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, Diag<RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, &DiagRef<'_, RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, &DiagMut<'_, RhsE>, Row<E>);
impl_mul!(&RowRef<'_, LhsE>, &Diag<RhsE>, Row<E>);

impl_mul!(RowMut<'_, LhsE>, DiagRef<'_, RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, DiagMut<'_, RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, Diag<RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, &DiagRef<'_, RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, &DiagMut<'_, RhsE>, Row<E>);
impl_mul!(RowMut<'_, LhsE>, &Diag<RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, DiagRef<'_, RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, DiagMut<'_, RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, Diag<RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, &DiagRef<'_, RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, &DiagMut<'_, RhsE>, Row<E>);
impl_mul!(&RowMut<'_, LhsE>, &Diag<RhsE>, Row<E>);

impl_mul!(Row<LhsE>, DiagRef<'_, RhsE>, Row<E>);
impl_mul!(Row<LhsE>, DiagMut<'_, RhsE>, Row<E>);
impl_mul!(Row<LhsE>, Diag<RhsE>, Row<E>);
impl_mul!(Row<LhsE>, &DiagRef<'_, RhsE>, Row<E>);
impl_mul!(Row<LhsE>, &DiagMut<'_, RhsE>, Row<E>);
impl_mul!(Row<LhsE>, &Diag<RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, DiagRef<'_, RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, DiagMut<'_, RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, Diag<RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, &DiagRef<'_, RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, &DiagMut<'_, RhsE>, Row<E>);
impl_mul!(&Row<LhsE>, &Diag<RhsE>, Row<E>);

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<DiagRef<'_, RhsE>> for DiagRef<'_, LhsE>
{
    type Output = Diag<E>;

    #[track_caller]
    fn mul(self, rhs: DiagRef<'_, RhsE>) -> Self::Output {
        let lhs = self.column_vector();
        let rhs = rhs.column_vector();
        assert!(lhs.nrows() == rhs.nrows());

        Col::from_fn(lhs.nrows(), |i| unsafe {
            E::faer_mul(
                lhs.read_unchecked(i).canonicalize(),
                rhs.read_unchecked(i).canonicalize(),
            )
        })
        .column_vector_into_diagonal()
    }
}

// impl_mul!(DiagRef<'_, LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(DiagRef<'_, LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(DiagRef<'_, LhsE>, Diag<RhsE>, Diag<E>);
impl_mul!(DiagRef<'_, LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(DiagRef<'_, LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(DiagRef<'_, LhsE>, &Diag<RhsE>, Diag<E>);
impl_mul!(&DiagRef<'_, LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(&DiagRef<'_, LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(&DiagRef<'_, LhsE>, Diag<RhsE>, Diag<E>);
impl_mul!(&DiagRef<'_, LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(&DiagRef<'_, LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(&DiagRef<'_, LhsE>, &Diag<RhsE>, Diag<E>);

impl_mul!(DiagMut<'_, LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(DiagMut<'_, LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(DiagMut<'_, LhsE>, Diag<RhsE>, Diag<E>);
impl_mul!(DiagMut<'_, LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(DiagMut<'_, LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(DiagMut<'_, LhsE>, &Diag<RhsE>, Diag<E>);
impl_mul!(&DiagMut<'_, LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(&DiagMut<'_, LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(&DiagMut<'_, LhsE>, Diag<RhsE>, Diag<E>);
impl_mul!(&DiagMut<'_, LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(&DiagMut<'_, LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(&DiagMut<'_, LhsE>, &Diag<RhsE>, Diag<E>);

impl_mul!(Diag<LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(Diag<LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(Diag<LhsE>, Diag<RhsE>, Diag<E>);
impl_mul!(Diag<LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(Diag<LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(Diag<LhsE>, &Diag<RhsE>, Diag<E>);
impl_mul!(&Diag<LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(&Diag<LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(&Diag<LhsE>, Diag<RhsE>, Diag<E>);
impl_mul!(&Diag<LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_mul!(&Diag<LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_mul!(&Diag<LhsE>, &Diag<RhsE>, Diag<E>);

impl<I: Index> Mul<PermRef<'_, I>> for PermRef<'_, I> {
    type Output = Perm<I>;

    #[track_caller]
    fn mul(self, rhs: PermRef<'_, I>) -> Self::Output {
        let lhs = self;
        assert!(lhs.len() == rhs.len());
        let truncate = <I::Signed as SignedIndex>::truncate;
        let mut fwd = alloc::vec![I::from_signed(truncate(0)); lhs.len()].into_boxed_slice();
        let mut inv = alloc::vec![I::from_signed(truncate(0)); lhs.len()].into_boxed_slice();

        for (fwd, rhs) in fwd.iter_mut().zip(rhs.arrays().0) {
            *fwd = lhs.arrays().0[rhs.to_signed().zx()];
        }
        for (i, fwd) in fwd.iter().enumerate() {
            inv[fwd.to_signed().zx()] = I::from_signed(I::Signed::truncate(i));
        }

        Perm::new_checked(fwd, inv)
    }
}

// impl_perm_perm!(PermRef<'_, I>, PermRef<'_, I>, Perm<I>);
impl_perm_perm!(PermRef<'_, I>, Perm<I>, Perm<I>);
impl_perm_perm!(PermRef<'_, I>, &PermRef<'_, I>, Perm<I>);
impl_perm_perm!(PermRef<'_, I>, &Perm<I>, Perm<I>);
impl_perm_perm!(&PermRef<'_, I>, PermRef<'_, I>, Perm<I>);
impl_perm_perm!(&PermRef<'_, I>, Perm<I>, Perm<I>);
impl_perm_perm!(&PermRef<'_, I>, &PermRef<'_, I>, Perm<I>);
impl_perm_perm!(&PermRef<'_, I>, &Perm<I>, Perm<I>);

impl_perm_perm!(Perm<I>, PermRef<'_, I>, Perm<I>);
impl_perm_perm!(Perm<I>, Perm<I>, Perm<I>);
impl_perm_perm!(Perm<I>, &PermRef<'_, I>, Perm<I>);
impl_perm_perm!(Perm<I>, &Perm<I>, Perm<I>);
impl_perm_perm!(&Perm<I>, PermRef<'_, I>, Perm<I>);
impl_perm_perm!(&Perm<I>, Perm<I>, Perm<I>);
impl_perm_perm!(&Perm<I>, &PermRef<'_, I>, Perm<I>);
impl_perm_perm!(&Perm<I>, &Perm<I>, Perm<I>);

impl<I: Index, E: Conjugate> Mul<MatRef<'_, E>> for PermRef<'_, I>
where
    E::Canonical: ComplexField,
{
    type Output = Mat<E::Canonical>;

    #[track_caller]
    fn mul(self, rhs: MatRef<'_, E>) -> Self::Output {
        let lhs = self;

        assert!(lhs.len() == rhs.nrows());
        let mut out = Mat::zeros(rhs.nrows(), rhs.ncols());
        let fwd = lhs.arrays().0;

        for j in 0..rhs.ncols() {
            for (i, fwd) in fwd.iter().enumerate() {
                out.write(i, j, rhs.read(fwd.to_signed().zx(), j).canonicalize());
            }
        }
        out
    }
}

// impl_perm!(PermRef<'_, I>, MatRef<'_, E>, Mat<E::Canonical>);
impl_perm!(PermRef<'_, I>, MatMut<'_, E>, Mat<E::Canonical>);
impl_perm!(PermRef<'_, I>, Mat<E>, Mat<E::Canonical>);
impl_perm!(PermRef<'_, I>, &MatRef<'_, E>, Mat<E::Canonical>);
impl_perm!(PermRef<'_, I>, &MatMut<'_, E>, Mat<E::Canonical>);
impl_perm!(PermRef<'_, I>, &Mat<E>, Mat<E::Canonical>);
impl_perm!(&PermRef<'_, I>, MatRef<'_, E>, Mat<E::Canonical>);
impl_perm!(&PermRef<'_, I>, MatMut<'_, E>, Mat<E::Canonical>);
impl_perm!(&PermRef<'_, I>, Mat<E>, Mat<E::Canonical>);
impl_perm!(&PermRef<'_, I>, &MatRef<'_, E>, Mat<E::Canonical>);
impl_perm!(&PermRef<'_, I>, &MatMut<'_, E>, Mat<E::Canonical>);
impl_perm!(&PermRef<'_, I>, &Mat<E>, Mat<E::Canonical>);

impl_perm!(Perm<I>, MatRef<'_, E>, Mat<E::Canonical>);
impl_perm!(Perm<I>, MatMut<'_, E>, Mat<E::Canonical>);
impl_perm!(Perm<I>, Mat<E>, Mat<E::Canonical>);
impl_perm!(Perm<I>, &MatRef<'_, E>, Mat<E::Canonical>);
impl_perm!(Perm<I>, &MatMut<'_, E>, Mat<E::Canonical>);
impl_perm!(Perm<I>, &Mat<E>, Mat<E::Canonical>);
impl_perm!(&Perm<I>, MatRef<'_, E>, Mat<E::Canonical>);
impl_perm!(&Perm<I>, MatMut<'_, E>, Mat<E::Canonical>);
impl_perm!(&Perm<I>, Mat<E>, Mat<E::Canonical>);
impl_perm!(&Perm<I>, &MatRef<'_, E>, Mat<E::Canonical>);
impl_perm!(&Perm<I>, &MatMut<'_, E>, Mat<E::Canonical>);
impl_perm!(&Perm<I>, &Mat<E>, Mat<E::Canonical>);

impl<I: Index, E: Conjugate> Mul<ColRef<'_, E>> for PermRef<'_, I>
where
    E::Canonical: ComplexField,
{
    type Output = Col<E::Canonical>;

    #[track_caller]
    fn mul(self, rhs: ColRef<'_, E>) -> Self::Output {
        let lhs = self;

        assert!(lhs.len() == rhs.nrows());
        let mut out = Col::zeros(rhs.nrows());
        let fwd = lhs.arrays().0;

        for (i, fwd) in fwd.iter().enumerate() {
            out.write(i, rhs.read(fwd.to_signed().zx()).canonicalize());
        }
        out
    }
}

// impl_perm!(PermRef<'_, I>, ColRef<'_, E>, Col<E::Canonical>);
impl_perm!(PermRef<'_, I>, ColMut<'_, E>, Col<E::Canonical>);
impl_perm!(PermRef<'_, I>, Col<E>, Col<E::Canonical>);
impl_perm!(PermRef<'_, I>, &ColRef<'_, E>, Col<E::Canonical>);
impl_perm!(PermRef<'_, I>, &ColMut<'_, E>, Col<E::Canonical>);
impl_perm!(PermRef<'_, I>, &Col<E>, Col<E::Canonical>);
impl_perm!(&PermRef<'_, I>, ColRef<'_, E>, Col<E::Canonical>);
impl_perm!(&PermRef<'_, I>, ColMut<'_, E>, Col<E::Canonical>);
impl_perm!(&PermRef<'_, I>, Col<E>, Col<E::Canonical>);
impl_perm!(&PermRef<'_, I>, &ColRef<'_, E>, Col<E::Canonical>);
impl_perm!(&PermRef<'_, I>, &ColMut<'_, E>, Col<E::Canonical>);
impl_perm!(&PermRef<'_, I>, &Col<E>, Col<E::Canonical>);

impl_perm!(Perm<I>, ColRef<'_, E>, Col<E::Canonical>);
impl_perm!(Perm<I>, ColMut<'_, E>, Col<E::Canonical>);
impl_perm!(Perm<I>, Col<E>, Col<E::Canonical>);
impl_perm!(Perm<I>, &ColRef<'_, E>, Col<E::Canonical>);
impl_perm!(Perm<I>, &ColMut<'_, E>, Col<E::Canonical>);
impl_perm!(Perm<I>, &Col<E>, Col<E::Canonical>);
impl_perm!(&Perm<I>, ColRef<'_, E>, Col<E::Canonical>);
impl_perm!(&Perm<I>, ColMut<'_, E>, Col<E::Canonical>);
impl_perm!(&Perm<I>, Col<E>, Col<E::Canonical>);
impl_perm!(&Perm<I>, &ColRef<'_, E>, Col<E::Canonical>);
impl_perm!(&Perm<I>, &ColMut<'_, E>, Col<E::Canonical>);
impl_perm!(&Perm<I>, &Col<E>, Col<E::Canonical>);

impl<I: Index, E: Conjugate> Mul<PermRef<'_, I>> for MatRef<'_, E>
where
    E::Canonical: ComplexField,
{
    type Output = Mat<E::Canonical>;

    #[track_caller]
    fn mul(self, rhs: PermRef<'_, I>) -> Self::Output {
        let lhs = self;

        assert!(lhs.ncols() == rhs.len());
        let mut out = Mat::zeros(lhs.nrows(), lhs.ncols());
        let inv = rhs.arrays().1;

        for (j, inv) in inv.iter().enumerate() {
            for i in 0..lhs.nrows() {
                out.write(i, j, lhs.read(i, inv.to_signed().zx()).canonicalize());
            }
        }
        out
    }
}

// impl_perm!(MatRef<'_, E>, PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(MatRef<'_, E>, Perm<I>, Mat<E::Canonical>);
impl_perm!(MatRef<'_, E>, &PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(MatRef<'_, E>, &Perm<I>, Mat<E::Canonical>);
impl_perm!(&MatRef<'_, E>, PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(&MatRef<'_, E>, Perm<I>, Mat<E::Canonical>);
impl_perm!(&MatRef<'_, E>, &PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(&MatRef<'_, E>, &Perm<I>, Mat<E::Canonical>);

impl_perm!(MatMut<'_, E>, PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(MatMut<'_, E>, Perm<I>, Mat<E::Canonical>);
impl_perm!(MatMut<'_, E>, &PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(MatMut<'_, E>, &Perm<I>, Mat<E::Canonical>);
impl_perm!(&MatMut<'_, E>, PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(&MatMut<'_, E>, Perm<I>, Mat<E::Canonical>);
impl_perm!(&MatMut<'_, E>, &PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(&MatMut<'_, E>, &Perm<I>, Mat<E::Canonical>);

impl_perm!(Mat<E>, PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(Mat<E>, Perm<I>, Mat<E::Canonical>);
impl_perm!(Mat<E>, &PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(Mat<E>, &Perm<I>, Mat<E::Canonical>);
impl_perm!(&Mat<E>, PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(&Mat<E>, Perm<I>, Mat<E::Canonical>);
impl_perm!(&Mat<E>, &PermRef<'_, I>, Mat<E::Canonical>);
impl_perm!(&Mat<E>, &Perm<I>, Mat<E::Canonical>);

impl<I: Index, E: Conjugate> Mul<PermRef<'_, I>> for RowRef<'_, E>
where
    E::Canonical: ComplexField,
{
    type Output = Row<E::Canonical>;

    #[track_caller]
    fn mul(self, rhs: PermRef<'_, I>) -> Self::Output {
        let lhs = self;

        assert!(lhs.ncols() == rhs.len());
        let mut out = Row::zeros(lhs.ncols());
        let inv = rhs.arrays().1;

        for (j, inv) in inv.iter().enumerate() {
            out.write(j, lhs.read(inv.to_signed().zx()).canonicalize());
        }
        out
    }
}

// impl_perm!(RowRef<'_, E>, PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(RowRef<'_, E>, Perm<I>, Row<E::Canonical>);
impl_perm!(RowRef<'_, E>, &PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(RowRef<'_, E>, &Perm<I>, Row<E::Canonical>);
impl_perm!(&RowRef<'_, E>, PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(&RowRef<'_, E>, Perm<I>, Row<E::Canonical>);
impl_perm!(&RowRef<'_, E>, &PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(&RowRef<'_, E>, &Perm<I>, Row<E::Canonical>);

impl_perm!(RowMut<'_, E>, PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(RowMut<'_, E>, Perm<I>, Row<E::Canonical>);
impl_perm!(RowMut<'_, E>, &PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(RowMut<'_, E>, &Perm<I>, Row<E::Canonical>);
impl_perm!(&RowMut<'_, E>, PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(&RowMut<'_, E>, Perm<I>, Row<E::Canonical>);
impl_perm!(&RowMut<'_, E>, &PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(&RowMut<'_, E>, &Perm<I>, Row<E::Canonical>);

impl_perm!(Row<E>, PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(Row<E>, Perm<I>, Row<E::Canonical>);
impl_perm!(Row<E>, &PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(Row<E>, &Perm<I>, Row<E::Canonical>);
impl_perm!(&Row<E>, PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(&Row<E>, Perm<I>, Row<E::Canonical>);
impl_perm!(&Row<E>, &PermRef<'_, I>, Row<E::Canonical>);
impl_perm!(&Row<E>, &Perm<I>, Row<E::Canonical>);

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<Scale<RhsE>> for MatRef<'_, LhsE>
{
    type Output = Mat<E>;

    fn mul(self, rhs: Scale<RhsE>) -> Self::Output {
        zipped!(self).map(|unzipped!(x)| x.read().canonicalize().faer_mul(rhs.0.canonicalize()))
    }
}
impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<MatRef<'_, RhsE>> for Scale<LhsE>
{
    type Output = Mat<E>;

    fn mul(self, rhs: MatRef<'_, RhsE>) -> Self::Output {
        zipped!(rhs).map(|unzipped!(x)| x.read().canonicalize().faer_mul(self.0.canonicalize()))
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<Scale<RhsE>> for ColRef<'_, LhsE>
{
    type Output = Col<E>;

    fn mul(self, rhs: Scale<RhsE>) -> Self::Output {
        zipped!(self).map(|unzipped!(x)| x.read().canonicalize().faer_mul(rhs.0.canonicalize()))
    }
}
impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<ColRef<'_, RhsE>> for Scale<LhsE>
{
    type Output = Col<E>;

    fn mul(self, rhs: ColRef<'_, RhsE>) -> Self::Output {
        zipped!(rhs).map(|unzipped!(x)| x.read().canonicalize().faer_mul(self.0.canonicalize()))
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<Scale<RhsE>> for RowRef<'_, LhsE>
{
    type Output = Row<E>;

    fn mul(self, rhs: Scale<RhsE>) -> Self::Output {
        zipped!(self).map(|unzipped!(x)| x.read().canonicalize().faer_mul(rhs.0.canonicalize()))
    }
}
impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<RowRef<'_, RhsE>> for Scale<LhsE>
{
    type Output = Row<E>;

    fn mul(self, rhs: RowRef<'_, RhsE>) -> Self::Output {
        zipped!(rhs).map(|unzipped!(x)| x.read().canonicalize().faer_mul(self.0.canonicalize()))
    }
}

impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<Scale<RhsE>> for DiagRef<'_, LhsE>
{
    type Output = Diag<E>;

    fn mul(self, rhs: Scale<RhsE>) -> Self::Output {
        zipped!(self.column_vector())
            .map(|unzipped!(x)| x.read().canonicalize().faer_mul(rhs.0.canonicalize()))
            .column_vector_into_diagonal()
    }
}
impl<E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<DiagRef<'_, RhsE>> for Scale<LhsE>
{
    type Output = Diag<E>;

    fn mul(self, rhs: DiagRef<'_, RhsE>) -> Self::Output {
        zipped!(rhs.column_vector())
            .map(|unzipped!(x)| x.read().canonicalize().faer_mul(self.0.canonicalize()))
            .column_vector_into_diagonal()
    }
}

// impl_mul_scalar!(MatRef<'_, LhsE>, Scale<RhsE>, Mat<E>);
impl_mul_scalar!(MatMut<'_, LhsE>, Scale<RhsE>, Mat<E>);
impl_mul_scalar!(Mat<LhsE>, Scale<RhsE>, Mat<E>);
impl_mul_scalar!(&MatRef<'_, LhsE>, Scale<RhsE>, Mat<E>);
impl_mul_scalar!(&MatMut<'_, LhsE>, Scale<RhsE>, Mat<E>);
impl_mul_scalar!(&Mat<LhsE>, Scale<RhsE>, Mat<E>);

// impl_scalar_mul!(Scale<LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_scalar_mul!(Scale<LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_scalar_mul!(Scale<LhsE>, Mat<RhsE>, Mat<E>);
impl_scalar_mul!(Scale<LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_scalar_mul!(Scale<LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_scalar_mul!(Scale<LhsE>, &Mat<RhsE>, Mat<E>);

// impl_mul_scalar!(ColRef<'_, LhsE>, Scale<RhsE>, Col<E>);
impl_mul_scalar!(ColMut<'_, LhsE>, Scale<RhsE>, Col<E>);
impl_mul_scalar!(Col<LhsE>, Scale<RhsE>, Col<E>);
impl_mul_scalar!(&ColRef<'_, LhsE>, Scale<RhsE>, Col<E>);
impl_mul_scalar!(&ColMut<'_, LhsE>, Scale<RhsE>, Col<E>);
impl_mul_scalar!(&Col<LhsE>, Scale<RhsE>, Col<E>);

// impl_scalar_mul!(Scale<LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_scalar_mul!(Scale<LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_scalar_mul!(Scale<LhsE>, Col<RhsE>, Col<E>);
impl_scalar_mul!(Scale<LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_scalar_mul!(Scale<LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_scalar_mul!(Scale<LhsE>, &Col<RhsE>, Col<E>);

// impl_mul_scalar!(RowRef<'_, LhsE>, Scale<RhsE>, Row<E>);
impl_mul_scalar!(RowMut<'_, LhsE>, Scale<RhsE>, Row<E>);
impl_mul_scalar!(Row<LhsE>, Scale<RhsE>, Row<E>);
impl_mul_scalar!(&RowRef<'_, LhsE>, Scale<RhsE>, Row<E>);
impl_mul_scalar!(&RowMut<'_, LhsE>, Scale<RhsE>, Row<E>);
impl_mul_scalar!(&Row<LhsE>, Scale<RhsE>, Row<E>);

// impl_scalar_mul!(Scale<LhsE>, RowRef<'_, RhsE>, Row<E>);
impl_scalar_mul!(Scale<LhsE>, RowMut<'_, RhsE>, Row<E>);
impl_scalar_mul!(Scale<LhsE>, Row<RhsE>, Row<E>);
impl_scalar_mul!(Scale<LhsE>, &RowRef<'_, RhsE>, Row<E>);
impl_scalar_mul!(Scale<LhsE>, &RowMut<'_, RhsE>, Row<E>);
impl_scalar_mul!(Scale<LhsE>, &Row<RhsE>, Row<E>);

// impl_mul_scalar!(DiagRef<'_, LhsE>, Scale<RhsE>, Diag<E>);
impl_mul_scalar!(DiagMut<'_, LhsE>, Scale<RhsE>, Diag<E>);
impl_mul_scalar!(Diag<LhsE>, Scale<RhsE>, Diag<E>);
impl_mul_scalar!(&DiagRef<'_, LhsE>, Scale<RhsE>, Diag<E>);
impl_mul_scalar!(&DiagMut<'_, LhsE>, Scale<RhsE>, Diag<E>);
impl_mul_scalar!(&Diag<LhsE>, Scale<RhsE>, Diag<E>);

// impl_scalar_mul!(Scale<LhsE>, DiagRef<'_, RhsE>, Diag<E>);
impl_scalar_mul!(Scale<LhsE>, DiagMut<'_, RhsE>, Diag<E>);
impl_scalar_mul!(Scale<LhsE>, Diag<RhsE>, Diag<E>);
impl_scalar_mul!(Scale<LhsE>, &DiagRef<'_, RhsE>, Diag<E>);
impl_scalar_mul!(Scale<LhsE>, &DiagMut<'_, RhsE>, Diag<E>);
impl_scalar_mul!(Scale<LhsE>, &Diag<RhsE>, Diag<E>);

impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> MulAssign<Scale<RhsE>>
    for MatMut<'_, LhsE>
{
    fn mul_assign(&mut self, rhs: Scale<RhsE>) {
        zipped!(self.as_mut())
            .for_each(|unzipped!(mut x)| x.write(x.read().faer_mul(rhs.0.canonicalize())))
    }
}
impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> MulAssign<Scale<RhsE>>
    for ColMut<'_, LhsE>
{
    fn mul_assign(&mut self, rhs: Scale<RhsE>) {
        zipped!(self.as_mut())
            .for_each(|unzipped!(mut x)| x.write(x.read().faer_mul(rhs.0.canonicalize())))
    }
}
impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> MulAssign<Scale<RhsE>>
    for RowMut<'_, LhsE>
{
    fn mul_assign(&mut self, rhs: Scale<RhsE>) {
        zipped!(self.as_mut())
            .for_each(|unzipped!(mut x)| x.write(x.read().faer_mul(rhs.0.canonicalize())))
    }
}
impl<LhsE: ComplexField, RhsE: Conjugate<Canonical = LhsE>> MulAssign<Scale<RhsE>>
    for DiagMut<'_, LhsE>
{
    fn mul_assign(&mut self, rhs: Scale<RhsE>) {
        zipped!(self.as_mut().column_vector_mut())
            .for_each(|unzipped!(mut x)| x.write(x.read().faer_mul(rhs.0.canonicalize())))
    }
}

// impl_mul_assign_scalar!(MatMut<'_, LhsE>, Scale<RhsE>);
impl_mul_assign_scalar!(Mat<LhsE>, Scale<RhsE>);
// impl_mul_assign_scalar!(ColMut<'_, LhsE>, Scale<RhsE>);
impl_mul_assign_scalar!(Col<LhsE>, Scale<RhsE>);
// impl_mul_assign_scalar!(RowMut<'_, LhsE>, Scale<RhsE>);
impl_mul_assign_scalar!(Row<LhsE>, Scale<RhsE>);
// impl_mul_assign_scalar!(DiagMut<'_, LhsE>, Scale<RhsE>);
impl_mul_assign_scalar!(Diag<LhsE>, Scale<RhsE>);

impl<I: Index, E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<MatRef<'_, RhsE>> for SparseColMatRef<'_, I, LhsE>
{
    type Output = Mat<E>;

    #[track_caller]
    fn mul(self, rhs: MatRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        let mut out = Mat::zeros(lhs.nrows(), rhs.ncols());
        crate::sparse::linalg::matmul::sparse_dense_matmul(
            out.as_mut(),
            lhs,
            rhs,
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<I: Index, E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<ColRef<'_, RhsE>> for SparseColMatRef<'_, I, LhsE>
{
    type Output = Col<E>;

    #[track_caller]
    fn mul(self, rhs: ColRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        let mut out = Col::zeros(lhs.nrows());
        crate::sparse::linalg::matmul::sparse_dense_matmul(
            out.as_mut().as_2d_mut(),
            lhs,
            rhs.as_2d(),
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<I: Index, E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<SparseColMatRef<'_, I, RhsE>> for MatRef<'_, LhsE>
{
    type Output = Mat<E>;

    #[track_caller]
    fn mul(self, rhs: SparseColMatRef<'_, I, RhsE>) -> Self::Output {
        let lhs = self;
        let mut out = Mat::zeros(lhs.nrows(), rhs.ncols());
        crate::sparse::linalg::matmul::dense_sparse_matmul(
            out.as_mut(),
            lhs,
            rhs,
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<I: Index, E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<SparseColMatRef<'_, I, RhsE>> for RowRef<'_, LhsE>
{
    type Output = Row<E>;

    #[track_caller]
    fn mul(self, rhs: SparseColMatRef<'_, I, RhsE>) -> Self::Output {
        let lhs = self;
        let mut out = Row::zeros(rhs.ncols());
        crate::sparse::linalg::matmul::dense_sparse_matmul(
            out.as_mut().as_2d_mut(),
            lhs.as_2d(),
            rhs,
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<I: Index, E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<MatRef<'_, RhsE>> for SparseRowMatRef<'_, I, LhsE>
{
    type Output = Mat<E>;

    #[track_caller]
    fn mul(self, rhs: MatRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        let mut out = Mat::zeros(lhs.nrows(), rhs.ncols());
        crate::sparse::linalg::matmul::dense_sparse_matmul(
            out.as_mut().transpose_mut(),
            rhs.transpose(),
            lhs.transpose(),
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<I: Index, E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<ColRef<'_, RhsE>> for SparseRowMatRef<'_, I, LhsE>
{
    type Output = Col<E>;

    #[track_caller]
    fn mul(self, rhs: ColRef<'_, RhsE>) -> Self::Output {
        let lhs = self;
        let mut out = Col::zeros(lhs.nrows());
        crate::sparse::linalg::matmul::dense_sparse_matmul(
            out.as_mut().transpose_mut().as_2d_mut(),
            rhs.transpose().as_2d(),
            lhs.transpose(),
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<I: Index, E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<SparseRowMatRef<'_, I, RhsE>> for MatRef<'_, LhsE>
{
    type Output = Mat<E>;

    #[track_caller]
    fn mul(self, rhs: SparseRowMatRef<'_, I, RhsE>) -> Self::Output {
        let lhs = self;
        let mut out = Mat::zeros(lhs.nrows(), rhs.ncols());
        crate::sparse::linalg::matmul::sparse_dense_matmul(
            out.as_mut().transpose_mut(),
            rhs.transpose(),
            lhs.transpose(),
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

impl<I: Index, E: ComplexField, LhsE: Conjugate<Canonical = E>, RhsE: Conjugate<Canonical = E>>
    Mul<SparseRowMatRef<'_, I, RhsE>> for RowRef<'_, LhsE>
{
    type Output = Row<E>;

    #[track_caller]
    fn mul(self, rhs: SparseRowMatRef<'_, I, RhsE>) -> Self::Output {
        let lhs = self;
        let mut out = Row::zeros(rhs.ncols());
        crate::sparse::linalg::matmul::sparse_dense_matmul(
            out.as_mut().transpose_mut().as_2d_mut(),
            rhs.transpose(),
            lhs.transpose().as_2d(),
            None,
            E::faer_one(),
            get_global_parallelism(),
        );
        out
    }
}

// impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, &Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, &Mat<RhsE>, Mat<E>);

impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, &Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, &Mat<RhsE>, Mat<E>);

impl_sparse_mul!(SparseColMat<I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, &Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, &Mat<RhsE>, Mat<E>);

// impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, &Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, &Mat<RhsE>, Mat<E>);

impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, &Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, &Mat<RhsE>, Mat<E>);

impl_sparse_mul!(SparseRowMat<I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, &Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, Mat<RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, &MatRef<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, &MatMut<'_, RhsE>, Mat<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, &Mat<RhsE>, Mat<E>);

// impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatRef<'_, I, LhsE>, &Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatRef<'_, I, LhsE>, &Col<RhsE>, Col<E>);

impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMatMut<'_, I, LhsE>, &Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMatMut<'_, I, LhsE>, &Col<RhsE>, Col<E>);

impl_sparse_mul!(SparseColMat<I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseColMat<I, LhsE>, &Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseColMat<I, LhsE>, &Col<RhsE>, Col<E>);

// impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatRef<'_, I, LhsE>, &Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatRef<'_, I, LhsE>, &Col<RhsE>, Col<E>);

impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMatMut<'_, I, LhsE>, &Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMatMut<'_, I, LhsE>, &Col<RhsE>, Col<E>);

impl_sparse_mul!(SparseRowMat<I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(SparseRowMat<I, LhsE>, &Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, Col<RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, &ColRef<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, &ColMut<'_, RhsE>, Col<E>);
impl_sparse_mul!(&SparseRowMat<I, LhsE>, &Col<RhsE>, Col<E>);

// impl_sparse_mul!(MatRef<'_, LhsE>, SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, SparseColMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, &SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, &SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, &SparseColMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, SparseColMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, &SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, &SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, &SparseColMat<I, RhsE>, Mat<E>);

impl_sparse_mul!(MatMut<'_, LhsE>, SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, SparseColMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, &SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, &SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, &SparseColMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, SparseColMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, &SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, &SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, &SparseColMat<I, RhsE>, Mat<E>);

impl_sparse_mul!(Mat<LhsE>, SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat<LhsE>, SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat< LhsE>, SparseColMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat<LhsE>, &SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat<LhsE>, &SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat<LhsE>, &SparseColMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, SparseColMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, &SparseColMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, &SparseColMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, &SparseColMat<I, RhsE>, Mat<E>);

// impl_sparse_mul!(RowRef<'_, LhsE>, SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, SparseColMat<I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, &SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, &SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, &SparseColMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, SparseColMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, &SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, &SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, &SparseColMat<I, RhsE>, Row<E>);

impl_sparse_mul!(RowMut<'_, LhsE>, SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, SparseColMat<I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, &SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, &SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, &SparseColMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, SparseColMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, &SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, &SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, &SparseColMat<I, RhsE>, Row<E>);

impl_sparse_mul!(Row<LhsE>, SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, SparseColMat<I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, &SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, &SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, &SparseColMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, SparseColMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, &SparseColMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, &SparseColMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, &SparseColMat<I, RhsE>, Row<E>);

// impl_sparse_mul!(MatRef<'_, LhsE>, SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, SparseRowMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, &SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, &SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatRef<'_, LhsE>, &SparseRowMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, SparseRowMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, &SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, &SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatRef<'_, LhsE>, &SparseRowMat<I, RhsE>, Mat<E>);

impl_sparse_mul!(MatMut<'_, LhsE>, SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, SparseRowMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, &SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, &SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(MatMut<'_, LhsE>, &SparseRowMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, SparseRowMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, &SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, &SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&MatMut<'_, LhsE>, &SparseRowMat<I, RhsE>, Mat<E>);

impl_sparse_mul!(Mat<LhsE>, SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat<LhsE>, SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat< LhsE>, SparseRowMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat<LhsE>, &SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat<LhsE>, &SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(Mat<LhsE>, &SparseRowMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, SparseRowMat<I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, &SparseRowMatRef<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, &SparseRowMatMut<'_, I, RhsE>, Mat<E>);
impl_sparse_mul!(&Mat<LhsE>, &SparseRowMat<I, RhsE>, Mat<E>);

// impl_sparse_mul!(RowRef<'_, LhsE>, SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, SparseRowMat<I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, &SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, &SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowRef<'_, LhsE>, &SparseRowMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, SparseRowMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, &SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, &SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowRef<'_, LhsE>, &SparseRowMat<I, RhsE>, Row<E>);

impl_sparse_mul!(RowMut<'_, LhsE>, SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, SparseRowMat<I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, &SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, &SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(RowMut<'_, LhsE>, &SparseRowMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, SparseRowMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, &SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, &SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&RowMut<'_, LhsE>, &SparseRowMat<I, RhsE>, Row<E>);

impl_sparse_mul!(Row<LhsE>, SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, SparseRowMat<I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, &SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, &SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(Row<LhsE>, &SparseRowMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, SparseRowMat<I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, &SparseRowMatRef<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, &SparseRowMatMut<'_, I, RhsE>, Row<E>);
impl_sparse_mul!(&Row<LhsE>, &SparseRowMat<I, RhsE>, Row<E>);

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use crate::mat;
    use crate::{assert, col::*, mat::*, perm::*, row::*};
    use assert_approx_eq::assert_approx_eq;

    fn matrices() -> (Mat<f64>, Mat<f64>) {
        let A = mat![[2.8, -3.3], [-1.7, 5.2], [4.6, -8.3],];

        let B = mat![[-7.9, 8.3], [4.7, -3.2], [3.8, -5.2],];
        (A, B)
    }

    #[test]
    #[should_panic]
    fn test_adding_matrices_of_different_sizes_should_panic() {
        let A = mat![[1.0, 2.0], [3.0, 4.0]];
        let B = mat![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        _ = A + B;
    }

    #[test]
    #[should_panic]
    fn test_subtracting_two_matrices_of_different_sizes_should_panic() {
        let A = mat![[1.0, 2.0], [3.0, 4.0]];
        let B = mat![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        _ = A - B;
    }

    #[test]
    fn test_add() {
        let (A, B) = matrices();

        let expected = mat![[-5.1, 5.0], [3.0, 2.0], [8.4, -13.5],];

        assert_matrix_approx_eq(A.as_ref() + B.as_ref(), &expected);
        assert_matrix_approx_eq(&A + &B, &expected);
        assert_matrix_approx_eq(A.as_ref() + &B, &expected);
        assert_matrix_approx_eq(&A + B.as_ref(), &expected);
        assert_matrix_approx_eq(A.as_ref() + B.clone(), &expected);
        assert_matrix_approx_eq(&A + B.clone(), &expected);
        assert_matrix_approx_eq(A.clone() + B.as_ref(), &expected);
        assert_matrix_approx_eq(A.clone() + &B, &expected);
        assert_matrix_approx_eq(A + B, &expected);
    }

    #[test]
    fn test_sub() {
        let (A, B) = matrices();

        let expected = mat![[10.7, -11.6], [-6.4, 8.4], [0.8, -3.1],];

        assert_matrix_approx_eq(A.as_ref() - B.as_ref(), &expected);
        assert_matrix_approx_eq(&A - &B, &expected);
        assert_matrix_approx_eq(A.as_ref() - &B, &expected);
        assert_matrix_approx_eq(&A - B.as_ref(), &expected);
        assert_matrix_approx_eq(A.as_ref() - B.clone(), &expected);
        assert_matrix_approx_eq(&A - B.clone(), &expected);
        assert_matrix_approx_eq(A.clone() - B.as_ref(), &expected);
        assert_matrix_approx_eq(A.clone() - &B, &expected);
        assert_matrix_approx_eq(A - B, &expected);
    }

    #[test]
    fn test_neg() {
        let (A, _) = matrices();

        let expected = mat![[-2.8, 3.3], [1.7, -5.2], [-4.6, 8.3],];

        assert_eq!(-A, expected);
    }

    #[test]
    fn test_scalar_mul() {
        use crate::scale;

        let (A, _) = matrices();
        let scale = scale(3.0);
        let expected = Mat::from_fn(A.nrows(), A.ncols(), |i, j| A.read(i, j) * scale.value());

        {
            assert_matrix_approx_eq(A.as_ref() * scale, &expected);
            assert_matrix_approx_eq(&A * scale, &expected);
            assert_matrix_approx_eq(A.as_ref() * scale, &expected);
            assert_matrix_approx_eq(&A * scale, &expected);
            assert_matrix_approx_eq(A.as_ref() * scale, &expected);
            assert_matrix_approx_eq(&A * scale, &expected);
            assert_matrix_approx_eq(A.clone() * scale, &expected);
            assert_matrix_approx_eq(A.clone() * scale, &expected);
            assert_matrix_approx_eq(A * scale, &expected);
        }

        let (A, _) = matrices();
        {
            assert_matrix_approx_eq(scale * A.as_ref(), &expected);
            assert_matrix_approx_eq(scale * &A, &expected);
            assert_matrix_approx_eq(scale * A.as_ref(), &expected);
            assert_matrix_approx_eq(scale * &A, &expected);
            assert_matrix_approx_eq(scale * A.as_ref(), &expected);
            assert_matrix_approx_eq(scale * &A, &expected);
            assert_matrix_approx_eq(scale * A.clone(), &expected);
            assert_matrix_approx_eq(scale * A.clone(), &expected);
            assert_matrix_approx_eq(scale * A, &expected);
        }
    }

    #[test]
    fn test_diag_mul() {
        let (A, _) = matrices();
        let diag_left = mat![[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]];
        let diag_right = mat![[4.0, 0.0], [0.0, 5.0]];

        assert!(&diag_left * &A == diag_left.diagonal() * &A);
        assert!(&A * &diag_right == &A * diag_right.diagonal());
    }

    #[test]
    fn test_perm_mul() {
        let A = Mat::from_fn(6, 5, |i, j| (j + 5 * i) as f64);
        let pl =
            Perm::<usize>::new_checked(Box::new([5, 1, 4, 0, 2, 3]), Box::new([3, 1, 4, 5, 2, 0]));
        let pr = Perm::<usize>::new_checked(Box::new([1, 4, 0, 2, 3]), Box::new([2, 0, 3, 4, 1]));

        let perm_left = mat![
            [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        ];
        let perm_right = mat![
            [0.0, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0],
        ];

        assert!(
            &pl * pl.as_ref().inverse()
                == PermRef::<'_, usize>::new_checked(&[0, 1, 2, 3, 4, 5], &[0, 1, 2, 3, 4, 5],)
        );
        assert!(&perm_left * &A == &pl * &A);
        assert!(&A * &perm_right == &A * &pr);
    }

    #[test]
    fn test_matmul_col_row() {
        let A = Col::from_fn(6, |i| i as f64);
        let B = Row::from_fn(6, |j| (5 * j + 1) as f64);

        // outer product
        assert_eq!(&A * &B, A.as_ref().as_2d() * B.as_ref().as_2d());
        // inner product
        assert_eq!(
            &B * &A,
            (B.as_ref().as_2d() * A.as_ref().as_2d()).read(0, 0),
        );
    }

    fn assert_matrix_approx_eq(given: Mat<f64>, expected: &Mat<f64>) {
        assert_eq!(given.nrows(), expected.nrows());
        assert_eq!(given.ncols(), expected.ncols());
        for i in 0..given.nrows() {
            for j in 0..given.ncols() {
                assert_approx_eq!(given.read(i, j), expected.read(i, j));
            }
        }
    }
}
