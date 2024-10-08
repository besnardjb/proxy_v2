use crate::proxy_common::{self, gen_range, ProxyErr};
use crate::proxywireprotocol::{self, ApiResponse, CounterSnapshot, CounterType, JobProfile};
use crate::{
    exporter::{Exporter, ExporterFactory},
    proxy_common::{concat_slices, derivate_time_serie, hostname, parse_bool},
};

use colored::Colorize;
use rouille::input::json::JsonError;
use rouille::{Request, Response};
use serde::Deserialize;
use static_files::Resource;
use std::collections::HashMap;
use std::path::Path;

use std::sync::{Arc, Mutex};

use crate::squeue;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

/*************
 * WEBSERVER *
 *************/

struct ClientPivot {
    url: String,
    refcount: u32,
    child: Vec<String>,
}

impl ClientPivot {
    fn new(url: String) -> ClientPivot {
        ClientPivot {
            url,
            refcount: 0,
            child: Vec::new(),
        }
    }

    fn mapto(&mut self, child_url: String) {
        self.refcount += 1;
        self.child.push(child_url);
    }

    fn is_partial(&self) -> bool {
        if (self.refcount < 2) && (1 < self.refcount) {
            return true;
        }

        false
    }

    fn is_free(&self) -> bool {
        if self.refcount < 2 {
            return true;
        }

        false
    }
}

pub(crate) struct Web {
    port: u32,
    factory: Arc<ExporterFactory>,
    static_files: HashMap<String, Resource>,
    known_client: Mutex<Vec<ClientPivot>>,
}

enum WebResponse {
    #[allow(unused)]
    Html(String),
    StaticHtml(String, &'static str, Vec<u8>),
    Text(String),
    BadReq(String),
    Success(String),
    #[allow(unused)]
    Redirect302(String),
    Native(Response),
    NoSuchDoc(),
}

impl WebResponse {
    fn serialize(self: WebResponse) -> Response {
        match self {
            WebResponse::Html(s) => Response::html(s),
            WebResponse::StaticHtml(name, mime, data) => {
                log::trace!("{} {} as {}", "STATIC".yellow(), name, mime);
                Response::from_data(mime, data)
            }
            WebResponse::Text(s) => Response::text(s),
            WebResponse::BadReq(operation) => {
                let r = ApiResponse {
                    operation,
                    success: false,
                };
                Response::json(&r).with_status_code(400)
            }
            WebResponse::Success(operation) => {
                let r = ApiResponse {
                    operation,
                    success: true,
                };
                Response::json(&r)
            }
            WebResponse::NoSuchDoc() => Response::empty_404(),
            WebResponse::Redirect302(url) => Response::redirect_302(url),
            WebResponse::Native(response) => response,
        }
    }
}

impl Web {
    pub(crate) fn new(port: u32, factory: Arc<ExporterFactory>) -> Web {
        let web = Web {
            port,
            factory,
            static_files: generate()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            known_client: Mutex::new(Vec::new()),
        };
        /* Add myself in the URLs */
        web.known_client
            .lock()
            .unwrap()
            .push(ClientPivot::new(web.url()));
        web
    }

    pub(crate) fn url(&self) -> String {
        format!("{}:{}", hostname(), self.port)
    }

    fn default_doc() -> String {
        "".to_string()
    }

