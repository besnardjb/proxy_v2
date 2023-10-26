use reqwest::blocking::Client;
use retry::{delay::Fixed, retry};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::sleep;
use std::time::Duration;

use crate::proxy_common::unix_ts;
use crate::proxywireprotocol::{ApiResponse, CounterSnapshot, CounterType, JobDesc, JobProfile};

use super::proxy_common::{hostname, is_url_live, list_files_with_ext_in, unix_ts_us, ProxyErr};

/***********************
 * PROMETHEUS EXPORTER *
 ***********************/

struct ExporterEntry {
    value: Arc<RwLock<CounterSnapshot>>,
}

impl ExporterEntry {
    fn new(value: CounterSnapshot) -> ExporterEntry {
        ExporterEntry {
            value: Arc::new(RwLock::new(value)),
        }
    }
}

struct ExporterEntryGroup {
    basename: String,
    doc: String,
    ht: RwLock<HashMap<String, ExporterEntry>>,
}

impl ExporterEntryGroup {
    fn new(basename: String, doc: String) -> ExporterEntryGroup {
        ExporterEntryGroup {
            basename,
            doc,
            ht: RwLock::new(HashMap::new()),
        }
    }

    fn basename(name: String) -> String {
        let spl: Vec<&str> = name.split('{').collect();
        spl[0].to_string()
    }

    fn set(&self, value: CounterSnapshot) -> Result<(), ProxyErr> {
        match self.ht.write().unwrap().get_mut(&value.name) {
            Some(v) => {
                let mut val = v.value.write().unwrap();
                *val = value;
                Ok(())
            }
            None => Err(ProxyErr::new("Failed to set counter")),
        }
    }

    fn accumulate(&self, snapshot: &CounterSnapshot) -> Result<(), ProxyErr> {
        match self.ht.write().unwrap().get_mut(&snapshot.name) {
            Some(v) => {
                let mut val = v.value.write().unwrap();
                val.merge(snapshot)?;
                Ok(())
            }
            None => Err(ProxyErr::new("Failed to set counter")),
        }
    }

    fn push(&self, snapshot: CounterSnapshot) -> Result<(), ProxyErr> {
        let name = snapshot.name.to_string();
        if self.ht.read().unwrap().contains_key(&name) {
            return Ok(());
        } else {
            if name.contains('{') && !name.contains('}') {
                return Err(ProxyErr::new(
                    format!("Bad metric name '{}' unmatched brackets", name).as_str(),
                ));
            }
            let new = ExporterEntry::new(snapshot);
            self.ht.write().unwrap().insert(name, new);
        }

        Ok(())
    }

    fn serialize(&self) -> Result<String, ProxyErr> {
        let mut ret: String = String::new();

        ret += format!("# HELP {} {}\n", self.basename, self.doc).as_str();
        ret += format!("# TYPE {} counter\n", self.basename).as_str();

        for (_, exporter_counter) in self.ht.read().unwrap().iter() {
            // Acquire the Mutex for this specific ExporterEntry
            let value = exporter_counter.value.read().unwrap();
            ret += value.serialize().as_str();
        }

        Ok(ret)
    }

    fn snapshot(&self) -> Result<Vec<CounterSnapshot>, ProxyErr> {
        let mut ret: Vec<CounterSnapshot> = Vec::new();

        for (_, exporter_counter) in self.ht.read().unwrap().iter() {
            // Acquire the Mutex for this specific ExporterEntry
            let value = exporter_counter.value.read().unwrap();
            ret.push(value.clone());
        }

        Ok(ret)
    }
}

pub(crate) struct Exporter {
    ht: RwLock<HashMap<String, ExporterEntryGroup>>,
}

impl Exporter {
    pub(crate) fn new() -> Exporter {
        Exporter {
            ht: RwLock::new(HashMap::new()),
        }
    }

    pub(crate) fn accumulate(&self, value: &CounterSnapshot) -> Result<(), ProxyErr> {
        let basename = ExporterEntryGroup::basename(value.name.to_string());

        if let Some(exporter_counter) = self.ht.read().unwrap().get(basename.as_str()) {
            exporter_counter.accumulate(value)
        } else {
            return Err(ProxyErr::new(
                format!("No such key {} cannot set it", value.name).as_str(),
            ));
        }
    }

