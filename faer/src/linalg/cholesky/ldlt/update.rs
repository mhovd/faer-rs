use crate::internal_prelude::*;
use pulp::Simd;

#[math]
fn rank_update_step_simd<T: ComplexField>(
	L: ColMut<'_, T, usize, ContiguousFwd>,
	W: MatMut<'_, T, usize, usize, ContiguousFwd>,
	p: ColRef<'_, T>,
	beta: ColRef<'_, T>,
	align_offset: usize,
) {
	struct Impl<'a, 'N, 'R, T: ComplexField> {
		L: ColMut<'a, T, Dim<'N>, ContiguousFwd>,
		W: MatMut<'a, T, Dim<'N>, Dim<'R>, ContiguousFwd>,
		p: ColRef<'a, T, Dim<'R>>,
		beta: ColRef<'a, T, Dim<'R>>,
		align_offset: usize,
	}

	impl<'a, 'N, 'R, T: ComplexField> pulp::WithSimd for Impl<'a, 'N, 'R, T> {
		type Output = ();

		#[inline(always)]
		fn with_simd<S: Simd>(self, simd: S) {
			let Self { L, W, p, beta, align_offset } = self;

			let mut L = L;
			let mut W = W;
			let N = W.nrows();
			let R = W.ncols();

			let simd = SimdCtx::<T, S>::new_align(T::simd_ctx(simd), N, align_offset);
			let (head, body, tail) = simd.indices();

			let mut iter = R.indices();
			let (i0, i1, i2, i3) = (iter.next(), iter.next(), iter.next(), iter.next());

			match (i0, i1, i2, i3) {
				(Some(i0), None, None, None) => {
					let p0 = simd.splat(&p[i0]);
					let beta0 = simd.splat(&beta[i0]);

					macro_rules! simd {
						($i: expr) => {{
							let i = $i;
							let mut l = simd.read(L.rb(), i);
							let mut w0 = simd.read(W.rb().col(i0), i);

							w0 = simd.mul_add(p0, l, w0);
							l = simd.mul_add(beta0, w0, l);

							simd.write(L.rb_mut(), i, l);
							simd.write(W.rb_mut().col_mut(i0), i, w0);
						}};
					}

					if let Some(i) = head {
						simd!(i);
					}
					for i in body {
						simd!(i);
					}
					if let Some(i) = tail {
						simd!(i);
					}
				},
				(Some(i0), Some(i1), None, None) => {
					let (p0, p1) = (simd.splat(&p[i0]), simd.splat(&p[i1]));
					let (beta0, beta1) = (simd.splat(&beta[i0]), simd.splat(&beta[i1]));

					macro_rules! simd {
						($i: expr) => {{
							let i = $i;
							let mut l = simd.read(L.rb(), i);
							let mut w0 = simd.read(W.rb().col(i0), i);
							let mut w1 = simd.read(W.rb().col(i1), i);

							w0 = simd.mul_add(p0, l, w0);
							l = simd.mul_add(beta0, w0, l);
							w1 = simd.mul_add(p1, l, w1);
							l = simd.mul_add(beta1, w1, l);

							simd.write(L.rb_mut(), i, l);
							simd.write(W.rb_mut().col_mut(i0), i, w0);
							simd.write(W.rb_mut().col_mut(i1), i, w1);
						}};
					}

					if let Some(i) = head {
						simd!(i);
					}
					for i in body {
						simd!(i);
					}
					if let Some(i) = tail {
						simd!(i);
					}
				},
				(Some(i0), Some(i1), Some(i2), None) => {
					let (p0, p1, p2) = (simd.splat(&p[i0]), simd.splat(&p[i1]), simd.splat(&p[i2]));
					let (beta0, beta1, beta2) = (simd.splat(&beta[i0]), simd.splat(&beta[i1]), simd.splat(&beta[i2]));

					macro_rules! simd {
						($i: expr) => {{
							let i = $i;
							let mut l = simd.read(L.rb(), i);
							let mut w0 = simd.read(W.rb().col(i0), i);
							let mut w1 = simd.read(W.rb().col(i1), i);
							let mut w2 = simd.read(W.rb().col(i2), i);

							w0 = simd.mul_add(p0, l, w0);
							l = simd.mul_add(beta0, w0, l);
							w1 = simd.mul_add(p1, l, w1);
							l = simd.mul_add(beta1, w1, l);
							w2 = simd.mul_add(p2, l, w2);
							l = simd.mul_add(beta2, w2, l);

							simd.write(L.rb_mut(), i, l);
							simd.write(W.rb_mut().col_mut(i0), i, w0);
							simd.write(W.rb_mut().col_mut(i1), i, w1);
							simd.write(W.rb_mut().col_mut(i2), i, w2);
						}};
					}

					if let Some(i) = head {
						simd!(i);
					}
					for i in body {
						simd!(i);
					}
					if let Some(i) = tail {
						simd!(i);
					}
				},
				(Some(i0), Some(i1), Some(i2), Some(i3)) => {
					let (p0, p1, p2, p3) = (simd.splat(&p[i0]), simd.splat(&p[i1]), simd.splat(&p[i2]), simd.splat(&p[i3]));
					let (beta0, beta1, beta2, beta3) = (simd.splat(&beta[i0]), simd.splat(&beta[i1]), simd.splat(&beta[i2]), simd.splat(&beta[i3]));

					macro_rules! simd {
						($i: expr) => {{
							let i = $i;
							let mut l = simd.read(L.rb(), i);
							let mut w0 = simd.read(W.rb().col(i0), i);
							let mut w1 = simd.read(W.rb().col(i1), i);
							let mut w2 = simd.read(W.rb().col(i2), i);
							let mut w3 = simd.read(W.rb().col(i3), i);

							w0 = simd.mul_add(p0, l, w0);
							l = simd.mul_add(beta0, w0, l);
							w1 = simd.mul_add(p1, l, w1);
							l = simd.mul_add(beta1, w1, l);
							w2 = simd.mul_add(p2, l, w2);
							l = simd.mul_add(beta2, w2, l);
							w3 = simd.mul_add(p3, l, w3);
							l = simd.mul_add(beta3, w3, l);

							simd.write(L.rb_mut(), i, l);
							simd.write(W.rb_mut().col_mut(i0), i, w0);
							simd.write(W.rb_mut().col_mut(i1), i, w1);
							simd.write(W.rb_mut().col_mut(i2), i, w2);
							simd.write(W.rb_mut().col_mut(i3), i, w3);
						}};
					}

					if let Some(i) = head {
						simd!(i);
					}
					for i in body {
						simd!(i);
					}
					if let Some(i) = tail {
						simd!(i);
					}
				},
				_ => panic!(),
			}
		}
	}

	with_dim!(N, W.nrows());
	with_dim!(R, W.ncols());

	dispatch!(
		Impl {
			L: L.as_row_shape_mut(N),
			W: W.as_shape_mut(N, R),
			p: p.as_row_shape(R),
			beta: beta.as_row_shape(R),
			align_offset,
		},
		Impl,
		T
	)
}