    fn parse_key_value(req: &Request) -> (Option<String>, Option<f64>, String, Option<String>) {
        let key: Option<String>;
        let svalue: Option<String>;
        let parsed_value: Option<f64>;
        let error: Option<String>;
        let doc: String; // Optionnal
        #[derive(Deserialize)]
        struct KeyValue {
            key: String,
            value: String,
            #[serde(default = "Web::default_doc")]
            doc: String,
        }

        match req.method() {
            "GET" => {
                key = req.get_param("key");
                svalue = req.get_param("value");
                match req.get_param("doc") {
                    Some(e) => {
                        doc = e;
                    }
                    None => {
                        doc = Web::default_doc();
                    }
                }
                error = None;
            }
            "POST" => {
                let kv: Result<KeyValue, rouille::input::json::JsonError> =
                    rouille::input::json_input(req);
                match kv {
                    Ok(v) => {
                        key = Some(v.key);
                        svalue = Some(v.value);
                        doc = v.doc;
                        error = None;
                    }
                    Err(e) => {
                        key = None;
                        svalue = None;
                        doc = Web::default_doc();
                        error = Some(e.to_string());
                    }
                }
            }
            _ => {
                key = None;
                svalue = None;
                doc = Web::default_doc();
                error = Some(format!("No such request type {}", req.method()));
            }
        }

        if let Some(val) = svalue {
            match val.parse::<f64>() {
                Ok(val) => {
                    parsed_value = Some(val);
                }
                Err(e) => {
                    return (key, None, doc, Some(e.to_string()));
                }
            }
        } else {
            parsed_value = None;
        }

        (key, parsed_value, doc, error)
    }

    fn handle_set(&self, req: &Request) -> WebResponse {
        let key: Option<String>;
        let error: Option<String>;
        let value: Option<f64>;
        let _doc: String;

        (key, value, _doc, error) = Web::parse_key_value(req);

        if error.is_some() {
            return WebResponse::BadReq(format!("Error parsing parameters: {}", error.unwrap()));
        }

        let key = key.unwrap();
        let value = value.unwrap();

        let snap = CounterSnapshot {
            name: key,
            doc: "".to_string(),
            ctype: CounterType::Counter {
                ts: proxy_common::unix_ts_us() as u64,
                value,
            },
        };

        match self.factory.get_main().set(snap) {
            Ok(_) => WebResponse::Success("set".to_string()),
            Err(e) => WebResponse::BadReq(e.to_string()),
        }
    }

    fn handle_accumulate(&self, req: &Request) -> WebResponse {
        let key: Option<String>;
        let error: Option<String>;
        let value: Option<f64>;
        let _doc: String;

        (key, value, _doc, error) = Web::parse_key_value(req);

        if error.is_some() {
            return WebResponse::BadReq(format!("Error parsing parameters: {}", error.unwrap()));
        }

        let key = key.unwrap();
        let value = value.unwrap();

        let snap = CounterSnapshot {
            name: key,
            doc: "".to_string(),
            ctype: CounterType::Counter {
                ts: proxy_common::unix_ts_us() as u64,
                value,
            },
        };

        match self.factory.get_main().accumulate(&snap, false) {
            Ok(_) => WebResponse::Success("inc".to_string()),
            Err(e) => WebResponse::BadReq(e.to_string()),
        }
    }

    fn handle_push(&self, req: &Request) -> WebResponse {
        let key: Option<String>;
        let error: Option<String>;
        let doc: String;

        (key, _, doc, error) = Web::parse_key_value(req);

        if error.is_some() {
            return WebResponse::BadReq(format!("Error parsing parameters: {}", error.unwrap()));
        }

        let key = key.unwrap();

        let snap = CounterSnapshot {
            name: key,
            doc,
            ctype: CounterType::newcounter(),
        };

        match self.factory.get_main().push(&snap) {
            Ok(_) => WebResponse::Success("push".to_string()),
            Err(e) => WebResponse::BadReq(e.to_string()),
        }
    }

    fn serialize_exporter(exporter: &Arc<Exporter>) -> WebResponse {
        match exporter.serialize() {
            Ok(v) => WebResponse::Text(v),
            Err(e) => WebResponse::BadReq(e.to_string()),
        }
    }

    fn handle_metrics(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("job") {
            if let Some(exporter) = self.factory.resolve_by_id(&jobid) {
                Web::serialize_exporter(&exporter)
            } else {
                WebResponse::BadReq(format!("No such jobid {}", jobid))
            }
        } else {
            Web::serialize_exporter(&self.factory.get_main())
        }
    }

    fn handle_queue(&self, _req: &Request) -> WebResponse {
        match squeue::SqueueJobList::init() {
            Ok(q) => WebResponse::Native(Response::json(&q)),
            Err(e) => WebResponse::BadReq(format!("Failed to call squeue: {}", e)),
        }
    }

