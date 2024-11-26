//! Block Householder transformations.
//!
//! A Householder reflection is linear transformation that describes a reflection about a
//! hyperplane that crosses the origin of the space.
//!
//! Let $v$ be a unit vector that is orthogonal to the hyperplane. Then the corresponding
//! Householder transformation in matrix form is $I - 2vv^H$, where $I$ is the identity matrix.
//!
//! In practice, a non unit vector $v$ is used, so the transformation is written as
//! $$H = I - \frac{vv^H}{\tau}.$$
//!
//! A block Householder transformation is a sequence of such transformations
//! $H_0, H_1, \dots, H_{b -1 }$ applied one after the other, with the restriction that the first
//! $i$ components of the vector $v_i$ of the $i$-th transformation are zero, and the component at
//! index $i$ is one.
//!
//! The matrix $V = [v_0\ v_1\ \dots\ v_{b-1}]$ is thus a lower trapezoidal matrix with unit
//! diagonal. We call it the Householder basis.
//!
//! There exists a unique upper triangular matrix $T$, that we call the Householder factor, such
//! that $$H_0 \times \dots \times H_{b-1} = I - VT^{-1}V^H.$$
//!
//! A block Householder sequence is a sequence of such transformations, composed of two matrices:
//! - a lower trapezoidal matrix with unit diagonal, which is the horizontal concatenation of the
//! bases of each block Householder transformation,
//! - a horizontal concatenation of the Householder factors.
//!
//! Examples on how to create and manipulate block Householder sequences are provided in the
//! documentation of the QR module.

use crate::{
    assert,
    linalg::{
        matmul::{
            dot, matmul, matmul_with_conj,
            triangular::{self, BlockStructure},
        },
        triangular_solve as solve,
    },
    utils::{simd::SimdCtx, thread::join_raw},
    ContiguousFwd, Stride,
};

use crate::internal_prelude::*;

/// Computes the Householder reflection $I - \frac{v v^H}{\tau}$ such that when multiplied by $x$
/// from the left, The result is $\beta e_0$. $\tau$ and $\beta$ are returned and $\tau$ is
/// real-valued.
///
/// $x$ is determined by $x_0$, contained in `head`, and $|x_{1\dots}|$, contained in `tail_norm`.
/// The vector $v$ is such that $v_0 = 1$ and $v_{1\dots}$ is stored in `essential` (when provided).
#[math]
#[inline]
pub fn make_householder_in_place<M: Shape, T: ComplexField>(
    head: &mut T,
    tail: ColMut<'_, T, M>,
) -> (T, Option<T>) {
    #[inline]
    pub fn imp<'M, T: ComplexField>(head: &mut T, tail: ColMut<'_, T, Dim<'M>>) -> (T, Option<T>) {
        let tail_norm = tail.norm_l2();

        let mut head_norm = abs(*head);
        if head_norm < min_positive() {
            *head = zero();
            head_norm = zero();
        }

        if tail_norm < min_positive() {
            return (infinity(), None);
        }

        let one_half = from_f64(0.5);

        let norm = hypot(head_norm, tail_norm);

        let sign = if head_norm != zero() {
            mul_real(*head, recip(head_norm))
        } else {
            one()
        };

        let signed_norm = sign * from_real(norm);
        let head_with_beta = *head + signed_norm;
        let head_with_beta_inv = recip(head_with_beta);

        zipped!(tail).for_each(|unzipped!(e)| {
            *e = e * head_with_beta_inv;
        });

        *head = -signed_norm;

        let tau = one_half * (one() + abs2(tail_norm * abs(head_with_beta_inv)));
        (from_real(tau), head_with_beta_inv.into())
    }

    imp(head, tail.bind_r(unique!()))
}

