/// Creates an uninitialized array on the stack, and then moves the contents
/// of the input collection into this array, until either the array is full,
/// or the collection ends.
///
/// If the size of the collection is less than the size of the array, then
/// part of the array remains uninitialized (even if the return type states the opposite).
///
/// Dropping uninitialized structures will cause undefined behavior, if they contain references
/// as fields.
pub(crate) unsafe fn partially_init_array<T, U, const N: usize>(
    into: impl Fn(T) -> U,
    items: impl IntoIterator<Item = T>,
) -> [U; N] {
    let mut arr: [U; N] = std::mem::MaybeUninit::uninit().assume_init();
    items
        .into_iter()
        .take(N)
        .map(into)
        .enumerate()
        .for_each(|(i, u)| arr[i] = u);
    arr
}