    pub(crate) fn set(&self, value: CounterSnapshot) -> Result<(), ProxyErr> {
        log::trace!("Exporter set {} {:?}", value.name, value);

        let basename = ExporterEntryGroup::basename(value.name.to_string());

        if let Some(exporter_counter) = self.ht.read().unwrap().get(basename.as_str()) {
            exporter_counter.set(value)
        } else {
            return Err(ProxyErr::new(
                format!("No such key {} cannot set it", value.name).as_str(),
            ));
        }
    }

    pub(crate) fn push(&self, value: &CounterSnapshot) -> Result<(), ProxyErr> {
        log::trace!("Exporter push {:?}", value);

        let basename = ExporterEntryGroup::basename(value.name.to_string());

        let mut ht = self.ht.write().unwrap();

        if ht.get(basename.as_str()).is_some() {
            return Ok(());
        } else {
            let ncnt = ExporterEntryGroup::new(basename.to_owned(), value.doc.to_string());
            ncnt.push(value.clone())?;
            ht.insert(basename, ncnt);
        }

        Ok(())
    }

    pub(crate) fn serialize(&self) -> Result<String, ProxyErr> {
        let mut ret: String = String::new();

        for (_, exporter_counter) in self.ht.read().unwrap().iter() {
            ret += exporter_counter.serialize()?.as_str();
        }

        ret += "# EOF\n";

        Ok(ret)
    }

    pub(crate) fn profile(&self, desc: &JobDesc) -> Result<JobProfile, ProxyErr> {
        let mut ret = JobProfile {
            desc: desc.clone(),
            counters: Vec::new(),
        };

        for (_, exporter_counter) in self.ht.read().unwrap().iter() {
            let snaps = exporter_counter.snapshot()?;
            ret.counters.extend(snaps);
        }

        Ok(ret)
    }
}

struct ProxyScraper {
    target_url: String,
    state: HashMap<String, JobProfile>,
    factory: Arc<ExporterFactory>,
    period: u64,
    last_scrape: u64,
}

impl ProxyScraper {
    fn new(target_url: String, period: u64, factory: Arc<ExporterFactory>) -> ProxyScraper {
        ProxyScraper {
            target_url,
            state: HashMap::new(),
            factory,
            period,
            last_scrape: 0,
        }
    }

    fn scrape(&mut self) -> Result<(), Box<dyn Error>> {
        if unix_ts() - self.last_scrape < self.period {
            /* Not to be scraped yet */
            return Ok(());
        }

        log::debug!("Scraping {}", self.target_url);

        let mut deleted: Vec<JobDesc> = Vec::new();

        let client = Client::new();
        let response = client.get(&self.target_url).send()?;

        // Check if the response was successful (status code 200 OK)
        if response.status().is_success() {
            // Deserialize the JSON response into your data structure
            let mut profiles: Vec<JobProfile> = response.json()?;
            let new_keys: HashSet<String> = profiles.iter().map(|v| v.desc.jobid.clone()).collect();

            /* First detect if a job has left */
            for (k, v) in self.state.iter() {
                if !new_keys.contains(k) {
                    /* Key has been dropped save name in list for notify */
                    deleted.push(v.desc.clone());
                    self.factory.relax_job(&v.desc)?;
                }
            }

            /* Remove all deleted from the shadow state */
            for k in deleted.iter() {
                self.state.remove(&k.jobid);
            }

            /* Now Update Values */

            for p in profiles.iter_mut() {
                log::debug!(
                    "#### Processing {} from {} #####",
                    p.desc.jobid,
                    self.target_url
                );
                let cur: JobProfile;
                if let Some(previous) = self.state.get(&p.desc.jobid) {
                    /* We clone previous snapshot before substracting */
                    cur = p.clone();
                    p.substract(previous)?;
                } else {
                    /* New Job Register in Job List */
                    let _ = self.factory.resolve_job(&p.desc, false);
                    cur = p.to_owned();
                }

                if let Some(exporter) = self.factory.resolve_by_id(&p.desc.jobid) {
                    for cnt in p.counters.iter() {
                        exporter.push(cnt)?;
                        exporter.accumulate(cnt)?;
                    }
                } else {
                    return Err(ProxyErr::newboxed("No such JobID"));
                }

                /* Now insert the non-substracted for next call state */
                self.state.insert(p.desc.jobid.to_string(), cur);
            }
        } else {
            return Err(ProxyErr::newboxed("Failed to make scraping request"));
        }

        self.last_scrape = unix_ts();

        Ok(())
    }
}