    fn handle_queue_model(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("job") {
            match squeue::SqueueJobList::init() {
                Ok(jlist) => match jlist.job_cmd(&jobid) {
                    Some(pending) => {
                        if let Ok(jsonl) = self.factory.profile_store.get_jsonl_by_cmd(&pending) {
                            return WebResponse::Native(Response::text(jsonl));
                        } else {
                            return WebResponse::BadReq(format!(
                                "Failed to retrieve a model for : {} ({})",
                                jobid, pending
                            ));
                        }
                    }
                    None => {
                        WebResponse::BadReq(format!("Failed to retrieve a job with id : {}", jobid))
                    }
                },
                Err(e) => WebResponse::BadReq(format!("Failed to call squeue: {}", e)),
            }
        } else {
            WebResponse::BadReq(format!("A Job Parameter is required"))
        }
    }

    fn handle_tracelist(&self, _req: &Request) -> WebResponse {
        let traces = self.factory.trace_store.list();
        WebResponse::Native(Response::json(&traces))
    }

    fn handle_traceread(&self, req: &Request) -> WebResponse {
        let filter = req.get_param("filter");
        if let Some(jobid) = req.get_param("job") {
            match self.factory.trace_store.read(&jobid, filter) {
                Ok(data) => {
                    return WebResponse::Native(Response::json(&data));
                }
                Err(e) => {
                    return WebResponse::BadReq(format!("Failed to generate data {}", e));
                }
            }
        }
        WebResponse::BadReq("No job GET parameter passed".to_string())
    }

    fn handle_tracemetrics(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("job") {
            match self.factory.trace_store.metrics(&jobid) {
                Ok(data) => {
                    return WebResponse::Native(Response::json(&data));
                }
                Err(e) => {
                    return WebResponse::BadReq(format!("Failed to generate data {}", e));
                }
            }
        }
        WebResponse::BadReq("No job GET parameter passed".to_string())
    }

    fn handle_traceplot(&self, req: &Request) -> WebResponse {
        #[derive(Deserialize)]
        struct Plotdef {
            jobid: String,
            filter: String,
            derivate: bool,
        }

        let (jobid, filter, derivate) = match req.method() {
            "GET" => (
                req.get_param("jobid"),
                req.get_param("filter"),
                match req.get_param("derivate") {
                    Some(e) => parse_bool(e.as_str()),
                    None => false,
                },
            ),
            "POST" => {
                let sel: Result<Plotdef, JsonError> = rouille::input::json_input(req);
                match sel {
                    Ok(e) => (Some(e.jobid), Some(e.filter), e.derivate),
                    Err(_) => {
                        return WebResponse::BadReq(
                            "Failed to parse plot POST request".to_string(),
                        );
                    }
                }
            }
            _ => (None, None, false),
        };

        if filter.is_none() {
            return WebResponse::BadReq("No filter GET parameter passed".to_string());
        }

        if let Some(jobid) = jobid {
            match self.factory.trace_store.plot(&jobid, filter.unwrap()) {
                Ok(data) => {
                    /* Do we need to derivate ? */
                    let fdata = if derivate {
                        derivate_time_serie(&data)
                    } else {
                        data
                    };
                    return WebResponse::Native(Response::json(&fdata));
                }
                Err(e) => {
                    return WebResponse::BadReq(format!("Failed to generate data {}", e));
                }
            }
        }
        WebResponse::BadReq("No job GET parameter passed".to_string())
    }

    fn handle_join_list(&self, _req: &Request) -> WebResponse {
        let scrapes = self.factory.list_scrapes();
        WebResponse::Native(Response::json(&scrapes))
    }

    fn handle_join(&self, req: &Request) -> WebResponse {
        let to = req.get_param("to");

        if to.is_none() {
            return WebResponse::BadReq("No to parameter passed".to_string());
        }

        let to = to.unwrap();

        if to.contains("http") {
            return WebResponse::BadReq(
                "To should not be an URL (with http://) but host:port".to_string(),
            );
        }

        let period: u64 = match req.get_param("period") {
            Some(e) => e.parse::<u64>().unwrap_or(1000),
            None => 1000,
        };

        if let Err(e) = ExporterFactory::add_scrape(self.factory.clone(), &to, period) {
            return WebResponse::BadReq(format!("Failed to add {} for scraping : {}", to, e));
        }

        WebResponse::Success(format!("Added {} for scraping", to))
    }

