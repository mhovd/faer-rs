/// Beginners guide to `faer`
///
///
/// # Creating a matrix
///
/// The basic type of matrix in `faer` is the [crate::Mat].
///
/// ### `mat!` macro
///
/// ```rust
/// use faer::mat;
///
/// let a = mat![
///	[1.0, 5.0, 9.0],
///	[2.0, 6.0, 10.0],
///	[3.0, 7.0, 11.0],
///	[4.0, 8.0, 12.0f64]];
/// ```
///
///