struct PerJobRefcount {
    desc: JobDesc,
    counter: i32,
    exporter: Arc<Exporter>,
    saveprofile: bool,
}

impl Drop for PerJobRefcount {
    fn drop(&mut self) {
        log::debug!("Dropping per job exporter for {}", self.desc.jobid);
    }
}

impl PerJobRefcount {
    fn profile(&self) -> Result<JobProfile, ProxyErr> {
        self.exporter.profile(&self.desc)
    }
}

pub(crate) struct ExporterFactory {
    main: Arc<Exporter>,
    perjob: Mutex<HashMap<String, PerJobRefcount>>,
    prefix: PathBuf,
    scrapes: Mutex<HashMap<String, ProxyScraper>>,
}

fn create_dir_or_fail(path: &PathBuf) {
    if let Err(e) = std::fs::create_dir(path) {
        log::error!(
            "Failed to create directory at {} : {}",
            path.to_str().unwrap_or(""),
            e
        );
        exit(1);
    }
}

impl ExporterFactory {
    fn check_profile_dir(path: &PathBuf) {
        // Main directory
        if !path.exists() {
            create_dir_or_fail(path);
        } else if !path.is_dir() {
            log::error!(
                "{} is not a directory cannot use it as per job profile prefix",
                path.to_str().unwrap_or("")
            );
            exit(1);
        }

        // Profile subdirectory
        let mut profile_dir = path.clone();
        profile_dir.push("profiles");

        if !profile_dir.exists() {
            create_dir_or_fail(&profile_dir);
        }

        // Partial subdirectory
        let mut partial_dir = path.clone();
        partial_dir.push("partial");

        if !partial_dir.exists() {
            create_dir_or_fail(&partial_dir);
        }
    }

    fn profile_parse_jobid(target: &String) -> Result<String, Box<dyn Error>> {
        let path = PathBuf::from(target);
        let filename = path
            .file_name()
            .ok_or("Failed to parse path")?
            .to_string_lossy()
            .to_string();

        if let Some(jobid) = filename.split("___").next() {
            return Ok(jobid.to_string());
        }

        Err(ProxyErr::newboxed("Failed to parse jobid"))
    }

    fn accumulate_a_profile(profile_dir: &Path, target: &String) -> Result<(), Box<dyn Error>> {
        let file = fs::File::open(target)?;
        let mut content: JobProfile = serde_json::from_reader(file)?;

        /* Compute path to profile for given job  */
        let jobid = ExporterFactory::profile_parse_jobid(target)?;
        let mut target_prof = profile_dir.to_path_buf();
        target_prof.push(format!("{}.profile", jobid));

        if target_prof.is_file() {
            /* We need to load and accumulate the existing profile */
            let e_profile_file = fs::File::open(&target_prof)?;
            let existing_prof: JobProfile = serde_json::from_reader(e_profile_file)?;
            /* Aggregate the existing content */
            content.merge(existing_prof)?;
        }

        /* Overwrite the profile */
        let outfile = fs::File::create(target_prof)?;
        serde_json::to_writer(outfile, &content)?;

        /* If we are here we managed to read and collect the file */
        fs::remove_file(target).ok();

        Ok(())
    }

    fn aggregate_profiles(prefix: PathBuf) -> Result<(), Box<dyn Error>> {
        let mut profile_dir = prefix.clone();
        profile_dir.push("profiles");

        let mut partial_dir = prefix.clone();
        partial_dir.push("partial");

        assert!(profile_dir.is_dir());
        assert!(partial_dir.is_dir());

        loop {
            let ret = list_files_with_ext_in(&partial_dir, ".partialprofile")?;

            for partial in ret.iter() {
                if let Err(e) = ExporterFactory::accumulate_a_profile(&profile_dir, partial) {
                    log::error!("Failed to process {} : {}", partial, e.to_string());
                } else {
                    log::trace!("Aggregated profile {}", partial);
                }
            }

            sleep(Duration::from_secs(1));
        }
    }