    fn handle_pivot(&self, req: &Request) -> WebResponse {
        let from = req.get_param("from");

        if from.is_none() {
            return WebResponse::BadReq("No from parameter passed".to_string());
        }

        let from: String = from.unwrap().clone();

        if from.contains("http") {
            return WebResponse::BadReq(
                "From should not be an URL (with http://) but host:port".to_string(),
            );
        }

        let mut clients = self.known_client.lock().unwrap();

        let mut target = clients.iter_mut().find(|v| v.is_partial());

        if target.is_none() {
            target = clients.iter_mut().find(|v| v.is_free());
        }

        let resp: WebResponse;

        if let Some(target) = target {
            target.mapto(from.to_string());

            log::info!(
                "Pivot response to {} is {} with ref {}",
                from,
                target.url,
                target.refcount
            );

            resp = WebResponse::Success(target.url.to_string());
        } else {
            resp = WebResponse::BadReq("Did not match any server".to_string());
        }

        clients.push(ClientPivot::new(from));

        resp
    }

    fn handle_topo(&self, _req: &Request) -> WebResponse {
        let mut resp: Vec<(String, String)> = Vec::new();

        for c in self.known_client.lock().unwrap().iter() {
            for t in &c.child {
                resp.push((c.url.clone(), t.clone()))
            }
        }

        if resp.is_empty() {
            resp.push((self.url().to_string(), self.url().to_string()));
        }

        WebResponse::Native(Response::json(&resp))
    }

    fn handle_job(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("job") {
            match self.factory.profile_of(&jobid, true) {
                Ok(p) => WebResponse::Native(Response::json(&p)),
                Err(e) => WebResponse::BadReq(e.to_string()),
            }
        } else {
            /* For all we skip null values to be faster */
            let all = self.factory.profiles(false);
            WebResponse::Native(Response::json(&all))
        }
    }

    fn handle_joblist(&self, _req: &Request) -> WebResponse {
        let jobs = self.factory.list_jobs();

        match serde_json::to_vec(&jobs) {
            Ok(_v) => WebResponse::Native(Response::json(&jobs)),
            Err(e) => WebResponse::BadReq(e.to_string()),
        }
    }

    fn handle_alarms(&self, _req: &Request) -> WebResponse {
        let trigerred_alarms = self.factory.check_alarms();
        WebResponse::Native(Response::json(&trigerred_alarms))
    }

    fn handle_add_alarms(&self, req: &Request) -> WebResponse {
        #[derive(Deserialize)]
        struct AlarmDef {
            name: String,
            target: String,
            metric: String,
            operation: String,
            value: f64,
        }

        let al: Result<AlarmDef, JsonError> = rouille::input::json_input(req);

        match al {
            Ok(def) => {
                match self.factory.add_alarm(
                    def.name,
                    def.target,
                    def.metric,
                    def.operation,
                    def.value,
                ) {
                    Ok(_) => WebResponse::Success("alarm registered".to_string()),
                    Err(e) => WebResponse::BadReq(e.to_string()),
                }
            }
            Err(e) => WebResponse::BadReq(e.to_string()),
        }
    }

    fn handle_del_alarms(&self, req: &Request) -> WebResponse {
        let (tjob, to_del) = match req.method() {
            "GET" => match (req.get_param("targetjob"), req.get_param("name")) {
                (Some(t), Some(v)) => (t, v),
                _ => {
                    return WebResponse::BadReq("Missing 'name' GET parameter".to_string());
                }
            },
            "POST" => {
                #[derive(Deserialize)]
                struct ToDel {
                    target: String,
                    name: String,
                }
                let al: Result<ToDel, JsonError> = rouille::input::json_input(req);
                match al {
                    Ok(v) => (v.target, v.name),
                    Err(e) => {
                        return WebResponse::BadReq(format!("Failed to parse json {}", e));
                    }
                }
            }
            _ => {
                return WebResponse::BadReq("No such request type".to_string());
            }
        };

        if let Err(e) = self.factory.delete_alarm(&tjob, &to_del) {
            WebResponse::BadReq(format!("Failed to delete {}", e))
        } else {
            WebResponse::Success(format!("Deleted {} from {}", to_del, tjob))
        }
    }

