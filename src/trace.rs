use std::{
    collections::HashMap,
    error::Error,
    fs::{remove_file, File, OpenOptions},
    io::{Read, Seek},
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use serde::{Deserialize, Serialize};
use serde_binary::binary_stream;

use crate::{
    proxy_common::{check_prefix_dir, list_files_with_ext_in, unix_ts, ProxyErr},
    proxywireprotocol::{CounterSnapshot, CounterType, CounterValue, JobDesc, JobProfile},
};

#[derive(Serialize, Deserialize)]
struct TraceHeader {
    desc: JobDesc,
}

#[derive(Serialize, Deserialize, Clone)]
struct TraceCounter {
    id: u64,
    value: CounterType,
}

#[derive(Serialize, Deserialize, Clone)]
struct TraceCounterMetadata {
    id: u64,
    name: String,
    doc: String,
}

#[derive(Serialize, Deserialize, Clone)]
enum TraceFrame {
    Desc {
        ts: u64,
        desc: JobDesc,
    },
    CounterMetadata {
        ts: u64,
        metadata: TraceCounterMetadata,
    },
    Counters {
        ts: u64,
        counters: Vec<TraceCounter>,
    },
}

impl TraceFrame {
    fn ts(&self) -> u64 {
        match *self {
            TraceFrame::Desc { ts, desc: _ } => ts,
            TraceFrame::CounterMetadata { ts, metadata: _ } => ts,
            TraceFrame::Counters { ts, counters: _ } => ts,
        }
    }

    fn desc(self) -> Result<JobDesc, ProxyErr> {
        match self {
            TraceFrame::Desc { ts: _, desc } => Ok(desc),
            _ => Err(ProxyErr::new("Frame is not a jobdesc")),
        }
    }
}

#[allow(unused)]
struct TraceState {
    size: u64,
    lastwrite: u64,
    path: PathBuf,
    counters: HashMap<String, TraceCounterMetadata>,
    current_counter_id: u64,
}

impl TraceState {
    fn open(&self, create: bool) -> Result<File, std::io::Error> {
        if create {
            OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(&self.path)
        } else {
            OpenOptions::new().read(true).append(true).open(&self.path)
        }
    }

    fn len(&self) -> Result<u64, Box<dyn Error>> {
        let fd = self.open(false)?;
        Ok(fd.metadata()?.len())
    }

    fn desc(&mut self) -> Result<JobDesc, Box<dyn Error>> {
        let mut fd = self.open(false)?;
        let (data, _) = Self::read_frame_at(&mut fd, 0)?;

        if let Some(frame) = data {
            return Ok(frame.desc()?);
        }

        Err(ProxyErr::newboxed(
            "First frame of the trace is not a trace description",
        ))
    }

    fn offset_of_last_frame_start(fd: &mut File) -> Result<u64, Box<dyn Error>> {
        let total_size = fd.metadata()?.len();
        let mut offset = 0;
        loop {
            let mut size: [u8; 8] = [0; 8];
            fd.read_exact_at(&mut size, offset).unwrap();
            let size = u64::from_le_bytes(size);

            offset += 8;

            match (offset + size).cmp(&total_size) {
                std::cmp::Ordering::Equal => {
                    return Ok(offset - 8);
                }
                std::cmp::Ordering::Greater => {
                    return Err(ProxyErr::newboxed(
                        "Overrun of the file when scanning for EOF",
                    ));
                }
                _ => {}
            }

            offset += size;
        }
    }

    fn read_last(&mut self) -> Result<Option<TraceFrame>, Box<dyn Error>> {
        let mut fd = self.open(false)?;
        let off = Self::offset_of_last_frame_start(&mut fd)?;
        if off == (fd.metadata()?.len() - 1) {
            /* This is an empty frame */
            return Ok(None);
        }
        let (data, _) = Self::read_frame_at(&mut fd, off)?;
        Ok(data)
    }

    fn read_frame_at(fd: &mut File, off: u64) -> Result<(Option<TraceFrame>, u64), Box<dyn Error>> {
        let mut data: Vec<u8> = Vec::new();
        let mut current_offset = off;

        /* We expect an 8 bytes integer at the start */
        let mut len_data: [u8; 8] = [0; 8];
        if fd.read_at(&mut len_data, current_offset)? == 0 {
            /* EOF */
            return Ok((None, current_offset));
        }
        current_offset += 8;

        let mut left_to_read = u64::from_le_bytes(len_data);

        loop {
            let block_size = if left_to_read < 1024 {
                left_to_read as usize
            } else {
                1024
            };

            let mut buff: [u8; 1024] = [0; 1024];
            let len = fd.read_at(&mut buff[..block_size], current_offset)?;

            for c in buff.iter().take(len) {
                current_offset += 1;
                left_to_read -= 1;

                data.push(*c);

                if left_to_read == 0 {
                    let frame: TraceFrame =
                        serde_binary::from_slice(&data, binary_stream::Endian::Little)?;
                    return Ok((Some(frame), current_offset));
                }
            }
        }
    }

    fn check_counter(&mut self, counters: &Vec<CounterSnapshot>) -> Vec<TraceFrame> {
        let mut ret: Vec<TraceFrame> = Vec::new();
        for c in counters.iter() {
            if !self.counters.contains_key(&c.name) {
                let metadata = TraceCounterMetadata {
                    id: self.current_counter_id,
                    name: c.name.to_string(),
                    doc: c.doc.to_string(),
                };

                self.current_counter_id += 1;

                self.counters.insert(c.name.to_string(), metadata.clone());

                let frame = TraceFrame::CounterMetadata {
                    ts: unix_ts(),
                    metadata,
                };

                ret.push(frame)
            }
        }
        ret
    }

    fn counter_id(&self, counter: &CounterSnapshot) -> Option<u64> {
        if let Some(c) = self.counters.get(&counter.name) {
            return Some(c.id);
        }

        None
    }

    fn do_write_frame(fd: &mut File, frame: TraceFrame) -> Result<(), Box<dyn Error>> {
        let buff: Vec<u8> = serde_binary::to_vec(&frame, binary_stream::Endian::Little)?;

        // First write length
        let len: u64 = buff.len() as u64;
        let len = len.to_le_bytes();

        let endoff = fd.stream_position()?;
        fd.write_all_at(&len, endoff)?;

        // And then write buff
        let endoff = fd.stream_position()?;
        fd.write_at(&buff, endoff)?;

        Ok(())
    }

    fn write_frame(&self, frame: TraceFrame) -> Result<(), Box<dyn Error>> {
        let mut fd = self.open(false)?;

        Self::do_write_frame(&mut fd, frame)?;

        Ok(())
    }

    fn write_frames(&mut self, frames: Vec<TraceFrame>) -> Result<(), Box<dyn Error>> {
        if frames.is_empty() {
            return Ok(());
        }

        let mut fd = self.open(false)?;

        for f in frames {
            Self::do_write_frame(&mut fd, f)?;
        }

        self.lastwrite = unix_ts();
        self.size = fd.metadata()?.len();

        Ok(())
    }

    fn push(&mut self, counters: Vec<CounterSnapshot>) -> Result<(), Box<dyn Error>> {
        let new_counters: Vec<TraceFrame> = self.check_counter(&counters);

        self.write_frames(new_counters)?;

        let counters = counters
            .iter()
            .map(|v| TraceCounter {
                id: self.counter_id(v).unwrap(),
                value: v.ctype.clone(),
            })
            .collect();

        let frame = TraceFrame::Counters {
            ts: unix_ts(),
            counters,
        };

        self.write_frame(frame)?;

        Ok(())
    }

    fn read_all(&mut self) -> Result<(JobDesc, Vec<TraceFrame>), Box<dyn Error>> {
        let mut frames = Vec::new();

        let mut fd = self.open(false)?;

        /* First frame is the desc */
        let (mut frame, mut current_offset) = Self::read_frame_at(&mut fd, 0)?;

        if frame.is_none() {
            return Err(ProxyErr::newboxed("Failed to read first frame"));
        }

        let desc = frame.unwrap().desc()?;

        loop {
            (frame, current_offset) = Self::read_frame_at(&mut fd, current_offset)?;

            if frame.is_none() {
                return Ok((desc, frames));
            }

            /* Full frame */
            frames.push(frame.unwrap());
        }
    }

    fn new(path: &Path, job: &JobDesc) -> Result<TraceState, Box<dyn Error>> {
        let ret = TraceState {
            size: 0,
            lastwrite: 0,
            path: path.to_path_buf(),
            counters: HashMap::new(),
            current_counter_id: 0,
        };

        let mut fd = ret.open(true)?;

        // First thing save the jobdesc
        let desc = TraceFrame::Desc {
            ts: unix_ts(),
            desc: job.clone(),
        };

        TraceState::do_write_frame(&mut fd, desc)?;

        Ok(ret)
    }

    fn from(path: &Path) -> Result<TraceState, Box<dyn Error>> {
        let mut ret = TraceState {
            size: 0,
            lastwrite: 0,
            path: path.to_path_buf(),
            counters: HashMap::new(),
            current_counter_id: 0,
        };
        let lastframe = ret.read_last()?;

        ret.size = ret.len()?;

        if let Some(f) = lastframe {
            ret.lastwrite = f.ts();
        }

        Ok(ret)
    }
}

pub(crate) struct Trace {
    desc: JobDesc,
    state: Mutex<TraceState>,
    done: RwLock<bool>,
}

impl Trace {
    fn new_from_file(file: &String) -> Result<Trace, Box<dyn Error>> {
        let path = Path::new(&file);
        let mut state = TraceState::from(path)?;
        let mut desc = state.desc()?;
        /* Assume end time is the last profile write ~1 sec exact */
        if state.lastwrite != 0 {
            desc.end_time = state.lastwrite;
        }
        Ok(Trace {
            desc,
            state: Mutex::new(state),
            done: RwLock::new(false),
        })
    }

    fn name(prefix: &Path, desc: &JobDesc) -> PathBuf {
        let mut path = prefix.to_path_buf();
        path.push(format!("{}.trace", desc.jobid));
        path
    }

    fn new(prefix: &Path, desc: &JobDesc) -> Result<Trace, Box<dyn Error>> {
        let path = Trace::name(prefix, desc);
        if path.exists() {
            return Err(ProxyErr::newboxed(format!(
                "Cannot create trace it already exists at {}",
                path.to_string_lossy(),
            )));
        }

        let state = TraceState::new(&path, desc)?;

        Ok(Trace {
            desc: desc.clone(),
            state: Mutex::new(state),
            done: RwLock::new(false),
        })
    }

    pub(crate) fn desc(&self) -> &JobDesc {
        &self.desc
    }

    pub(crate) fn path(&self) -> String {
        self.state
            .lock()
            .unwrap()
            .path
            .to_string_lossy()
            .to_string()
    }

    pub(crate) fn push(&self, profile: JobProfile) -> Result<(), Box<dyn Error>> {
        let done = self.done.read().unwrap();

        if *done {
            return Err(ProxyErr::newboxed("Job is done"));
        }

        self.state.lock().unwrap().push(profile.counters)?;

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct TraceInfo {
    desc: JobDesc,
    size: u64,
    lastwrite: u64,
}

#[derive(Serialize)]
pub(crate) struct TraceRead {
    info: TraceInfo,
    frames: Vec<TraceFrame>,
}

impl TraceInfo {
    pub(crate) fn new(trace: &Trace) -> TraceInfo {
        let infos = trace.state.lock().unwrap();
        TraceInfo {
            desc: trace.desc.clone(),
            size: infos.size,
            lastwrite: infos.lastwrite,
        }
    }
}

pub(crate) struct TraceView {
    prefix: PathBuf,
    traces: RwLock<HashMap<String, Arc<Trace>>>,
}

impl TraceView {
    fn load_existing_traces(
        prefix: &PathBuf,
    ) -> Result<HashMap<String, Arc<Trace>>, Box<dyn Error>> {
        let mut ret: HashMap<String, Arc<Trace>> = HashMap::new();

        let files = list_files_with_ext_in(prefix, "trace")?;

        for f in files.iter() {
            match Trace::new_from_file(f) {
                Ok(t) => {
                    ret.insert(t.desc.jobid.to_string(), Arc::new(t));
                }
                Err(e) => {
                    log::error!("Failed to load trace from {} : {}", f, e);
                }
            }
        }

        Ok(ret)
    }

    pub(crate) fn list(&self) -> Vec<TraceInfo> {
        self.traces
            .read()
            .unwrap()
            .values()
            .map(|v| TraceInfo::new(v))
            .collect()
    }

    pub(crate) fn clear(&self, desc: &JobDesc) -> Result<(), Box<dyn Error>> {
        self.done(desc)?;
        let path = Trace::name(&self.prefix, desc);
        if path.is_file() {
            log::error!("Removing {}", path.to_string_lossy());
            remove_file(path)?;
        }
        Ok(())
    }

    pub(crate) fn read(
        &self,
        jobid: String,
        filter: Option<String>,
    ) -> Result<TraceRead, ProxyErr> {
        let ht = self.traces.read().unwrap();

        if let Some(trace) = ht.get(&jobid) {
            let (_, frames) = trace.state.lock().unwrap().read_all()?;

            let frames = if let Some(filter) = filter {
                let mut tmp_frames: Vec<TraceFrame> = Vec::new();
                let mut filter_id: Option<u64> = None;

                for f in frames.iter() {
                    match f {
                        TraceFrame::CounterMetadata { ts: _, metadata } => {
                            if metadata.name == filter {
                                tmp_frames.push(f.clone());
                                filter_id = Some(metadata.id);
                            }
                        }
                        TraceFrame::Desc { ts: _, desc: _ } => {}
                        TraceFrame::Counters { ts, counters } => {
                            if let Some(id) = filter_id {
                                let counters: Vec<TraceCounter> =
                                    counters.iter().filter(|v| v.id == id).cloned().collect();
                                if !counters.is_empty() {
                                    let counterframe = TraceFrame::Counters { ts: *ts, counters };
                                    tmp_frames.push(counterframe);
                                }
                            }
                        }
                    }
                }

                tmp_frames
            } else {
                frames
            };

            return Ok(TraceRead {
                info: TraceInfo::new(trace),
                frames,
            });
        }

        Err(ProxyErr::new(format!("No such trace id {}", jobid)))
    }

    pub(crate) fn get(&self, jobdesc: &JobDesc) -> Result<Arc<Trace>, Box<dyn Error>> {
        let mut ht = self.traces.write().unwrap();

        let trace = match ht.get(&jobdesc.jobid) {
            Some(v) => v.clone(),
            None => {
                let trace = Trace::new(&self.prefix, jobdesc)?;
                let ret = Arc::new(trace);
                ht.insert(jobdesc.jobid.to_string(), ret.clone());
                ret
            }
        };

        Ok(trace)
    }

    pub(crate) fn done(&self, job: &JobDesc) -> Result<(), Box<dyn Error>> {
        if let Some(j) = self.traces.write().unwrap().get_mut(&job.jobid) {
            *j.done.write().unwrap() = true;
        }

        self.traces.write().unwrap().remove(&job.jobid);
        Ok(())
    }

    pub(crate) fn new(prefix: &PathBuf) -> Result<TraceView, Box<dyn Error>> {
        let prefix = check_prefix_dir(prefix, "traces")?;
        let traces = RwLock::new(Self::load_existing_traces(&prefix)?);
        Ok(TraceView { prefix, traces })
    }
}
