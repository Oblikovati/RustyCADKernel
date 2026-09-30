//! Deterministic median-of-three partitioning for the Linux/GCC oracle profile.
//! Equal-coordinate permutation is observable in KDTree tie indices and result order.
pub(super) fn select(indices: &mut [usize], nth: usize, less: impl Fn(usize, usize) -> bool) {
    if indices.len() < 2 {
        return;
    }
    let mut lo = 0;
    let mut hi = indices.len();
    let mut budget = 2 * indices.len().ilog2();
    while hi - lo > 3 {
        if budget == 0 {
            heap_select(indices, lo, hi, nth, &less);
            return;
        }
        budget -= 1;
        let pivot = median(indices, lo + 1, lo + (hi - lo) / 2, hi - 1, &less);
        indices.swap(lo, pivot);
        let cut = partition(indices, lo, hi, &less);
        if cut <= nth {
            lo = cut;
        } else {
            hi = cut;
        }
    }
    insertion(&mut indices[lo..hi], &less);
}
fn median(
    a: &[usize],
    x: usize,
    y: usize,
    z: usize,
    less: &impl Fn(usize, usize) -> bool,
) -> usize {
    if less(a[x], a[y]) {
        if less(a[y], a[z]) {
            y
        } else if less(a[x], a[z]) {
            z
        } else {
            x
        }
    } else if less(a[x], a[z]) {
        x
    } else if less(a[y], a[z]) {
        z
    } else {
        y
    }
}
fn partition(a: &mut [usize], lo: usize, hi: usize, less: &impl Fn(usize, usize) -> bool) -> usize {
    let pivot = a[lo];
    let mut left = lo + 1;
    let mut right = hi;
    loop {
        while less(a[left], pivot) {
            left += 1;
        }
        right -= 1;
        while less(pivot, a[right]) {
            right -= 1;
        }
        if left >= right {
            return left;
        }
        a.swap(left, right);
        left += 1;
    }
}
fn insertion(a: &mut [usize], less: &impl Fn(usize, usize) -> bool) {
    for index in 1..a.len() {
        let value = a[index];
        let mut position = index;
        while position > 0 && less(value, a[position - 1]) {
            a[position] = a[position - 1];
            position -= 1;
        }
        a[position] = value;
    }
}
fn heap_select(
    a: &mut [usize],
    lo: usize,
    hi: usize,
    nth: usize,
    less: &impl Fn(usize, usize) -> bool,
) {
    let count = nth - lo + 1;
    for parent in (0..count / 2).rev() {
        let value = a[lo + parent];
        adjust(&mut a[lo..lo + count], parent, value, less);
    }
    for index in nth + 1..hi {
        if less(a[index], a[lo]) {
            let value = a[index];
            a[index] = a[lo];
            adjust(&mut a[lo..lo + count], 0, value, less);
        }
    }
    a.swap(lo, nth);
}
fn adjust(a: &mut [usize], start: usize, value: usize, less: &impl Fn(usize, usize) -> bool) {
    let mut hole = start;
    let mut right = 2 * (hole + 1);
    while right < a.len() {
        let child = if less(a[right], a[right - 1]) {
            right - 1
        } else {
            right
        };
        a[hole] = a[child];
        hole = child;
        right = 2 * (hole + 1);
    }
    if right == a.len() {
        a[hole] = a[right - 1];
        hole = right - 1;
    }
    while hole > start && less(a[(hole - 1) / 2], value) {
        a[hole] = a[(hole - 1) / 2];
        hole = (hole - 1) / 2;
    }
    a[hole] = value;
}