    fn handle_list_alarms(&self, _: &Request) -> WebResponse {
        let alarms = self.factory.list_alarms();
        WebResponse::Native(Response::json(&alarms))
    }

    fn handle_list_profiles(&self, _: &Request) -> WebResponse {
        let prof = self.factory.profile_store.get_profile_list();
        WebResponse::Native(Response::json(&prof))
    }

    fn handle_get_profiles(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("jobid") {
            if let Ok(prof) = self.factory.profile_store.get_profile(&jobid) {
                return WebResponse::Native(Response::json(&prof));
            }
            return WebResponse::BadReq(format!("Failed to get {}", jobid));
        }
        WebResponse::BadReq("A GET parameter jobid must be passed".to_string())
    }

    fn job_id_to_profile(&self, jobid: &str) -> Option<JobProfile> {
        // First assume it is a profile

        if let Ok(prof) = self.factory.profile_store.get_profile(jobid) {
            /* Found */
            Some(prof)
        } else if let Ok(prof) = self.factory.profile_of(jobid, false) {
            Some(prof)
        } else {
            None
        }
    }

    fn handle_extrap_get_jsonl(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("jobid") {
            let prof = if let Some(prof) = self.job_id_to_profile(&jobid) {
                prof
            } else {
                return WebResponse::BadReq("No such jobid".to_string());
            };

            if let Ok(jsonl) = self.factory.profile_store.get_jsonl(&prof.desc) {
                return WebResponse::Native(Response::text(jsonl));
            }
            return WebResponse::BadReq(format!("Failed to get {}", jobid));
        }
        WebResponse::BadReq("A GET parameter for a reference jobid must be passed".to_string())
    }

    fn handle_ftio_get_model(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("jobid") {
            if let Some(model) = self.factory.trace_store.get_job_freq_model(jobid) {
                return WebResponse::Native(Response::json(&model));
            } else {
                return WebResponse::BadReq("No such jobid".to_string());
            }
        }
        WebResponse::BadReq("A GET parameter for a reference jobid must be passed".to_string())
    }

    fn handle_extrap_get_model(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("jobid") {
            let prof = if let Some(prof) = self.job_id_to_profile(&jobid) {
                prof
            } else {
                return WebResponse::BadReq("No such jobid".to_string());
            };

            if let Ok(model) = self.factory.profile_store.extrap_model_list(&prof.desc) {
                return WebResponse::Native(Response::json(&model));
            }
            return WebResponse::BadReq(format!("Failed to get {}", jobid));
        }
        WebResponse::BadReq("A GET parameter for a reference jobid must be passed".to_string())
    }

    fn handle_extrap_get_model_for(&self, req: &Request) -> WebResponse {
        if let Some(jobids) = req.get_param("jobids") {
            let job_list: Vec<&str> = jobids.split(',').collect();

            let profiles: Vec<JobProfile> = job_list
                .iter()
                .filter_map(|v| self.job_id_to_profile(v))
                .collect();

            /* Make sure we had all the jobids */
            if job_list.len() != profiles.len() {
                let profiles_id: Vec<String> =
                    profiles.iter().map(|v| v.desc.jobid.clone()).collect();
                return WebResponse::BadReq(format!(
                    "Requested Jobs {:?} (count = {}) and only retrieved {:?} (count = {})",
                    job_list,
                    job_list.len(),
                    profiles_id,
                    profiles.len()
                ));
            }
        }

        WebResponse::BadReq(
            "A GET parameter for a reference jobid (or a comma separated list of) must be passed"
                .to_string(),
        )
    }

