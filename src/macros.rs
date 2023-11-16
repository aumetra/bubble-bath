/// Handy macro to construct a hashmap using the [`ahash`] hasher
///
/// Example:
///
/// ```rust
/// # use bubble_bath::hashmap;
/// hashmap! [
///     "key" => hashmap![],
///     "key2" => hashmap![
///         "inner key" => "inner value",
///     ],
/// ];
/// ```
#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $value:expr),*$(,)?) => {{
        let mut hashmap = $crate::ahash::AHashMap::new();

        $(
            let _ = hashmap.insert($key, $value);
        )*

        hashmap
    }}
}

/// Handy macro to construct a hashset using the [`ahash`] hasher
///
/// Example:
///
/// ```rust
/// # use bubble_bath::hashset;
/// hashset![
///     "key1",
///     "key2",
///     "key3",
/// ];
#[macro_export]
macro_rules! hashset {
    ($($value:expr),*$(,)?) => {{
        let mut hashset = $crate::ahash::AHashSet::new();

        $(
            let _ = hashset.insert($value);
        )*

        hashset
    }}
}