#[doc(hidden)]
#[math]
pub fn upgrade_householder_factor<T: ComplexField>(
    householder_factor: MatMut<'_, T>,
    essentials: MatRef<'_, T>,
    blocksize: usize,
    prev_blocksize: usize,
    par: Par,
) {
    assert!(all(
        householder_factor.nrows() == householder_factor.ncols(),
        essentials.ncols() == householder_factor.ncols(),
    ));

    if blocksize == prev_blocksize || householder_factor.nrows().unbound() <= prev_blocksize {
        return;
    }

    let n = essentials.ncols();
    let mut householder_factor = householder_factor;
    let essentials = essentials;

    assert!(householder_factor.nrows() == householder_factor.ncols());

    let block_count = householder_factor.nrows().div_ceil(blocksize);
    if block_count > 1 {
        assert!(all(
            blocksize > prev_blocksize,
            blocksize % prev_blocksize == 0,
        ));
        let mid = block_count / 2;

        let (tau_tl, _, _, tau_br) = householder_factor.split_at_mut(mid, mid);
        let (basis_left, basis_right) = essentials.split_at_col(mid);
        let basis_right = basis_right.split_at_row(mid).1;
        join_raw(
            |parallelism| {
                upgrade_householder_factor(
                    tau_tl,
                    basis_left,
                    blocksize,
                    prev_blocksize,
                    parallelism,
                )
            },
            |parallelism| {
                upgrade_householder_factor(
                    tau_br,
                    basis_right,
                    blocksize,
                    prev_blocksize,
                    parallelism,
                )
            },
            par,
        );
        return;
    }

    if prev_blocksize < 8 {
        // pretend that prev_blocksize == 1, recompute whole top half of matrix

        let (basis_top, basis_bot) = essentials.split_at_row(n);
        let acc_structure = BlockStructure::UnitTriangularUpper;

        triangular::matmul(
            householder_factor.rb_mut(),
            acc_structure,
            Accum::Replace,
            basis_top.adjoint(),
            BlockStructure::UnitTriangularUpper,
            basis_top,
            BlockStructure::UnitTriangularLower,
            one(),
            par,
        );
        triangular::matmul(
            householder_factor.rb_mut(),
            acc_structure,
            Accum::Add,
            basis_bot.adjoint(),
            BlockStructure::Rectangular,
            basis_bot,
            BlockStructure::Rectangular,
            one(),
            par,
        );
    } else {
        let prev_block_count = householder_factor.nrows().div_ceil(prev_blocksize);

        let mid = (prev_block_count / 2) * prev_blocksize;

        let (tau_tl, mut tau_tr, _, tau_br) = householder_factor.split_at_mut(mid, mid);
        let (basis_left, basis_right) = essentials.split_at_col(mid);
        let basis_right = basis_right.split_at_row(mid).1;

        join_raw(
            |parallelism| {
                join_raw(
                    |parallelism| {
                        upgrade_householder_factor(
                            tau_tl,
                            basis_left,
                            blocksize,
                            prev_blocksize,
                            parallelism,
                        )
                    },
                    |parallelism| {
                        upgrade_householder_factor(
                            tau_br,
                            basis_right,
                            blocksize,
                            prev_blocksize,
                            parallelism,
                        )
                    },
                    parallelism,
                );
            },
            |parallelism| {
                let basis_left = basis_left.split_at_row(mid).1;
                let row_mid = basis_right.ncols();

                let (basis_left_top, basis_left_bot) = basis_left.split_at_row(row_mid);
                let (basis_right_top, basis_right_bot) = basis_right.split_at_row(row_mid);

                triangular::matmul(
                    tau_tr.rb_mut(),
                    BlockStructure::Rectangular,
                    Accum::Replace,
                    basis_left_top.adjoint(),
                    BlockStructure::Rectangular,
                    basis_right_top,
                    BlockStructure::UnitTriangularLower,
                    one(),
                    parallelism,
                );
                matmul(
                    tau_tr.rb_mut(),
                    Accum::Add,
                    basis_left_bot.adjoint(),
                    basis_right_bot,
                    one(),
                    parallelism,
                );
            },
            par,
        );
    }
}