#[math]
fn rank_update_step_fallback<T: ComplexField>(L: ColMut<'_, T>, W: MatMut<'_, T>, p: ColRef<'_, T>, beta: ColRef<'_, T>) {
	let mut L = L;
	let mut W = W;
	let n = W.nrows();
	let r = W.ncols();

	let mut iter = 0..r;
	let (i0, i1, i2, i3) = (iter.next(), iter.next(), iter.next(), iter.next());

	match (i0, i1, i2, i3) {
		(Some(i0), None, None, None) => {
			let p0 = &p[i0];
			let beta0 = &beta[i0];

			for i in 0..n {
				let mut l = copy(L[i]);
				let mut w0 = copy(W[(i, i0)]);

				w0 = *p0 * l + w0;
				l = *beta0 * w0 + l;

				L[i] = l;
				W[(i, i0)] = w0;
			}
		},
		(Some(i0), Some(i1), None, None) => {
			let (p0, p1) = (&p[i0], &p[i1]);
			let (beta0, beta1) = (&beta[i0], &beta[i1]);

			for i in 0..n {
				let mut l = copy(L[i]);
				let mut w0 = copy(W[(i, i0)]);
				let mut w1 = copy(W[(i, i1)]);

				w0 = *p0 * l + w0;
				l = *beta0 * w0 + l;
				w1 = *p1 * l + w1;
				l = *beta1 * w1 + l;

				L[i] = l;
				W[(i, i0)] = w0;
				W[(i, i1)] = w1;
			}
		},
		(Some(i0), Some(i1), Some(i2), None) => {
			let (p0, p1, p2) = (&p[i0], &p[i1], &p[i2]);
			let (beta0, beta1, beta2) = (&beta[i0], &beta[i1], &beta[i2]);

			for i in 0..n {
				let mut l = copy(L[i]);
				let mut w0 = copy(W[(i, i0)]);
				let mut w1 = copy(W[(i, i1)]);
				let mut w2 = copy(W[(i, i2)]);

				w0 = *p0 * l + w0;
				l = *beta0 * w0 + l;
				w1 = *p1 * l + w1;
				l = *beta1 * w1 + l;
				w2 = *p2 * l + w2;
				l = *beta2 * w2 + l;

				L[i] = l;
				W[(i, i0)] = w0;
				W[(i, i1)] = w1;
				W[(i, i2)] = w2;
			}
		},
		(Some(i0), Some(i1), Some(i2), Some(i3)) => {
			let (p0, p1, p2, p3) = (&p[i0], &p[i1], &p[i2], &p[i3]);
			let (beta0, beta1, beta2, beta3) = (&beta[i0], &beta[i1], &beta[i2], &beta[i3]);

			for i in 0..n {
				let mut l = copy(L[i]);
				let mut w0 = copy(W[(i, i0)]);
				let mut w1 = copy(W[(i, i1)]);
				let mut w2 = copy(W[(i, i2)]);
				let mut w3 = copy(W[(i, i3)]);

				w0 = *p0 * l + w0;
				l = *beta0 * w0 + l;
				w1 = *p1 * l + w1;
				l = *beta1 * w1 + l;
				w2 = *p2 * l + w2;
				l = *beta2 * w2 + l;
				w3 = *p3 * l + w3;
				l = *beta3 * w3 + l;

				L[i] = l;
				W[(i, i0)] = w0;
				W[(i, i1)] = w1;
				W[(i, i2)] = w2;
				W[(i, i3)] = w3;
			}
		},
		_ => panic!(),
	}
}

