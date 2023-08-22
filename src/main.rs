#![feature(const_size_of_val)]
#![feature(buf_read_has_data_left)]

mod parse;

use crate::parse::*;

struct PrintEvents {
    current_block_remaining: i64,
    current_block_capno: Option<u16>,
}

impl PrintEvents {
    fn new() -> PrintEvents {
        PrintEvents {
            current_block_remaining: 0,
            current_block_capno: None,
        }
    }

    fn start(&self, _time: u64) {
        print!("[{_time}]{} ", match self.current_block_capno {
            Some(x) if self.current_block_remaining > 0 =>
                format!("[{}]", x),
            _ => String::new(),
        });
    }
}

impl EventlogParser for PrintEvents {
    fn event_start(&mut self, _id: u16, _time: u64, _size: usize) {
        self.current_block_remaining -= (2 + 8 + _size) as i64;
    }
    fn event_unknown(&mut self, id: u16, _time: u64, bytes: Vec<u8>) {
        self.start(_time);
        println!("[unknown {id}] {} bytes", bytes.len());
    }

    fn event_block_marker(&mut self, _time: u64, _block_size: u32, _time_end: u64, _capno: u16) {
        self.start(_time);

        let _capno = if _capno == !0 {
            None
        } else {
            Some(_capno)
        };

        println!("block start c:{_capno:?} {_block_size} {_time_end}");
        self.current_block_capno = _capno;
        self.current_block_remaining = _block_size as i64;
    }

    fn event_capset_create(&mut self, _time: u64, _capset: u32, _type_: u16) {
        self.start(_time);
        println!("capset create {_capset} {_type_}");
    }
    fn event_capset_delete(&mut self, _time: u64, _capset: u32) {
        self.start(_time);
        println!("capset delete {_capset}");
    }
    fn event_capset_assign_cap(&mut self, _time: u64, _capset: u32, _capno: u16) {
        self.start(_time);
        println!("capset assign {_capset} c:{_capno}");
    }
    fn event_capset_remove_cap(&mut self, _time: u64, _capset: u32, _capno: u16) {
        self.start(_time);
        println!("capset remove {_capset} c:{_capno}");
    }

    fn event_cap_create(&mut self, _time: u64, _capno: u16) {
        self.start(_time);
        println!("cap create c:{_capno}");
    }
    fn event_cap_delete(&mut self, _time: u64, _capno: u16) {
        self.start(_time);
        println!("cap delete c:{_capno}");
    }
    fn event_cap_disable(&mut self, _time: u64, _capno: u16) {
        self.start(_time);
        println!("cap disable c:{_capno}");
    }

    fn event_task_create(&mut self, _time: u64, _taskid: u64, _capno: u16, _k_threadid: u64) {
        self.start(_time);
        println!("task create ta:{_taskid} c:{_capno} {_k_threadid}");
    }
    fn event_task_migrate(&mut self, _time: u64, _taskid: u64, _from_capno: u16, _to_capno: u16) {
        self.start(_time);
        println!("task migrate ta:{_taskid} c:{_from_capno} c:{_to_capno}");
    }
    fn event_task_delete(&mut self, _time: u64, _taskid: u64) {
        self.start(_time);
        println!("task delete ta:{_taskid}");
    }