/// Computes the size and alignment of required workspace for applying a block Householder
/// transformation to a right-hand-side matrix in place.
pub fn apply_block_householder_on_the_left_in_place_scratch<T: ComplexField>(
    householder_basis_nrows: usize,
    blocksize: usize,
    rhs_ncols: usize,
) -> Result<StackReq, SizeOverflow> {
    let _ = householder_basis_nrows;
    temp_mat_scratch::<T>(blocksize, rhs_ncols)
}

/// Computes the size and alignment of required workspace for applying the transpose of a block
/// Householder transformation to a right-hand-side matrix in place.
pub fn apply_block_householder_transpose_on_the_left_in_place_scratch<T: ComplexField>(
    householder_basis_nrows: usize,
    blocksize: usize,
    rhs_ncols: usize,
) -> Result<StackReq, SizeOverflow> {
    let _ = householder_basis_nrows;
    temp_mat_scratch::<T>(blocksize, rhs_ncols)
}

/// Computes the size and alignment of required workspace for applying a block Householder
/// transformation to a left-hand-side matrix in place.
pub fn apply_block_householder_on_the_right_in_place_scratch<T: ComplexField>(
    householder_basis_nrows: usize,
    blocksize: usize,
    lhs_nrows: usize,
) -> Result<StackReq, SizeOverflow> {
    let _ = householder_basis_nrows;
    temp_mat_scratch::<T>(blocksize, lhs_nrows)
}

/// Computes the size and alignment of required workspace for applying the transpose of a block
/// Householder transformation to a left-hand-side matrix in place.
pub fn apply_block_householder_transpose_on_the_right_in_place_scratch<T: ComplexField>(
    householder_basis_nrows: usize,
    blocksize: usize,
    lhs_nrows: usize,
) -> Result<StackReq, SizeOverflow> {
    let _ = householder_basis_nrows;
    temp_mat_scratch::<T>(blocksize, lhs_nrows)
}

/// Computes the size and alignment of required workspace for applying the transpose of a sequence
/// of block Householder transformations to a right-hand-side matrix in place.
pub fn apply_block_householder_sequence_transpose_on_the_left_in_place_scratch<T: ComplexField>(
    householder_basis_nrows: usize,
    blocksize: usize,
    rhs_ncols: usize,
) -> Result<StackReq, SizeOverflow> {
    let _ = householder_basis_nrows;
    temp_mat_scratch::<T>(blocksize, rhs_ncols)
}

/// Computes the size and alignment of required workspace for applying a sequence of block
/// Householder transformations to a right-hand-side matrix in place.
pub fn apply_block_householder_sequence_on_the_left_in_place_scratch<T: ComplexField>(
    householder_basis_nrows: usize,
    blocksize: usize,
    rhs_ncols: usize,
) -> Result<StackReq, SizeOverflow> {
    let _ = householder_basis_nrows;
    temp_mat_scratch::<T>(blocksize, rhs_ncols)
}

/// Computes the size and alignment of required workspace for applying the transpose of a sequence
/// of block Householder transformations to a left-hand-side matrix in place.
pub fn apply_block_householder_sequence_transpose_on_the_right_in_place_scratch<T: ComplexField>(
    householder_basis_nrows: usize,
    blocksize: usize,
    lhs_nrows: usize,
) -> Result<StackReq, SizeOverflow> {
    let _ = householder_basis_nrows;
    temp_mat_scratch::<T>(blocksize, lhs_nrows)
}

/// Computes the size and alignment of required workspace for applying a sequence of block
/// Householder transformations to a left-hand-side matrix in place.
pub fn apply_block_householder_sequence_on_the_right_in_place_scratch<T: ComplexField>(
    householder_basis_nrows: usize,
    blocksize: usize,
    lhs_nrows: usize,
) -> Result<StackReq, SizeOverflow> {
    let _ = householder_basis_nrows;
    temp_mat_scratch::<T>(blocksize, lhs_nrows)
}