    fn run_scrapping(&self) {
        loop {
            let mut to_delete: Vec<String> = Vec::new();

            /* Scrape all the candidates */
            for (k, v) in self.scrapes.lock().unwrap().iter_mut() {
                if let Err(e) = v.scrape() {
                    log::error!("Failed to scrape {} : {}", k, e);
                    to_delete.push(k.to_string());
                }
            }

            /* Remove failed scrapes */
            for k in to_delete {
                self.scrapes.lock().unwrap().remove(&k);
            }

            sleep(Duration::from_secs(1));
        }
    }

    pub(crate) fn add_scrape(
        factory: Arc<ExporterFactory>,
        url: &String,
        period: u64,
    ) -> Result<(), Box<dyn Error>> {
        let job_url = "http://".to_string() + url + "/job";
        is_url_live(job_url.as_str())?;

        let new = ProxyScraper::new(job_url, period, factory.clone());
        factory
            .scrapes
            .lock()
            .unwrap()
            .insert(new.target_url.to_string(), new);
        Ok(())
    }

    pub(crate) fn join(
        root_server: &String,
        my_server_address: &String,
        period: u64,
    ) -> Result<(), ProxyErr> {
        let pivot_url = root_server.to_string() + "/pivot?from=" + my_server_address;

        /* We add some delay as the root server may get smashed */
        let resp = retry(Fixed::from_millis(2000).take(5), || {
            ApiResponse::query(&pivot_url)
        })?;

        let target_url = "http://".to_string()
            + resp.operation.as_str()
            + "/join?to="
            + my_server_address
            + "&period="
            + period.to_string().as_str();

        /* We add some delay as the root server may get smashed */
        match ApiResponse::query(&target_url) {
            Ok(_) => {
                log::info!(
                    "Joining aggregating proxy {} with period {}",
                    root_server,
                    period
                );
                Ok(())
            }
            Err(e) => Err(ProxyErr::from(e)),
        }
    }

    pub(crate) fn new(profile_prefix: PathBuf, aggregate: bool) -> Arc<ExporterFactory> {
        ExporterFactory::check_profile_dir(&profile_prefix);

        if aggregate {
            let thread_prefix = profile_prefix.clone();
            // Start Aggreg thread
            std::thread::spawn(move || {
                ExporterFactory::aggregate_profiles(thread_prefix).unwrap();
            });
        }

        let ret = Arc::new(ExporterFactory {
            main: Arc::new(Exporter::new()),
            perjob: Mutex::new(HashMap::new()),
            prefix: profile_prefix,
            scrapes: Mutex::new(HashMap::new()),
        });

        let scrape_ref = ret.clone();
        // Start Scraping thread
        std::thread::spawn(move || {
            scrape_ref.run_scrapping();
        });

        /* This creates a job entry for the cumulative job */
        let main_job = PerJobRefcount {
            desc: JobDesc {
                jobid: "main".to_string(),
                command: "Sum of all Jobs".to_string(),
                size: 0,
                nodelist: "".to_string(),
                partition: "".to_string(),
                cluster: "".to_string(),
                run_dir: "".to_string(),
                start_time: 0,
                end_time: 0,
            },
            exporter: ret.main.clone(),
            counter: 1,
            saveprofile: false,
        };
        ret.perjob
            .lock()
            .unwrap()
            .insert(main_job.desc.jobid.to_string(), main_job);

        ret
    }

    pub(crate) fn get_main(&self) -> Arc<Exporter> {
        self.main.clone()
    }

    pub(crate) fn resolve_by_id(&self, jobid: &String) -> Option<Arc<Exporter>> {
        if let Some(r) = self.perjob.lock().unwrap().get(jobid) {
            return Some(r.exporter.clone());
        }
        None
    }