    fn handle_get_json_trace(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("jobid") {
            if let Ok(jsontrace) = self.factory.trace_store.export(&jobid) {
                return WebResponse::Native(Response::json(&jsontrace));
            }
            return WebResponse::BadReq(format!("Failed to get {}", jobid));
        }
        WebResponse::BadReq("A GET parameter jobid must be passed".to_string())
    }

    fn handle_extrap_plot_model(&self, req: &Request) -> WebResponse {
        let (jobid, metric, start, end, step) = match req.method() {
            "GET" => match (
                req.get_param("jobid"),
                req.get_param("metric"),
                req.get_param("start"),
                req.get_param("end"),
                req.get_param("step"),
            ) {
                (Some(jobid), Some(metric), Some(start), Some(end), step) => {
                    let step = if let Some(sstep) = step {
                        if let Ok(step) = sstep.parse::<f64>() {
                            step
                        } else {
                            return WebResponse::BadReq(
                                "Failed to parse the step parameter as an integer".to_string(),
                            );
                        }
                    } else {
                        1.0
                    };

                    let (start, end) =
                        if let (Ok(start), Ok(end)) = (start.parse::<f64>(), end.parse::<f64>()) {
                            (start, end)
                        } else {
                            return WebResponse::BadReq(
                                "Failed to parse the start/end parameter as an integer".to_string(),
                            );
                        };

                    (jobid, metric, start, end, step)
                }
                _ => {
                    return WebResponse::BadReq("Missing 'name' GET parameter".to_string());
                }
            },
            "POST" => {
                #[derive(Deserialize, Debug)]
                struct ToDel {
                    jobid: String,
                    metric: String,
                    start: f64,
                    end: f64,
                    step: Option<f64>,
                }
                let al: Result<ToDel, JsonError> = rouille::input::json_input(req);
                match al {
                    Ok(v) => {
                        let step = if let Some(step) = v.step { step } else { 1.0 };
                        (v.jobid, v.metric, v.start, v.end, step)
                    }
                    Err(e) => {
                        log::error!("Failed to process plot : {}", e);
                        return WebResponse::BadReq(format!("Failed to parse json {}", e));
                    }
                }
            }
            _ => {
                return WebResponse::BadReq("No such request type".to_string());
            }
        };

        let step = if let Some(sstep) = req.get_param("step") {
            if let Ok(step) = sstep.parse::<f64>() {
                step
            } else {
                return WebResponse::BadReq(
                    "Failed to parse the step parameter as an integer".to_string(),
                );
            }
        } else {
            1.0
        };

        let prof = if let Some(prof) = self.job_id_to_profile(&jobid) {
            prof
        } else {
            return WebResponse::BadReq("No such jobid".to_string());
        };

        let range: Vec<f64> = match gen_range(start, end, step) {
            Ok(range) => range,
            Err(e) => {
                return WebResponse::BadReq(format!("Failed to generate range {}", e));
            }
        };

        match self
            .factory
            .profile_store
            .extrap_model_plot(&prof.desc, metric, &range)
        {
            Ok(model) => return WebResponse::Native(Response::json(&model)),
            Err(e) => WebResponse::BadReq(format!("Failed to plot {}", e)),
        }
    }

    fn handle_profile_points(&self, req: &Request) -> WebResponse {
        if let Some(jobid) = req.get_param("jobid") {
            let prof = if let Some(prof) = self.job_id_to_profile(&jobid) {
                prof
            } else {
                return WebResponse::BadReq("No such jobid".to_string());
            };

            match self
                .factory
                .profile_store
                .generate_profile_points(&prof.desc)
            {
                Ok(ret) => return WebResponse::Native(Response::json(&ret)),
                Err(e) => return WebResponse::BadReq(e.to_string()),
            }
        }

        WebResponse::BadReq("A GET parameter for a reference jobid must be passed".to_string())
    }

    fn handle_list_profiles_per_cmd(&self, _: &Request) -> WebResponse {
        let prof = self.factory.profile_store.gather_by_command();
        WebResponse::Native(Response::json(&prof))
    }