#[track_caller]
#[math]
fn apply_block_householder_on_the_left_in_place_generic<'M, 'N, 'K, T: ComplexField>(
    householder_basis: MatRef<'_, T, Dim<'M>, Dim<'N>>,
    householder_factor: MatRef<'_, T, Dim<'N>, Dim<'N>>,
    conj_lhs: Conj,
    matrix: MatMut<'_, T, Dim<'M>, Dim<'K>>,
    forward: bool,
    par: Par,
    stack: &mut DynStack,
) {
    assert!(all(
        householder_factor.nrows() == householder_factor.ncols(),
        householder_basis.ncols() == householder_factor.nrows(),
        matrix.nrows() == householder_basis.nrows(),
    ));

    let mut matrix = matrix;

    let M = householder_basis.nrows();
    let N = householder_basis.ncols();

    make_guard!(TAIL);
    let midpoint = M.head_partition(N, TAIL);

    if let (Some(householder_basis), Some(matrix), 1, true) = (
        householder_basis.try_as_col_major(),
        matrix.rb_mut().try_as_col_major_mut(),
        N.unbound(),
        T::SIMD_CAPABILITIES.is_simd(),
    ) {
        let arch = T::Arch::default();

        struct ApplyOnLeft<'a, 'TAIL, 'K, T: ComplexField, const CONJ: bool> {
            tau_inv: &'a T,
            essential: ColRef<'a, T, Dim<'TAIL>, ContiguousFwd>,
            rhs0: RowMut<'a, T, Dim<'K>>,
            rhs: MatMut<'a, T, Dim<'TAIL>, Dim<'K>, ContiguousFwd>,
        }

        impl<'TAIL, 'K, T: ComplexField, const CONJ: bool> pulp::WithSimd
            for ApplyOnLeft<'_, 'TAIL, 'K, T, CONJ>
        {
            type Output = ();

            #[inline(always)]
            fn with_simd<S: pulp::Simd>(self, simd: S) -> Self::Output {
                let Self {
                    tau_inv,
                    essential,
                    mut rhs,
                    mut rhs0,
                } = self;

                if rhs.nrows().unbound() == 0 {
                    return;
                }

                let N = rhs.nrows();
                let K = rhs.ncols();
                let simd = SimdCtx::<T, S>::new(T::simd_ctx(simd), N);

                let (head, indices, tail) = simd.indices();

                for idx in K.indices() {
                    let col0 = rhs0.rb_mut().at_mut(idx);
                    let mut col = rhs.rb_mut().col_mut(idx);
                    let essential = essential;

                    let dot = if const { CONJ } {
                        *col0 + dot::inner_prod_no_conj_simd(simd, essential.rb(), col.rb())
                    } else {
                        *col0 + dot::inner_prod_conj_lhs_simd(simd, essential.rb(), col.rb())
                    };

                    let k = -dot * tau_inv;
                    *col0 = col0 + k;

                    let k = simd.splat(&k);
                    macro_rules! simd {
                        ($i: expr) => {{
                            let i = $i;
                            let mut a = simd.read(col.rb(), i);
                            let b = simd.read(essential.rb(), i);

                            if const { CONJ } {
                                a = simd.conj_mul_add(b, k, a);
                            } else {
                                a = simd.mul_add(b, k, a);
                            }

                            simd.write(col.rb_mut(), i, a);
                        }};
                    }

                    if let Some(i) = head {
                        simd!(i);
                    }
                    for i in indices.clone() {
                        simd!(i);
                    }
                    if let Some(i) = tail {
                        simd!(i);
                    }
                }
            }
        }

        let N0 = N.check(0);

        let essential = householder_basis.col(N0).split_rows_with(midpoint).1;
        let (rhs0, rhs) = matrix.split_rows_with_mut(midpoint);
        let rhs0 = rhs0.row_mut(N0);

        let tau_inv: T = from_real(recip(real(householder_factor[(N0, N0)])));

        if const { T::IS_REAL } || matches!(conj_lhs, Conj::No) {
            arch.dispatch(ApplyOnLeft::<_, false> {
                tau_inv: &tau_inv,
                essential,
                rhs,
                rhs0,
            });
        } else {
            arch.dispatch(ApplyOnLeft::<_, true> {
                tau_inv: &tau_inv,
                essential,
                rhs,
                rhs0,
            });
        }
    } else {
        let (essentials_top, essentials_bot) = householder_basis.split_rows_with(midpoint);
        let M = matrix.nrows();
        let K = matrix.ncols();

        // essentials* × mat
        let (mut tmp, _) = unsafe { temp_mat_uninit::<T, _, _>(N, K, stack) };
        let mut tmp = tmp.as_mat_mut();

        let mut n_tasks = Ord::min(
            Ord::min(crate::utils::thread::parallelism_degree(par), K.unbound()),
            4,
        );
        if (M.unbound() * K.unbound()).saturating_mul(4 * M.unbound())
            < gemm::get_threading_threshold()
        {
            n_tasks = 1;
        }

        let inner_parallelism = match par {
            Par::Seq => Par::Seq,
            #[cfg(feature = "rayon")]
            Par::Rayon(par) => {
                let par = par.get();

                if par >= 2 * n_tasks {
                    Par::rayon(par / n_tasks)
                } else {
                    Par::Seq
                }
            }
        };

        let func = |(mut tmp, mut matrix): (MatMut<'_, T, Dim<'N>>, MatMut<'_, T, Dim<'M>>)| {
            let (mut top, mut bot) = matrix.rb_mut().split_rows_with_mut(midpoint);

            triangular::matmul_with_conj(
                tmp.rb_mut(),
                BlockStructure::Rectangular,
                Accum::Replace,
                essentials_top.transpose(),
                BlockStructure::UnitTriangularUpper,
                Conj::Yes.compose(conj_lhs),
                top.rb(),
                BlockStructure::Rectangular,
                Conj::No,
                one(),
                inner_parallelism,
            );

            matmul_with_conj(
                tmp.rb_mut(),
                Accum::Add,
                essentials_bot.transpose(),
                Conj::Yes.compose(conj_lhs),
                bot.rb(),
                Conj::No,
                one(),
                inner_parallelism,
            );

            // [T^-1|T^-*] × essentials* × tmp
            if forward {
                solve::solve_lower_triangular_in_place_with_conj(
                    householder_factor.transpose(),
                    Conj::Yes.compose(conj_lhs),
                    tmp.rb_mut(),
                    inner_parallelism,
                );
            } else {
                solve::solve_upper_triangular_in_place_with_conj(
                    householder_factor,
                    Conj::No.compose(conj_lhs),
                    tmp.rb_mut(),
                    inner_parallelism,
                );
            }

            // essentials × [T^-1|T^-*] × essentials* × tmp
            triangular::matmul_with_conj(
                top.rb_mut(),
                BlockStructure::Rectangular,
                Accum::Add,
                essentials_top,
                BlockStructure::UnitTriangularLower,
                Conj::No.compose(conj_lhs),
                tmp.rb(),
                BlockStructure::Rectangular,
                Conj::No,
                -one(),
                inner_parallelism,
            );
            matmul_with_conj(
                bot.rb_mut(),
                Accum::Add,
                essentials_bot,
                Conj::No.compose(conj_lhs),
                tmp.rb(),
                Conj::No,
                -one(),
                inner_parallelism,
            );
        };

        if n_tasks <= 1 {
            func((tmp.as_dyn_cols_mut(), matrix.as_dyn_cols_mut()));
            return;
        } else {
            #[cfg(feature = "rayon")]
            {
                use rayon::prelude::*;
                tmp.rb_mut()
                    .par_col_partition_mut(n_tasks)
                    .zip_eq(matrix.rb_mut().par_col_partition_mut(n_tasks))
                    .for_each(func);
            }
        }
    }
}

