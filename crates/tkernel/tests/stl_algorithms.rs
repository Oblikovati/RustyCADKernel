//! libstdc++ `std::sort`/`std::partition` element-order parity.
//!
//! Expected values were emitted by GCC 13.3/libstdc++ (x86-64 Linux) from the
//! same inputs: tie orders, a McIlroy "antiqsort" adversary that drives
//! introsort into its heap-sort fallback (comparison count and FNV-1a hash of
//! every comparator argument), and bidirectional partitions.
use rustycad_tkernel::stl::{partition, sort_by};

fn tie_order(count: usize) -> Vec<usize> {
    let keys: Vec<usize> = (0..count).map(|index| (index * 7 + 3) % 5).collect();
    let mut indices: Vec<usize> = (0..count).collect();
    sort_by(&mut indices, |first, second| keys[*first] < keys[*second]);
    indices
}

#[test]
fn sort_by_keeps_libstdcxx_order_of_ties() {
    assert_eq!(tie_order(5), [1, 4, 2, 0, 3]);
    assert_eq!(
        tie_order(16),
        [1, 6, 11, 4, 9, 14, 2, 7, 12, 0, 5, 10, 15, 3, 8, 13]
    );
    assert_eq!(
        tie_order(17),
        [1, 16, 11, 6, 4, 9, 14, 7, 2, 12, 5, 10, 15, 0, 3, 8, 13]
    );
    let forty = [
        1, 36, 31, 6, 26, 21, 11, 16, 39, 34, 29, 24, 19, 14, 9, 4, 37, 12, 17, 2, 32, 22, 7, 27,
        0, 35, 30, 5, 25, 20, 10, 15, 28, 8, 23, 33, 3, 18, 38, 13,
    ];
    assert_eq!(tie_order(40), forty);
    let hundred_prefix = [
        1, 36, 31, 41, 46, 26, 51, 56, 21, 61, 66, 16, 71, 76, 11, 96, 81, 86,
    ];
    assert_eq!(tie_order(100)[..18], hundred_prefix);
    let long_prefix = [
        96, 146, 141, 136, 131, 126, 121, 116, 111, 106, 101, 151, 91, 86, 81,
    ];
    assert_eq!(tie_order(257)[..15], long_prefix);
    assert_eq!(
        tie_order(257)[241..],
        [
            138, 33, 133, 203, 128, 123, 28, 208, 118, 113, 213, 218, 103, 8, 98, 223
        ]
    );
}

/// McIlroy's adversary: values are frozen lazily so every partition is lopsided.
struct Adversary {
    values: Vec<usize>,
    gas: usize,
    solid: usize,
    candidate: usize,
    hash: u64,
    count: usize,
}
impl Adversary {
    fn new(count: usize) -> Self {
        Self {
            values: vec![count; count],
            gas: count,
            solid: 0,
            candidate: 0,
            hash: 1_469_598_103_934_665_603,
            count: 0,
        }
    }
    fn mix(&mut self, value: usize) {
        self.hash = (self.hash ^ value as u64).wrapping_mul(1_099_511_628_211);
    }
    fn freeze(&mut self, index: usize) {
        self.values[index] = self.solid;
        self.solid += 1;
    }
    fn less(&mut self, first: usize, second: usize) -> bool {
        self.count += 1;
        self.mix(first);
        self.mix(second);
        if self.values[first] == self.gas && self.values[second] == self.gas {
            self.freeze(if first == self.candidate {
                first
            } else {
                second
            });
        }
        if self.values[first] == self.gas {
            self.candidate = first;
        } else if self.values[second] == self.gas {
            self.candidate = second;
        }
        self.values[first] < self.values[second]
    }
}

fn adversarial_sort(count: usize) -> (usize, u64, Vec<usize>) {
    let mut adversary = Adversary::new(count);
    let mut indices: Vec<usize> = (0..count).collect();
    sort_by(&mut indices, |first, second| {
        adversary.less(*first, *second)
    });
    (adversary.count, adversary.hash, indices[..8].to_vec())
}

#[test]
fn sort_by_matches_libstdcxx_comparisons_under_adversary() {
    assert_eq!(
        adversarial_sort(64),
        (
            988,
            17_228_887_281_560_171_699,
            vec![32, 1, 33, 3, 34, 5, 35, 7]
        )
    );
    // 300 elements exhaust the 2*log2(n) depth budget, so the heap fallback runs.
    assert_eq!(
        adversarial_sort(300),
        (
            7282,
            1_174_775_100_165_771_425,
            vec![150, 1, 151, 3, 152, 5, 153, 7]
        )
    );
}

fn partitioned(count: usize) -> (usize, Vec<usize>) {
    let mut values: Vec<usize> = (0..count).map(|index| (index * 5 + 1) % 9).collect();
    let split = partition(&mut values, |value| value % 2 == 0);
    (split, values)
}

#[test]
fn partition_uses_libstdcxx_two_ended_swaps() {
    assert_eq!(partitioned(0), (0, vec![]));
    assert_eq!(partitioned(1), (0, vec![1]));
    assert_eq!(partitioned(7), (4, vec![4, 6, 2, 8, 3, 7, 1]));
    assert_eq!(
        partitioned(20),
        (
            11,
            vec![6, 6, 2, 0, 4, 8, 4, 0, 8, 2, 6, 1, 7, 3, 5, 3, 7, 5, 1, 1]
        )
    );
}