struct RankRUpdate<'a, T: ComplexField> {
	ld: MatMut<'a, T>,
	w: MatMut<'a, T>,
	alpha: ColMut<'a, T>,
	r: &'a mut dyn FnMut() -> usize,
}

impl<T: ComplexField> RankRUpdate<'_, T> {
	// On the Modification of LDLT Factorizations
	// By R. Fletcher and M. J. D. Powell
	// https://www.ams.org/journals/mcom/1974-28-128/S0025-5718-1974-0359297-1/S0025-5718-1974-0359297-1.pdf

	#[math]
	fn run(self) {
		let Self { mut ld, mut w, mut alpha, r } = self;

		let n = w.nrows();
		let k = w.ncols();

		for j in 0..n {
			let mut L_col = ld.rb_mut().col_mut(j);

			let r = Ord::min(r(), k);
			let mut W = w.rb_mut().subcols_mut(0, r);
			let mut alpha = alpha.rb_mut().subrows_mut(0, r);
			let R = r;

			const BLOCKSIZE: usize = 4;

			let mut r = 0;
			while r < R {
				let bs = Ord::min(BLOCKSIZE, R - r);

				stack_mat!(p, bs, 1, BLOCKSIZE, 1, T);
				stack_mat!(beta, bs, 1, BLOCKSIZE, 1, T);

				let mut p = p.rb_mut().col_mut(0);
				let mut beta = beta.rb_mut().col_mut(0);

				for k in 0..bs {
					let p = p.rb_mut().at_mut(k);
					let beta = beta.rb_mut().at_mut(k);
					let alpha = alpha.rb_mut().at_mut(r + k);
					let d = L_col.rb_mut().at_mut(j);

					let w = W.rb().col(r + k);

					*p = copy(w[j]);

					let alpha_conj_p = *alpha * conj(*p);
					let new_d = real(*d) + real(mul(alpha_conj_p, *p));
					*beta = mul_real(alpha_conj_p, recip(new_d));

					*alpha = from_real(real(*alpha) - new_d * abs2(*beta));
					*d = from_real(new_d);
					*p = -*p;
				}

				let mut L_col = L_col.rb_mut().get_mut(j + 1..);
				let mut W_col = W.rb_mut().subcols_mut(r, bs).get_mut(j + 1.., ..);

				if const { T::SIMD_CAPABILITIES.is_simd() } {
					if let (Some(L_col), Some(W_col)) = (L_col.rb_mut().try_as_col_major_mut(), W_col.rb_mut().try_as_col_major_mut()) {
						rank_update_step_simd(L_col, W_col, p.rb(), beta.rb(), simd_align(j + 1));
					} else {
						rank_update_step_fallback(L_col, W_col, p.rb(), beta.rb());
					}
				} else {
					rank_update_step_fallback(L_col, W_col, p.rb(), beta.rb());
				}
				r += bs;
			}
		}
	}
}

