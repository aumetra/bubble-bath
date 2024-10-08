/// Handy macro to construct a hashmap
///
/// Example:
///
/// ```rust
/// # use bubble_bath::hashmap;
/// # use std::collections::HashMap;
/// let _: HashMap<_, HashMap<_, _>> = hashmap! [
///     "key" => hashmap![],
///     "key2" => hashmap![
///         "inner key" => "inner value",
///     ],
/// ];
/// ```
#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $value:expr),*$(,)?) => {{
        let mut hashmap = ::std::collections::HashMap::default();

        $(
            let _ = hashmap.insert($key, $value);
        )*

        hashmap
    }}
}

/// Handy macro to construct a hashset
///
/// Example:
///
/// ```rust
/// # use bubble_bath::hashset;
/// # use std::collections::HashSet;
/// let _ : HashSet<_> = hashset![
///     "key1",
///     "key2",
///     "key3",
/// ];
#[macro_export]
macro_rules! hashset {
    ($($value:expr),*$(,)?) => {{
        let mut hashset = ::std::collections::HashSet::default();

        $(
            let _ = hashset.insert($value);
        )*

        hashset
    }}
}
