//! Rust side of the Standard_MMgrOpt script case (tools/oracle/tkernel_standard_services_memory.cxx).
use crate::common::next;
use rustycad_tkernel::standard::{Failure, MMgrOpt, StorageAddress};
use std::str::SplitWhitespace;
use std::sync::Mutex;

/// One callback observation: (is_alloc, round size, requested size).
type Event = (bool, usize, usize);
static EVENTS: Mutex<Vec<Event>> = Mutex::new(Vec::new());

fn record(is_alloc: bool, _: StorageAddress, round_size: usize, size: usize) {
    EVENTS.lock().unwrap().push((is_alloc, round_size, size));
}

fn events() -> std::sync::MutexGuard<'static, Vec<Event>> {
    EVENTS.lock().unwrap()
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Live,
    Cached,
    Gone,
}

#[derive(Clone, Copy)]
struct Block {
    address: StorageAddress,
    round: usize,
    filled: usize,
    status: State,
    owner: usize,
}

/// Replays one script against a fresh manager, as the native `mmgr::Script` does.
struct Script {
    clear: bool,
    mmap: bool,
    cell: usize,
    max: usize,
    blocks: Vec<Block>,
    pool_base: usize,
    next_small: usize,
}

impl Script {
    fn run(&mut self, manager: &mut MMgrOpt, token: &str) -> String {
        events().clear();
        let result = self.step(manager, token).unwrap_or_else(|failure| {
            // A failed allocation may have purged cached blocks; forget them.
            self.forget_cached();
            format!(
                "E{}:{}",
                failure.exception_type(),
                String::from_utf8_lossy(failure.what())
            )
        });
        format!("{}/{result}", event_text())
    }

    fn step(&mut self, manager: &mut MMgrOpt, token: &str) -> Result<String, Failure> {
        let (operation, arguments) = token.split_at(1);
        match operation {
            "a" => self.added(manager, |m, size| m.allocate(size), arguments),
            "R" => self.added(manager, |m, size| m.reallocate(None, size), arguments),
            "m" => self.many(manager, arguments),
            "f" => self.freed(manager, arguments.parse().expect("index")),
            "F" => self.freed_range(manager, arguments),
            "r" => self.reallocated(manager, arguments),
            "n" => manager.free(None).map(|()| "-".into()),
            _ => {
                let count = manager.purge(false);
                self.forget_cached();
                Ok(format!("P{count}"))
            }
        }
    }

    fn added(
        &mut self,
        manager: &mut MMgrOpt,
        allocate: fn(&mut MMgrOpt, usize) -> Result<StorageAddress, Failure>,
        arguments: &str,
    ) -> Result<String, Failure> {
        let size: usize = arguments.parse().expect("size");
        let address = allocate(manager, size)?;
        let round = events().last().map_or(0, |event| event.1);
        self.relation(manager, address, round, size)
    }

    fn many(&mut self, manager: &mut MMgrOpt, arguments: &str) -> Result<String, Failure> {
        let (count, size) = pair(arguments);
        let mut pools = 0;
        let mut last = String::new();
        for _ in 0..count {
            last = self.added(manager, |m, size| m.allocate(size), &size.to_string())?;
            pools += usize::from(last.starts_with("@new"));
        }
        keep_last_event();
        Ok(format!("pools={pools};{last}"))
    }

    fn freed(&mut self, manager: &mut MMgrOpt, index: usize) -> Result<String, Failure> {
        manager.free(Some(self.blocks[index].address))?;
        self.release(index);
        Ok("-".into())
    }

    fn freed_range(&mut self, manager: &mut MMgrOpt, arguments: &str) -> Result<String, Failure> {
        let (first, count) = pair(arguments);
        for index in first..first + count {
            self.freed(manager, index)?;
        }
        keep_last_event();
        Ok("-".into())
    }

    fn reallocated(&mut self, manager: &mut MMgrOpt, arguments: &str) -> Result<String, Failure> {
        let (index, size) = pair(arguments);
        let old = self.blocks[index];
        let moved = manager.reallocate(Some(old.address), size)?;
        let kept = old.filled.min(size);
        let bytes = &manager.bytes(moved)?[..kept];
        let copied = (0..kept).all(|i| bytes[i] == pattern(old.owner, i));
        let copy = if copied { "c1" } else { "c0" };
        if moved == old.address {
            self.blocks[index].status = State::Gone;
            self.blocks.push(Block {
                status: State::Live,
                ..old
            });
            return Ok(format!("same{copy}"));
        }
        self.release(index);
        let round = events().first().map_or(0, |event| event.1);
        Ok(self.relation(manager, moved, round, size)? + copy)
    }