    fn serve_static_file(&self, url: &str) -> WebResponse {
        /* remove slash before */
        assert!(url.starts_with('/'));

        let url = url[1..].to_string();

        if let Some(file) = self.static_files.get(&url) {
            /* Handle HTML templating on the fly */
            let data = if url.ends_with(".html") {
                let header = self.static_files.get("header.html.in").unwrap();
                let footer = self.static_files.get("footer.html.in").unwrap();
                concat_slices([header.data, file.data, footer.data])
            } else {
                file.data.to_vec()
            };

            WebResponse::StaticHtml(url, file.mime_type, data)
        } else {
            log::warn!("{} {}", "No such file".red(), url);
            WebResponse::NoSuchDoc()
        }
    }

    fn parse_url(surl: &str) -> (String, String) {
        let url = surl[1..].to_string();

        assert!(surl.starts_with('/'));

        let path = Path::new(&url);

        let segments: Vec<String> = path
            .iter()
            .map(|v| v.to_string_lossy().to_string())
            .collect();

        if segments.len() == 1 {
            /* Only a single entry /metric/ */
            return (segments[0].to_string(), "".to_string());
        } else if segments.is_empty() {
            /* Only / */
            return ("/".to_string(), "".to_string());
        }

        let prefix = segments
            .iter()
            .take(segments.len() - 1)
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("/");
        let resource = segments.last().take().unwrap().to_string();

        (prefix, resource)
    }

    pub(crate) fn run_blocking(self) {
        let hostname = hostname();
        log::info!(
            "Proxy webserver listening on http://{}:{}",
            hostname,
            self.port
        );

        rouille::start_server(format!("0.0.0.0:{}", self.port), move |request| {
            let url = request.url();

            let (prefix, resource) = Web::parse_url(&url);

            log::trace!(
                "GET {} mapped to ({} , {})",
                request.raw_url(),
                prefix.red(),
                resource.yellow()
            );

            let resp: WebResponse = match prefix.as_str() {
                "/" => self.serve_static_file("/index.html"),
                "set" => self.handle_set(request),
                "accumulate" => self.handle_accumulate(request),
                "push" => self.handle_push(request),
                "metrics" => self.handle_metrics(request),
                "job" => match resource.as_str() {
                    "list" => self.handle_joblist(request),
                    "" => self.handle_job(request),
                    _ => WebResponse::BadReq(url),
                },
                "queue" => match resource.as_str() {
                    "list" => self.handle_queue(request),
                    "model" => self.handle_queue_model(request),
                    _ => WebResponse::BadReq(url),
                },
                "trace" => match resource.as_str() {
                    "list" => self.handle_tracelist(request),
                    "read" => self.handle_traceread(request),
                    "plot" => self.handle_traceplot(request),
                    "metrics" => self.handle_tracemetrics(request),
                    "json" => self.handle_get_json_trace(request),
                    _ => WebResponse::BadReq(url),
                },
                "profiles" => match resource.as_str() {
                    "" => self.handle_list_profiles(request),
                    "get" => self.handle_get_profiles(request),
                    "percmd" => self.handle_list_profiles_per_cmd(request),
                    "extrap" => self.handle_extrap_get_jsonl(request),
                    "points" => self.handle_profile_points(request),
                    _ => WebResponse::BadReq(url),
                },
                "model" => match resource.as_str() {
                    "ftio" => self.handle_ftio_get_model(request),
                    "download" => self.handle_extrap_get_jsonl(request),
                    "get" => self.handle_extrap_get_model(request),
                    "plot" => self.handle_extrap_plot_model(request),
                    _ => WebResponse::BadReq(url),
                },
                "pivot" => self.handle_pivot(request),
                "topo" => self.handle_topo(request),
                "join" => match resource.as_str() {
                    "" => self.handle_join(request),
                    "list" => self.handle_join_list(request),
                    _ => WebResponse::BadReq(url),
                },
                "alarms" => match resource.as_str() {
                    "" => self.handle_alarms(request),
                    "add" => self.handle_add_alarms(request),
                    "del" => self.handle_del_alarms(request),
                    "list" => self.handle_list_alarms(request),
                    _ => WebResponse::BadReq(url),
                },
                _ => self.serve_static_file(url.as_str()),
            };

            resp.serialize()
        });
    }
}
