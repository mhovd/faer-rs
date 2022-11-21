use faer_core::{solve, ComplexField, Conj, MatMut, MatRef, Parallelism};
use reborrow::*;

use assert2::assert as fancy_assert;

#[track_caller]
pub fn solve_in_place<T: ComplexField>(
    cholesky_factors: MatRef<'_, T>,
    conj_lhs: Conj,
    rhs: MatMut<'_, T>,
    conj_rhs: Conj,
    parallelism: Parallelism,
) {
    let n = cholesky_factors.nrows();

    fancy_assert!(cholesky_factors.nrows() == cholesky_factors.ncols());
    fancy_assert!(rhs.nrows() == n);

    let mut rhs = rhs;

    solve::solve_lower_triangular_in_place(
        cholesky_factors,
        conj_lhs,
        rhs.rb_mut(),
        conj_rhs,
        parallelism,
    );

    solve::solve_upper_triangular_in_place(
        cholesky_factors.transpose(),
        match conj_lhs {
            Conj::No => Conj::Yes,
            Conj::Yes => Conj::No,
        },
        rhs.rb_mut(),
        Conj::No,
        parallelism,
    );
}