/// Computes the product of the matrix, multiplied by the given block Householder transformation,
/// and stores the result in `matrix`.
#[track_caller]
pub fn apply_block_householder_on_the_right_in_place_with_conj<
    T: ComplexField,
    M: Shape,
    N: Shape,
    K: Shape,
>(
    householder_basis: MatRef<'_, T, M, N, impl Stride, impl Stride>,
    householder_factor: MatRef<'_, T, N, N, impl Stride, impl Stride>,
    conj_rhs: Conj,
    matrix: MatMut<'_, T, K, M, impl Stride, impl Stride>,
    par: Par,
    stack: &mut DynStack,
) {
    apply_block_householder_transpose_on_the_left_in_place_with_conj(
        householder_basis,
        householder_factor,
        conj_rhs,
        matrix.transpose_mut(),
        par,
        stack,
    )
}

/// Computes the product of the matrix, multiplied by the transpose of the given block Householder
/// transformation, and stores the result in `matrix`.
#[track_caller]
pub fn apply_block_householder_transpose_on_the_right_in_place_with_conj<
    M: Shape,
    N: Shape,
    K: Shape,
    T: ComplexField,
>(
    householder_basis: MatRef<'_, T, M, N, impl Stride, impl Stride>,
    householder_factor: MatRef<'_, T, N, N, impl Stride, impl Stride>,
    conj_rhs: Conj,
    matrix: MatMut<'_, T, K, M, impl Stride, impl Stride>,
    par: Par,
    stack: &mut DynStack,
) {
    apply_block_householder_on_the_left_in_place_with_conj(
        householder_basis,
        householder_factor,
        conj_rhs,
        matrix.transpose_mut(),
        par,
        stack,
    )
}

