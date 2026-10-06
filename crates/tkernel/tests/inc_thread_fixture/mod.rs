//! Pointer observations normalize native block identity without dereferencing raw storage.
use rustycad_tkernel::ncollection::BaseAllocator;
use std::fmt::Write;

pub struct Trace {
    pub arena: BaseAllocator,
    blocks: Vec<(usize, usize)>,
    block_size: usize,
    row: String,
}

impl Trace {
    pub fn new(name: &str, enabled: bool) -> Self {
        let arena = BaseAllocator::incremental(2048);
        arena.set_thread_safe(enabled).unwrap();
        Self {
            arena,
            blocks: Vec::new(),
            block_size: 2048,
            row: name.into(),
        }
    }

    pub fn allocate(&mut self, size: usize, optimal: bool) {
        let pointer = if optimal {
            self.arena.allocate_optimal_raw(size)
        } else {
            self.arena.allocate_raw(size)
        }
        .unwrap();
        let address = pointer.as_ptr().addr();
        let (block, offset) = self.locate(address);
        write!(self.row, " {block}:{offset}").unwrap();
    }

    fn locate(&mut self, address: usize) -> (usize, usize) {
        for (index, &(start, length)) in self.blocks.iter().enumerate() {
            if (start..=start + length).contains(&address) {
                return (index, address - start);
            }
        }
        if (self.blocks.len() + 1).is_multiple_of(5) {
            self.block_size *= 8;
        }
        self.blocks.push((address, self.block_size));
        (self.blocks.len() - 1, 0)
    }

    pub fn reset(&mut self, release: bool) {
        self.arena.reset(release).unwrap();
        if release {
            self.blocks.clear();
            self.block_size = 12288;
        }
        self.row
            .push_str(if release { " release" } else { " rewind" });
    }

    pub fn assert_native(self) {
        let expected =
            include_str!("../../../../tools/oracle/tkernel_inc_allocator_thread_policy.expected");
        let label = self.row.split_whitespace().next().unwrap();
        let native = expected
            .lines()
            .find(|line| line.split_whitespace().next() == Some(label));
        assert_eq!(Some(self.row.as_str()), native, "native row {label}");
    }
}