#[track_caller]
pub fn rank_r_update_clobber<T: ComplexField>(cholesky_factors: MatMut<'_, T>, w: MatMut<'_, T>, alpha: DiagMut<'_, T>) {
	let n = cholesky_factors.nrows();
	let r = w.ncols();

	if n == 0 {
		return;
	}

	RankRUpdate {
		ld: cholesky_factors,
		w,
		alpha: alpha.column_vector_mut(),
		r: &mut || r,
	}
	.run();
}

#[cfg(test)]
mod tests {
	use dyn_stack::MemBuffer;

	use super::*;
	use crate::stats::prelude::*;
	use crate::utils::approx::*;
	use crate::{Col, Mat, assert, c64};

	#[test]
	#[azucar::infer]
	fn test_rank_update() {
		let rng = &mut StdRng::seed_from_u64(0);

		let approx_eq = CwiseMat(ApproxEq {
			abs_tol: 1e-12,
			rel_tol: 1e-12,
		});

		for r in [0, 1, 2, 3, 4, 5, 6, 7, 8, 10] {
			for n in [2, 4, 8, 15] {
				let A = CwiseMatDistribution {
					nrows: n,
					ncols: n,
					dist: ComplexDistribution::new(StandardNormal, StandardNormal),
				}
				.rand::<Mat<c64>>(rng);
				let mut W = CwiseMatDistribution {
					nrows: n,
					ncols: r,
					dist: ComplexDistribution::new(StandardNormal, StandardNormal),
				}
				.rand::<Mat<c64>>(rng);
				let mut alpha = CwiseColDistribution {
					nrows: r,
					dist: ComplexDistribution::new(StandardNormal, StandardNormal),
				}
				.rand::<Col<c64>>(rng)
				.into_diagonal();

				for j in 0..r {
					alpha.column_vector_mut()[j].im = 0.0;
				}

				let A = &A * &A.adjoint();
				let A_new = &A + &W * &alpha * &W.adjoint();

				let A = A.as_ref();
				let A_new = A_new.as_ref();

				let mut L = A.cloned();
				let mut L = L.as_mut();

				linalg::cholesky::ldlt::factor::cholesky_in_place(
					L.rb_mut(),
					default(),
					Par::Seq,
					MemStack::new(&mut MemBuffer::new(linalg::cholesky::ldlt::factor::cholesky_in_place_scratch::<c64>(
						n,
						Par::Seq,
						_,
					))),
					_,
				)
				.unwrap();

				linalg::cholesky::ldlt::update::rank_r_update_clobber(L.rb_mut(), W.as_mut(), alpha.as_mut());
				let D = L.as_mut().diagonal().column_vector().as_mat().cloned();
				let D = D.col(0).as_diagonal();

				for j in 0..n {
					for i in 0..j {
						L[(i, j)] = c64::ZERO;
					}
					L[(j, j)] = c64::ONE;
				}
				let L = L.as_ref();

				assert!(A_new ~ L * D * L.adjoint());
			}
		}
	}
}