/// Computes the product of the given block Householder transformation, multiplied by `matrix`, and
/// stores the result in `matrix`.
#[track_caller]
pub fn apply_block_householder_on_the_left_in_place_with_conj<
    T: ComplexField,
    M: Shape,
    N: Shape,
    K: Shape,
>(
    householder_basis: MatRef<'_, T, M, N, impl Stride, impl Stride>,
    householder_factor: MatRef<'_, T, N, N, impl Stride, impl Stride>,
    conj_lhs: Conj,
    matrix: MatMut<'_, T, M, K, impl Stride, impl Stride>,
    par: Par,
    stack: &mut DynStack,
) {
    make_guard!(M);
    make_guard!(N);
    make_guard!(K);
    let M = householder_basis.nrows().bind(M);
    let N = householder_basis.ncols().bind(N);
    let K = matrix.ncols().bind(K);

    apply_block_householder_on_the_left_in_place_generic(
        householder_basis.as_shape(M, N).as_dyn_stride(),
        householder_factor.as_shape(N, N).as_dyn_stride(),
        conj_lhs,
        matrix.as_shape_mut(M, K).as_dyn_stride_mut(),
        false,
        par,
        stack,
    )
}

/// Computes the product of the transpose of the given block Householder transformation, multiplied
/// by `matrix`, and stores the result in `matrix`.
#[track_caller]
pub fn apply_block_householder_transpose_on_the_left_in_place_with_conj<
    T: ComplexField,
    M: Shape,
    N: Shape,
    K: Shape,
>(
    householder_basis: MatRef<'_, T, M, N, impl Stride, impl Stride>,
    householder_factor: MatRef<'_, T, N, N, impl Stride, impl Stride>,
    conj_lhs: Conj,
    matrix: MatMut<'_, T, M, K, impl Stride, impl Stride>,
    par: Par,
    stack: &mut DynStack,
) {
    make_guard!(M);
    make_guard!(N);
    make_guard!(K);
    let M = householder_basis.nrows().bind(M);
    let N = householder_basis.ncols().bind(N);
    let K = matrix.ncols().bind(K);

    apply_block_householder_on_the_left_in_place_generic(
        householder_basis.as_shape(M, N).as_dyn_stride(),
        householder_factor.as_shape(N, N).as_dyn_stride(),
        conj_lhs.compose(Conj::Yes),
        matrix.as_shape_mut(M, K).as_dyn_stride_mut(),
        true,
        par,
        stack,
    )
}

/// Computes the product of a sequence of block Householder transformations given by
/// `householder_basis` and `householder_factor`, multiplied by `matrix`, and stores the result in
/// `matrix`.
#[track_caller]
pub fn apply_block_householder_sequence_on_the_left_in_place_with_conj<
    T: ComplexField,
    M: Shape,
    N: Shape,
    K: Shape,
    B: Shape,
