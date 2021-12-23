pub struct SparseSet<T> {
    dense: Vec<T>,
}

struct Page {
    sparse: Vec<u32>,
}
