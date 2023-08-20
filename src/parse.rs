use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum EventSize {
    Variable,
    Fixed(u16),
}

pub struct EventType {
    pub size: EventSize,
    pub descr: String,
}

// an os thread has a cap
// a cap is running a thread

pub trait EventlogParser {
    fn event_start(&mut self, _id: u16, _time: u64, _size: usize) {}

    fn event_unknown(&mut self, _tag: u16, _time: u64, _bytes: Vec<u8>) {}

    fn event_block_marker(&mut self, _time: u64, _block_size: u32, _time_end: u64, _capno: u16) {}
    fn event_rts_identifier(&mut self, _time: u64, _capset: u32, _name: Vec<u8>) {}

    fn event_wall_clock_time(&mut self, _time: u64, _capset: u32, _sec: u64, _nsec: u32) {}
    fn event_osprocess_pid(&mut self, _time: u64, _capset: u32, _pid: u32) {}
    fn event_osprocess_ppid(&mut self, _time: u64, _capset: u32, _ppid: u32) {}
    fn event_program_args(&mut self, _time: u64, _args: Vec<u8>) {}
    fn event_program_env(&mut self, _time: u64, _env: Vec<u8>) {}

    fn event_spark_counters(&mut self, _time: u64, _counters: [u64; 7]) {}

    fn event_thread_create(&mut self, _time: u64, _threadid: u32) {}
    fn event_thread_run(&mut self, _time: u64, _threadid: u32) {}
    fn event_thread_stop(&mut self, _time: u64, _threadid: u32, _status: u16, _block_threadid: u32) {}
    fn event_thread_label(&mut self, _time: u64, _threadid: u32, _label: Vec<u8>) {}
    fn event_thread_runnable(&mut self, _time: u64, _threadid: u32) {}
    fn event_thread_migrate(&mut self, _time: u64, _threadid: u32, _capno: u16) {}
    fn event_thread_wakeup(&mut self, _time: u64, _threadid: u32, _capno: u16) {}

    fn event_task_create(&mut self, _time: u64, _taskid: u64, _capno: u16, _k_threadid: u64) {}
    fn event_task_migrate(&mut self, _time: u64, _taskid: u64, _from_capno: u16, _to_capno: u16) {}
    fn event_task_delete(&mut self, _time: u64, _taskid: u64) {}

    fn event_request_seq_gc(&mut self, _time: u64) {}
    fn event_request_par_gc(&mut self, _time: u64) {}

    fn event_gc_start(&mut self, _time: u64) {}
    fn event_gc_end(&mut self, _time: u64) {}
    fn event_gc_work(&mut self, _time: u64) {}
    fn event_gc_idle(&mut self, _time: u64) {}
    fn event_gc_done(&mut self, _time: u64) {}
    fn event_gc_global_sync(&mut self, _time: u64) {}

    fn event_gc_stats_ghc(&mut self, _time: u64, _capset: u32, _gen: u16, _copied: u64, _slop: u64, _fragmentation: u64, _threads: u32, _max_copied: u64, _total_copied: u64, _balanced_copied: u64) {}

    fn event_heap_info_ghc(&mut self, _time: u64, _capset: u32, _gen: u16, _max_heap: u64, _alloc_size: u64, _mblock_size: u64, _block_size: u64) {}

    fn event_heap_allocated(&mut self, _time: u64, _capset: u32, _allocated_bytes: u64) {}
    fn event_heap_size(&mut self, _time: u64, _capset: u32, _size: u64) {}
    fn event_heap_live(&mut self, _time: u64, _capset: u32, _size: u64) {}
    fn event_blocks_size(&mut self, _time: u64, _capset: u32, _blocks: u64) {}
    fn event_mem_return(&mut self, _time: u64, _capset: u32, _mblocks: u32, _retain: u32, _return_: u32) {}

    fn event_user_msg(&mut self, _time: u64, _bytes: Vec<u8>) {}

    fn event_cap_create(&mut self, _time: u64, _capno: u16) {}
    fn event_cap_delete(&mut self, _time: u64, _capno: u16) {}
    fn event_cap_disable(&mut self, _time: u64, _capno: u16) {}

    fn event_capset_create(&mut self, _time: u64, _capset: u32, _type_: u16) {}
    fn event_capset_delete(&mut self, _time: u64, _capset: u32) {}
    fn event_capset_assign_cap(&mut self, _time: u64, _capset: u32, _capno: u16) {}
    fn event_capset_remove_cap(&mut self, _time: u64, _capset: u32, _capno: u16) {}
}