>(
    householder_basis: MatRef<'_, T, M, N, impl Stride, impl Stride>,
    householder_factor: MatRef<'_, T, B, N, impl Stride, impl Stride>,
    conj_lhs: Conj,
    matrix: MatMut<'_, T, M, K, impl Stride, impl Stride>,
    par: Par,
    stack: &mut DynStack,
) {
    #[track_caller]
    pub fn imp<'M, 'N, 'K, 'B, T: ComplexField>(
        householder_basis: MatRef<'_, T, Dim<'M>, Dim<'N>>,
        householder_factor: MatRef<'_, T, Dim<'B>, Dim<'N>>,
        conj_lhs: Conj,
        matrix: MatMut<'_, T, Dim<'M>, Dim<'K>>,
        par: Par,
        stack: &mut DynStack,
    ) {
        let mut matrix = matrix;
        let mut stack = stack;

        assert!(*householder_factor.nrows() > 0);
        let M = householder_basis.nrows();
        let N = householder_basis.ncols();

        let size = householder_factor.ncols();

        let mut j = size.end();

        let mut blocksize = *size % *householder_factor.nrows();
        if blocksize == 0 {
            blocksize = *householder_factor.nrows();
        }

        while *j > 0 {
            let j_prev = size.idx(*j - blocksize);
            blocksize = *householder_factor.nrows();

            {
                let jn = N.checked_idx_inc(*j);
                let jn_prev = N.checked_idx_inc(*j_prev);
                let jm = M.checked_idx_inc(*j_prev);

                let essentials = householder_basis.submatrix_range((jm, M.end()), (jn_prev, jn));

                let householder = householder_factor
                    .subcols_range((j_prev, j))
                    .subrows(IdxInc::ZERO, *j - *j_prev);

                let matrix = matrix.rb_mut().subrows_range_mut((jm, M.end()));
                make_guard!(M);
                make_guard!(N);
                let M = essentials.nrows().bind(M);
                let N = essentials.ncols().bind(N);

                apply_block_householder_on_the_left_in_place_with_conj(
                    essentials.as_shape(M, N),
                    householder.as_shape(N, N),
                    conj_lhs,
                    matrix.as_row_shape_mut(M),
                    par,
                    stack.rb_mut(),
                );
            }

            j = j_prev.to_incl();
        }
    }
    make_guard!(M);
    make_guard!(N);
    make_guard!(K);
    make_guard!(B);
    let M = householder_basis.nrows().bind(M);
    let N = householder_basis.ncols().bind(N);
    let B = householder_factor.nrows().bind(B);
    let K = matrix.ncols().bind(K);
    imp(
        householder_basis.as_dyn_stride().as_shape(M, N),
        householder_factor.as_dyn_stride().as_shape(B, N),
        conj_lhs,
        matrix.as_dyn_stride_mut().as_shape_mut(M, K),
        par,
        stack,
    )
}

/// Computes the product of the transpose of a sequence block Householder transformations given by
/// `householder_basis` and `householder_factor`, multiplied by `matrix`, and stores the result in
/// `matrix`.
#[track_caller]
pub fn apply_block_householder_sequence_transpose_on_the_left_in_place_with_conj<
    T: ComplexField,
    M: Shape,
    N: Shape,
    K: Shape,
    B: Shape,