    fn event_thread_create(&mut self, _time: u64, _threadid: u32) {
        self.start(_time);
        println!("thread create ti:{_threadid}");
    }
    fn event_thread_run(&mut self, _time: u64, _threadid: u32) {
        self.start(_time);
        println!("thread run ti:{_threadid}");
    }
    fn event_thread_stop(&mut self, _time: u64, _threadid: u32, _status: u16, _block_threadid: u32) {
        self.start(_time);
        println!("thread stop ti:{_threadid}");
    }
    fn event_thread_label(&mut self, _time: u64, _threadid: u32, _label: Vec<u8>) {
        self.start(_time);
        println!("thread label ti:{_threadid}");
    }
    fn event_thread_runnable(&mut self, _time: u64, _threadid: u32) {
        self.start(_time);
        println!("thread runnable ti:{_threadid}");
    }
    fn event_thread_migrate(&mut self, _time: u64, _threadid: u32, _capno: u16) {
        self.start(_time);
        println!("thread migrate ti:{_threadid} c:{_capno}");
    }
    fn event_thread_wakeup(&mut self, _time: u64, _threadid: u32, _capno: u16) {
        self.start(_time);
        println!("thread wakeup ti:{_threadid} c:{_capno}");
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum StopStatus {
    HeapOverflow = 1,
    StackOverflow = 2,
    ThreadYielding = 3,
    ThreadBlocked = 4,
    ThreadFinished = 5,
    ForeignCall = 6,
    BlockedOnMVar = 7,
    BlockedOnBlackHole = 8,
    BlockedOnRead = 9,
    BlockedOnWrite = 10,
    BlockedOnDelay = 11,
    BlockedOnSTM = 12,
    BlockedOnDoProc = 13,
    UNUSED2 = 14, // ? maybe used
    UNUSED3 = 15, // ? maybe used
    BlockedOnMsgThrowTo = 16,
    UNUSED4 = 17, // ? maybe used
    UNUSED5 = 18, // ? maybe used
    UNUSED6 = 19, // ? maybe used
    BlockedOnMVarRead = 20,
}

impl StopStatus {
    fn from(status: u16) -> StopStatus {
        assert!(
            (status > 0 && status <= 20),
            "unknown stop status: {status}");
        unsafe {
            *(&status as *const u16 as *const StopStatus)
        }
    }
}

#[derive(Debug)]
pub enum Ev {
    CapCreate {
        capno: u16,
    },
    CapDelete {
        capno: u16,
    },
    CapDisable {
        capno: u16,
    },

    TaskCreate {
        taskid: u64,
        capno: u16,
        kernel_tid: u64,
    },
    TaskMigrate {
        taskid: u64,
        from: u16,
        to: u16,
    },
    TaskDelete {
        taskid: u64,
    },

    ThreadCreate {
        id: u32,
    },
    ThreadLabel {
        id: u32,
        label: String,
    },
    ThreadRun {
        id: u32,
    },
    ThreadStop {
        id: u32,
        status: StopStatus,
        block_on: u32,
    },
    ThreadRunnable {
        id: u32,
    },
    ThreadMigrate {
        id: u32,
        capno: u16,
    },
    ThreadWakeup {
        id: u32,
        capno: u16,
    },
}

use std::collections::BTreeMap;

struct SchedEvents {
    ordered: BTreeMap<u64, Vec<(Option<u16>, Ev)>>,
    current_block_remaining: i64,
    current_block_capno: Option<u16>,
}

impl SchedEvents {
    fn new() -> Self {
        SchedEvents {
            ordered: BTreeMap::new(),
            current_block_remaining: 0,
            current_block_capno: None,
        }
    }

    fn add(&mut self, _time: u64, ev: Ev) {
        let block_cap = self.block_cap();
        self.ordered.entry(_time).or_default()
            .push((block_cap, ev));
    }

    fn block_cap(&self) -> Option<u16> {
        if self.current_block_remaining > 0 {
            self.current_block_capno
        } else {
            None
        }
    }
}

impl EventlogParser for SchedEvents {
    fn event_start(&mut self, _id: u16, _time: u64, _size: usize) {
        self.current_block_remaining -= (2 + 8 + _size) as i64;
    }

    fn event_block_marker(&mut self, _time: u64, _block_size: u32, _time_end: u64, _capno: u16) {
        let _capno = if _capno == !0 {
            None
        } else {
            Some(_capno)
        };

        self.current_block_capno = _capno;
        self.current_block_remaining = _block_size as i64;
    }

    fn event_unknown(&mut self, id: u16, _time: u64, _bytes: Vec<u8>) {
        panic!("unknown event {id} len={}", _bytes.len());
    }

    fn event_cap_create(&mut self, _time: u64, _capno: u16) {
        let ev = Ev::CapCreate { capno: _capno };
        self.add(_time, ev);
    }
    fn event_cap_delete(&mut self, _time: u64, _capno: u16) {
        let ev = Ev::CapDelete { capno: _capno };
        self.add(_time, ev);
    }
    fn event_cap_disable(&mut self, _time: u64, _capno: u16) {
        let ev = Ev::CapDisable { capno: _capno };
        self.add(_time, ev);
    }

    fn event_task_create(&mut self, _time: u64, taskid: u64, capno: u16, kernel_tid: u64) {
        let ev = Ev::TaskCreate { taskid, capno, kernel_tid };
        self.add(_time, ev);
    }
    fn event_task_migrate(&mut self, _time: u64, taskid: u64, from: u16, to: u16) {
        let ev = Ev::TaskMigrate { taskid, from, to };
        self.add(_time, ev);
    }
    fn event_task_delete(&mut self, _time: u64, taskid: u64) {
        let ev = Ev::TaskDelete { taskid };
        self.add(_time, ev);
    }

    fn event_thread_create(&mut self, _time: u64, id: u32) {
        let ev = Ev::ThreadCreate { id };
        self.add(_time, ev);
    }
    fn event_thread_run(&mut self, _time: u64, id: u32) {
        let ev = Ev::ThreadRun { id };
        self.add(_time, ev);
    }
    fn event_thread_stop(&mut self, _time: u64, id: u32, status: u16, block_on: u32) {
        let ev = Ev::ThreadStop { id, status: StopStatus::from(status), block_on };
        self.add(_time, ev);
    }
    fn event_thread_label(&mut self, _time: u64, id: u32, _label: Vec<u8>) {
        let ev = Ev::ThreadLabel { id, label: String::from_utf8(_label).unwrap() };
        self.add(_time, ev);
    }
    fn event_thread_runnable(&mut self, _time: u64, id: u32) {
        let ev = Ev::ThreadRunnable { id };
        self.add(_time, ev);
    }
    fn event_thread_migrate(&mut self, _time: u64, id: u32, capno: u16) {
        let ev = Ev::ThreadMigrate { id, capno };
        self.add(_time, ev);
    }
    fn event_thread_wakeup(&mut self, _time: u64, id: u32, capno: u16) {
        let ev = Ev::ThreadWakeup { id, capno };
        self.add(_time, ev);
    }
}




fn main() {
    // let mut printer = PrintEvents::new();
    // parse("./ghc-9.4.4.eventlog", &mut printer);
    let mut printer = SchedEvents::new();
    // parse("./ghc-9.4.4.eventlog", &mut printer);
    parse("./main.eventlog", &mut printer);
    for (time, events) in printer.ordered {
        for (mcap, event) in events {
            print!("[{time}]");
            match mcap {
                Some(x) => { print!("[{}]", x) },
                _ => {},
            }
            println!(" {event:?}");
        }
    }
}