///
/// ```
/// EventLog :
///       EVENT_HEADER_BEGIN 4xWord8 -- 'hdre'
///       EVENT_HET_BEGIN 4xWord8 -- 'hetb'
///       EventType*
///       EVENT_HET_END 4xWord8 -- 'hete'
///       EVENT_HEADER_END 4xWord8 -- 'hetb'
///       EVENT_DATA_BEGIN 4xWord8 -- 'datb'
///       Event*
///       EVENT_DATA_END Word16 -- 0xffff
///
/// EventType :
///       EVENT_ET_BEGIN
///       Word16         -- event type id, unique identifier for this event
///       Int16          -- >=0  size of the event record in bytes (minus the event type id and timestamp fields)
///                      -- -1   variable size
///       Word32         -- size of the event description in bytes
///       Word8*         -- event description, UTF8 encoded string describing the event
///       Word32         -- size of the extra info in bytes
///       Word8*         -- extra info (for future extensions)
///       EVENT_ET_END   --
///
/// Event :
///       Word16         -- event type id, as included in the event log header
///       Word64         -- timestamp (nanoseconds)
///       [Word16]       -- length of the rest (optional, for variable-sized events only)
///       ... event specific info ...
/// ```
pub fn parse<File: AsRef<Path>, Parser: EventlogParser>(file: File, handle: &mut Parser) {
    let file = OpenOptions::new()
        .read(true)
        .open(file).unwrap();

    let mut reader = std::io::BufReader::new(file);

    let mut event_types = HashMap::new();

    macro_rules! check_constant {
        ($comp:expr, $err:expr) => {{
            let mut buf = [0u8; std::mem::size_of_val($comp)];
            reader.read_exact(&mut buf[..]).unwrap();
            assert!(&buf == $comp, $err);
        }}
    }

    macro_rules! skip {
        ($len:expr) => {{
            if $len != 0 {
                reader.seek_relative($len).unwrap();
            }
        }}
    }

    macro_rules! bytes {
        ($len:literal) => {{
            let mut buf = [0u8; $len];
            reader.read_exact(&mut buf[..]).unwrap();
            buf
        }};
        ($len:expr) => {{
            let mut buf = vec![0u8; $len];
            reader.read_exact(&mut buf[..]).unwrap();
            buf
        }};
    }

    macro_rules! num {
        ($ty:ty) => {{
            let mut buf = [0u8; std::mem::size_of::<$ty>()];
            reader.read_exact(&mut buf[..]).unwrap();
            <$ty>::from_be_bytes(buf)
        }}
    }

    check_constant!(b"hdrb", "header begin");
    check_constant!(b"hetb", "header event types begin");

    // read event types
    loop {
        let etb = bytes!(4);

        if &etb == b"hete" {
            // header event types end, no more types
            break;
        }

        assert!(&etb == b"etb\0",
            "event type begin");

        let id = num!(u16);

        let size = num!(i16);
        let size = if size == -1 {
            EventSize::Variable
        } else if size >= 0 {
            EventSize::Fixed(size as u16)
        } else {
            panic!("event size bad {}", size)
        };

        let descr_size = num!(u32);
        let descr = bytes!(descr_size as usize);
        let descr = String::from_utf8(descr).unwrap();

        // println!("{id} {size:?} {descr}");

        event_types.insert(id, EventType {
            size,
            descr,
        });

        let extra_size = num!(u32);
        skip!(extra_size as i64);

        check_constant!(b"ete\0", "event type end");
    }

    check_constant!(b"hdre", "header end");

    check_constant!(b"datb", "data begin");


    // parse all the events
    loop {
        let id = num!(u16);
        if id == 0xffff {
            // we've reached the end of the event log data
            break;
        }

        let time = num!(u64);

        let et = event_types.get(&id).unwrap();
        let size = match et.size {
            EventSize::Variable => {
                let size = num!(u16);
                size as usize
            },
            EventSize::Fixed(size) => {
                size as usize
            },
        };

        handle.event_start(id, time, size);

        match id {
            // CREATE_THREAD
            0 => {
                let threadid = num!(u32);
                handle.event_thread_create(time, threadid);
            },
            // RUN_THREAD
            1 => {
                let threadid = num!(u32);
                handle.event_thread_run(time, threadid);
            },
            // STOP_THREAD
            2 => {
                let threadid = num!(u32);
                let status = num!(u16);
                let block_threadid = num!(u32);
                handle.event_thread_stop(time, threadid, status, block_threadid);
            },
            // THREAD_RUNNABLE
            3 => {
                let threadid = num!(u32);
                handle.event_thread_runnable(time, threadid);
            },
            // MIGRATE_THREAD
            4 => {
                let threadid = num!(u32);
                let capno = num!(u16);
                handle.event_thread_migrate(time, threadid, capno);
            },
            // THREAD_WAKEUP
            8 => {
                let threadid = num!(u32);
                let capno = num!(u16);
                handle.event_thread_wakeup(time, threadid, capno);
            },
            // GC_START
            9 => {
                handle.event_gc_start(time);
            }
            // GC_END
            10 => {
                handle.event_gc_end(time);
            }
            // REQUEST_PAR_GC
            11 => {
                handle.event_request_seq_gc(time);
            }
            // REQUEST_PAR_GC
            12 => {
                handle.event_request_par_gc(time);
            }
            // BLOCK_MARKER
            18 => {
                let block_size = num!(u32);
                let time_end = num!(u64);
                let cap_no = num!(u16);
                handle.event_block_marker(time, block_size, time_end, cap_no);
            },
            // USER_MSG
            19 => {
                let message = bytes!(size);
                handle.event_user_msg(time, message);
            },
            // GC_IDLE
            20 => {
                handle.event_gc_idle(time);
            },
            // GC_WORK
            21 => {
                handle.event_gc_work(time);
            },
            // GC_DONE
            22 => {
                handle.event_gc_done(time);
            },
            // CAPSET_CREATE
            25 => {
                let capset = num!(u32);
                let type_ = num!(u16);
                handle.event_capset_create(time, capset, type_);
            },
            // CAPSET_DELETE
            26 => {
                let capset = num!(u32);
                handle.event_capset_delete(time, capset);
            },
            // CAPSET_ASSIGN_CAP
            27 => {
                let capset = num!(u32);
                let capno = num!(u16);
                handle.event_capset_assign_cap(time, capset, capno);
            },
            // CAPSET_REMOVE_CAP
            28 => {
                let capset = num!(u32);
                let capno = num!(u16);
                handle.event_capset_remove_cap(time, capset, capno);
            },
            // RTS_IDENTIFIER
            29 => {
                let capset = num!(u32);
                let bytes = bytes!(size - 4);
                handle.event_rts_identifier(time, capset, bytes);
            },
            // PROGRAM_ARGS
            30 => {
                let args = bytes!(size);
                handle.event_program_args(time, args);
            },
            // OSPROCESS_PID
            32 => {
                let capset = num!(u32);
                let pid = num!(u32);
                handle.event_osprocess_pid(time, capset, pid);
            }
            // OSPROCESS_PPID
            33 => {
                let capset = num!(u32);
                let ppid = num!(u32);
                handle.event_osprocess_ppid(time, capset, ppid);
            }
            // SPARK_COUNTERS
            34 => {
                let mut counters = [0; 7];
                for i in 0..7 {
                    counters[i] = num!(u64);
                }
                handle.event_spark_counters(time, counters);
            }
            // WALL_CLOCK_TIME
            43 => {
                let capset = num!(u32);
                let sec = num!(u64);
                let nsec = num!(u32);
                handle.event_wall_clock_time(time, capset, sec, nsec);
            },
            // THREAD_LABEL
            44 => {
                let threadid = num!(u32);
                let label = bytes!(size - 4);
                handle.event_thread_label(time, threadid, label);
            },
            // CAP_CREATE
            45 => {
                let capno = num!(u16);
                handle.event_cap_create(time, capno);
            },
            // CAP_DELETE
            46 => {
                let capno = num!(u16);
                handle.event_cap_delete(time, capno);
            },
            // CAP_DELETE
            47 => {
                let capno = num!(u16);
                handle.event_cap_disable(time, capno);
            },
            // HEAP_ALLOCATED
            49 => {
                let capset = num!(u32);
                let allocated_bytes = num!(u64);
                handle.event_heap_allocated(time, capset, allocated_bytes);
            },
            // HEAP_SIZE
            50 => {
                let capset = num!(u32);
                let size = num!(u64);
                handle.event_heap_size(time, capset, size);
            },
            // HEAP_LIVE
            51 => {
                let capset = num!(u32);
                let size = num!(u64);
                handle.event_heap_live(time, capset, size);
            },
            // HEAP_INFO_GHC
            52 => {
                let capset = num!(u32);
                let gen = num!(u16);
                let max_heap = num!(u64);
                let alloc_size = num!(u64);
                let mblock_size = num!(u64);
                let block_size = num!(u64);
                handle.event_heap_info_ghc(time, capset, gen, max_heap, alloc_size, mblock_size, block_size);
            },
            // GC_STATS_GHC
            53 => {
                let capset = num!(u32);
                let gen = num!(u16);
                let copied = num!(u64);
                let slop = num!(u64);
                let fragmentation = num!(u64);
                let threads = num!(u32);
                let max_copied = num!(u64);
                let total_copied = num!(u64);
                let balanced_copied = num!(u64);

                handle.event_gc_stats_ghc(time, capset, gen, copied, slop, fragmentation, threads, max_copied, total_copied, balanced_copied);
            },
            // GC_GLOBAL_SYNC
            54 => {
                handle.event_gc_global_sync(time);
            },
            // TASK_CREATE
            55 => {
                let taskid = num!(u64);
                let capno = num!(u16);
                let k_threadid = num!(u64);
                handle.event_task_create(time, taskid, capno, k_threadid);
            },
            // TASK_MIGRATE
            56 => {
                let taskid = num!(u64);
                let from_capno = num!(u16);
                let to_capno = num!(u16);
                handle.event_task_migrate(time, taskid, from_capno, to_capno);
            },
            // TASK_DELETE
            57 => {
                let taskid = num!(u64);
                handle.event_task_delete(time, taskid);
            },
            // MEM_RETURN
            90 => {
                let capset = num!(u32);
                let mblocks = num!(u32);
                let retain = num!(u32);
                let return_ = num!(u32);
                handle.event_mem_return(time, capset, mblocks, retain, return_);
            },
            // BLOCKS_SIZE
            91 => {
                let capset = num!(u32);
                let blocks = num!(u64);
                handle.event_blocks_size(time, capset, blocks);
            },
            _ => {
                let bytes = bytes!(size);
                handle.event_unknown(id, time, bytes);
            },
        }

    }
}
