pub fn same_discriminant<V>(a: &V, b: &V) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