    /// Reuse of a cached block, offset in the current pool or a fresh system block, then
    /// the address modulo 16 and the zero check; the block is then filled and recorded.
    fn relation(
        &mut self,
        manager: &mut MMgrOpt,
        address: StorageAddress,
        round: usize,
        size: usize,
    ) -> Result<String, Failure> {
        let mut relation = self.origin(address.get(), round);
        relation += &format!("%{}", address.get() % 16);
        let zero = self.fill_and_record(manager, address, round, size.min(round))?;
        if self.clear {
            relation += if zero { "z1" } else { "z0" };
        }
        Ok(relation)
    }

    /// Reports whether the first `fill` bytes were zero, writes the block's pattern over
    /// them and records the block.
    fn fill_and_record(
        &mut self,
        manager: &mut MMgrOpt,
        address: StorageAddress,
        round: usize,
        fill: usize,
    ) -> Result<bool, Failure> {
        let bytes = &mut manager.bytes_mut(address)?[..fill];
        let zero = bytes.iter().all(|byte| *byte == 0);
        let owner = self.blocks.len();
        bytes
            .iter_mut()
            .enumerate()
            .for_each(|(i, byte)| *byte = pattern(owner, i));
        let status = State::Live;
        self.blocks.push(Block {
            address,
            round,
            filled: fill,
            status,
            owner,
        });
        Ok(zero)
    }

    /// "=j" for a cached block, "@..." inside the small-block pool, "new" otherwise.
    fn origin(&mut self, address: usize, round: usize) -> String {
        let reused = self.reuse(address);
        if !reused.is_empty() {
            return reused;
        }
        if round <= self.cell && (round >> 3) <= self.max {
            return self.pool_offset(address, round);
        }
        "new".into()
    }

    fn reuse(&mut self, address: usize) -> String {
        let found = self
            .blocks
            .iter()
            .rposition(|block| block.status == State::Cached && block.address.get() == address);
        let Some(index) = found else {
            return String::new();
        };
        self.blocks[index].status = State::Gone;
        format!("={index}")
    }

    fn pool_offset(&mut self, address: usize, round: usize) -> String {
        let relation = if self.pool_base == 0 || address != self.next_small {
            self.pool_base = address;
            // A mapped pool starts on a page; a malloc'ed pool's page offset is not reproducible.
            let offset = if self.mmap {
                (address % 4096).to_string()
            } else {
                String::new()
            };
            format!("@new{offset}")
        } else {
            format!("@{}", address - self.pool_base)
        };
        self.next_small = address + round + size_of::<usize>();
        relation
    }

    fn release(&mut self, index: usize) {
        let large = (self.blocks[index].round >> 3) > self.max;
        self.blocks[index].status = if large { State::Gone } else { State::Cached };
    }

    fn forget_cached(&mut self) {
        for block in &mut self.blocks {
            if block.status == State::Cached {
                block.status = State::Gone;
            }
        }
    }
}

fn pattern(index: usize, byte: usize) -> u8 {
    ((index * 31 + byte + 1) & 0xff) as u8
}

fn pair(arguments: &str) -> (usize, usize) {
    let (first, second) = arguments.split_once(',').expect("pair");
    (
        first.parse().expect("first"),
        second.parse().expect("second"),
    )
}

fn keep_last_event() {
    let mut events = events();
    let keep = events.len().saturating_sub(1);
    events.drain(..keep);
}

fn event_text() -> String {
    let text: Vec<String> = events()
        .iter()
        .map(|&(is_alloc, round, size)| {
            if is_alloc {
                format!("A{round}:{size}")
            } else {
                format!("F{round}")
            }
        })
        .collect();
    if text.is_empty() {
        "-".into()
    } else {
        text.join(",")
    }
}

/// "services_mmgr <clear> <mmap> <cell> <pages> <threshold> <token>...": one result per token.
pub fn script_case(input: &mut SplitWhitespace<'_>) -> String {
    let clear = next::<i32>(input) != 0;
    let mmap = next::<i32>(input) != 0;
    let cell: usize = next(input);
    let pages: i32 = next(input);
    let threshold: usize = next(input);
    MMgrOpt::set_call_back_function(Some(record));
    let mut manager = MMgrOpt::new(clear, mmap, cell, pages, threshold);
    let mut script = Script {
        clear,
        mmap,
        cell: cell.wrapping_add(15) & !15,
        max: (threshold.wrapping_sub(1).wrapping_add(7) & !7) >> 3,
        blocks: Vec::new(),
        pool_base: 0,
        next_small: 0,
    };
    let results: Vec<String> = input.map(|token| script.run(&mut manager, token)).collect();
    MMgrOpt::set_call_back_function(None);
    format!("OK {}", results.join(" "))
}
