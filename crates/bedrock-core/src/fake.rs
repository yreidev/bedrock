//! 假的端口：时间、编号固定，备忘存在内存里。给测试和演示模式用。

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

use jiff::Timestamp;

use crate::note::MemoryNotes;
use crate::{Clock, Core, Ids, Ports};

/// 从 2026-01-01T00:00:00Z 开始，每问一次往后走一秒。
#[derive(Debug, Default)]
pub struct StepClock {
    calls: AtomicI64,
}

impl Clock for StepClock {
    fn now(&self) -> Timestamp {
        let step = self.calls.fetch_add(1, Ordering::Relaxed) + 1;
        Timestamp::constant(1_767_225_600 + step, 0)
    }
}

/// 依次给出 `id-1`、`id-2`……
#[derive(Debug, Default)]
pub struct SeqIds {
    next: AtomicU64,
}

impl Ids for SeqIds {
    fn new_id(&self) -> String {
        format!("id-{}", self.next.fetch_add(1, Ordering::Relaxed) + 1)
    }
}

#[derive(Debug, Default)]
pub struct FakePorts {
    pub clock: StepClock,
    pub ids: SeqIds,
    pub notes: MemoryNotes,
}

impl Ports for FakePorts {
    type Clock = StepClock;
    type Ids = SeqIds;
    type Notes = MemoryNotes;

    fn clock(&self) -> &StepClock {
        &self.clock
    }

    fn ids(&self) -> &SeqIds {
        &self.ids
    }

    fn notes(&self) -> &MemoryNotes {
        &self.notes
    }
}

/// 一个全用假端口的核心。
pub fn core() -> Core<FakePorts> {
    Core::new(FakePorts::default())
}