>(
    householder_basis: MatRef<'_, T, M, N, impl Stride, impl Stride>,
    householder_factor: MatRef<'_, T, B, N, impl Stride, impl Stride>,
    conj_lhs: Conj,
    matrix: MatMut<'_, T, M, K, impl Stride, impl Stride>,
    par: Par,
    stack: &mut DynStack,
) {
    #[track_caller]
    pub fn imp<'M, 'N, 'K, 'B, T: ComplexField>(
        householder_basis: MatRef<'_, T, Dim<'M>, Dim<'N>>,
        householder_factor: MatRef<'_, T, Dim<'B>, Dim<'N>>,
        conj_lhs: Conj,
        matrix: MatMut<'_, T, Dim<'M>, Dim<'K>>,
        par: Par,
        stack: &mut DynStack,
    ) {
        let mut matrix = matrix;
        let mut stack = stack;

        let blocksize = householder_factor.nrows();

        assert!(blocksize.unbound() > 0);
        let M = householder_basis.nrows();
        let N = householder_basis.ncols();

        let size = householder_factor.ncols();

        let mut J = Dim::start();

        while let Some(j) = size.try_check(*J) {
            let j_next = size.advance(j, *blocksize);

            {
                let jn = N.checked_idx_inc(*j);
                let jn_next = N.checked_idx_inc(*j_next);
                let jm = M.checked_idx_inc(*jn);

                let essentials = householder_basis.submatrix_range((jm, M.end()), (jn, jn_next));
                let householder = householder_factor
                    .subcols_range((j, j_next))
                    .subrows(IdxInc::ZERO, *j_next - *jn);

                let matrix = matrix.rb_mut().subrows_range_mut((jm, M.end()));
                make_guard!(M);
                make_guard!(N);
                let M = essentials.nrows().bind(M);
                let N = essentials.ncols().bind(N);

                apply_block_householder_transpose_on_the_left_in_place_with_conj(
                    essentials.as_shape(M, N),
                    householder.as_shape(N, N),
                    conj_lhs,
                    matrix.as_row_shape_mut(M),
                    par,
                    stack.rb_mut(),
                );
            }

            J = j_next;
        }
    }
    make_guard!(M);
    make_guard!(N);
    make_guard!(K);
    make_guard!(B);
    let M = householder_basis.nrows().bind(M);
    let N = householder_basis.ncols().bind(N);
    let B = householder_factor.nrows().bind(B);
    let K = matrix.ncols().bind(K);
    imp(
        householder_basis.as_dyn_stride().as_shape(M, N),
        householder_factor.as_dyn_stride().as_shape(B, N),
        conj_lhs,
        matrix.as_dyn_stride_mut().as_shape_mut(M, K),
        par,
        stack,
    )
}

/// Computes the product of `matrix`, multiplied by a sequence of block Householder transformations
/// given by `householder_basis` and `householder_factor`, and stores the result in `matrix`.
#[track_caller]
pub fn apply_block_householder_sequence_on_the_right_in_place_with_conj<
    T: ComplexField,
    M: Shape,
    N: Shape,
    K: Shape,
    H: Shape,
>(
    householder_basis: MatRef<'_, T, M, N, impl Stride, impl Stride>,
    householder_factor: MatRef<'_, T, H, N, impl Stride, impl Stride>,
    conj_rhs: Conj,
    matrix: MatMut<'_, T, K, M, impl Stride, impl Stride>,
    par: Par,
    stack: &mut DynStack,
) {
    apply_block_householder_sequence_transpose_on_the_left_in_place_with_conj(
        householder_basis,
        householder_factor,
        conj_rhs,
        matrix.transpose_mut(),
        par,
        stack,
    )
}

/// Computes the product of `matrix`, multiplied by the transpose of a sequence of block Householder
/// transformations given by `householder_basis` and `householder_factor`, and stores the result in
/// `matrix`.
#[track_caller]
pub fn apply_block_householder_sequence_transpose_on_the_right_in_place_with_conj<
    T: ComplexField,
    M: Shape,
    N: Shape,
    K: Shape,
    H: Shape,
>(
    householder_basis: MatRef<'_, T, M, N, impl Stride, impl Stride>,
    householder_factor: MatRef<'_, T, H, N, impl Stride, impl Stride>,
    conj_rhs: Conj,
    matrix: MatMut<'_, T, K, M, impl Stride, impl Stride>,
    par: Par,
    stack: &mut DynStack,
) {
    apply_block_householder_sequence_on_the_left_in_place_with_conj(
        householder_basis,
        householder_factor,
        conj_rhs,
        matrix.transpose_mut(),
        par,
        stack,
    )
}