    pub(crate) fn resolve_job(&self, desc: &JobDesc, tobesaved: bool) -> Arc<Exporter> {
        let mut ht: std::sync::MutexGuard<'_, HashMap<String, PerJobRefcount>> =
            self.perjob.lock().unwrap();

        let v = match ht.get_mut(&desc.jobid) {
            Some(e) => {
                log::debug!("Cloning existing job exporter for {}", &desc.jobid);
                /* Incr Refcount */
                e.counter += 1;
                /* Make sure save flags match */
                if tobesaved {
                    e.saveprofile = true;
                }
                log::debug!(
                    "ACQUIRING Per Job exporter {} has refcount {}",
                    &desc.jobid,
                    e.counter
                );
                e.exporter.clone()
            }
            None => {
                log::debug!("Creating new job exporter for {}", &desc.jobid);
                let new = PerJobRefcount {
                    desc: desc.clone(),
                    exporter: Arc::new(Exporter::new()),
                    counter: 1,
                    saveprofile: tobesaved,
                };
                let ret = new.exporter.clone();
                ht.insert(desc.jobid.to_string(), new);
                ret
            }
        };

        v
    }

    fn saveprofile(&self, per_job: &PerJobRefcount, desc: &JobDesc) -> Result<(), Box<dyn Error>> {
        let snap = per_job.exporter.profile(desc)?;

        let mut target_dir = self.prefix.clone();
        target_dir.push("partial");

        let hostname = hostname();

        let fname = format!(
            "{}___{}.{}.partialprofile",
            desc.jobid,
            hostname,
            unix_ts_us()
        );

        target_dir.push(fname);

        log::debug!(
            "Saving partial profile to {}",
            target_dir.to_str().unwrap_or("")
        );

        let file = fs::File::create(target_dir)?;

        serde_json::to_writer(file, &snap)?;

        Ok(())
    }

    pub(crate) fn list_jobs(&self) -> Vec<JobDesc> {
        self.perjob
            .lock()
            .unwrap()
            .values()
            .map(|k| k.desc.clone())
            .collect()
    }

    pub(crate) fn profiles(&self) -> Vec<JobProfile> {
        let mut ret: Vec<JobProfile> = Vec::new();

        if let Ok(ht) = self.perjob.lock() {
            for v in ht.values() {
                if let Ok(p) = v.profile() {
                    ret.push(p);
                }
            }
        }

        ret
    }

    pub(crate) fn profile_of(&self, jobid: &String) -> Result<JobProfile, ProxyErr> {
        if let Some(elem) = self.perjob.lock().unwrap().get(jobid) {
            return elem.profile();
        }

        Err(ProxyErr::new("No such Job ID"))
    }

    pub(crate) fn relax_job(&self, desc: &JobDesc) -> Result<(), Box<dyn Error>> {
        let mut ht: std::sync::MutexGuard<'_, HashMap<String, PerJobRefcount>> =
            self.perjob.lock().unwrap();

        if let Some(job_entry) = ht.get_mut(&desc.jobid) {
            job_entry.counter -= 1;
            log::debug!(
                "RELAXING Per Job exporter {} has refcount {}",
                desc.jobid,
                job_entry.counter
            );
            assert!(0 <= job_entry.counter);
            if job_entry.counter == 0 {
                /* Serialize */
                if let Some(perjob) = ht.get(&desc.jobid) {
                    self.saveprofile(perjob, desc)?;
                    /* Delete */
                    ht.remove(&desc.jobid);
                }
            }
        } else {
            return Err(ProxyErr::newboxed("No such job to remove"));
        }

        Ok(())
    }

    pub(crate) fn push(
        &self,
        name: &str,
        doc: &str,
        ctype: CounterType,
        perjob_exporter: Option<Arc<Exporter>>,
    ) -> Result<(), ProxyErr> {
        let snapshot = CounterSnapshot {
            name: name.to_string(),
            doc: doc.to_string(),
            ctype,
        };
        self.get_main().push(&snapshot)?;

        if let Some(e) = perjob_exporter {
            e.push(&snapshot)?;
        }

        Ok(())
    }

    pub(crate) fn accumulate(
        &self,
        name: &str,
        ctype: CounterType,
        perjob_exporter: Option<Arc<Exporter>>,
    ) -> Result<(), ProxyErr> {
        let snapshot = CounterSnapshot {
            name: name.to_string(),
            doc: "".to_string(),
            ctype,
        };

        self.get_main().accumulate(&snapshot)?;

        if let Some(e) = perjob_exporter {
            e.accumulate(&snapshot)?;
        }

        Ok(())
    }
}
