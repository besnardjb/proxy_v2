use std::error::Error;
use std::process::exit;
use std::thread::{self, sleep};
use std::time::Duration;
mod proxy_common;
use proxy_common::{get_proxy_path, init_log};

mod exporter;
use exporter::ExporterFactory;

mod proxy;
use proxy::UnixProxy;

mod webserver;
use webserver::Web;

mod extrap;
mod profiles;
mod proxywireprotocol;
mod scrapper;
mod systemmetrics;
mod trace;

extern crate clap;

use clap::Parser;

/// ADMIRE project Instrumentation Proxy
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Port number of the HTTP server
    #[arg(short, long, default_value_t = 1337)]
    port: u32,

    // Path of the UNIX proxy for the gateway
    #[arg(short, long)]
    unix: Option<String>,

    /// Should profile aggregation be deactivated
    #[arg(short, long, default_value_t = false)]
    inhibit_profile_agreggation: bool,

    /// Subservers to be scrapped (optionnal comma separated list) use ADDR\@[PERIOD in ms] to set the scraping period
    #[arg(short, long, value_delimiter = ',')]
    sub_proxies: Option<Vec<String>>,

    /// Address of the proxy to pivot on to build a proxy tree
    #[arg(short, long)]
    root_proxy: Option<String>,

    /// Maximum trace size to maintain in the file-system in MB
    #[arg(short, long)]
    max_trace_size: Option<f64>,
}

fn parse_period(arg: &String) -> (String, u64) {
    let mut spl = arg.split('@');

    let url = spl.next();
    let stime = spl.next();

    if url.is_none() || stime.is_none() {
        return (arg.to_string(), 1000);
    }

    match str::parse::<u64>(stime.unwrap()) {
        Ok(v) => (url.unwrap().to_string(), v),
        Err(e) => {
            log::error!("Failed to parse scrape time in {} : {}", arg, e);
            (arg.to_string(), 5)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    init_log();

    let args = Args::parse();

    let mut profile_prefix = dirs::home_dir().unwrap();
    profile_prefix.push(".proxyprofiles");

    let max_trace_size = if let Some(max_size) = args.max_trace_size {
        max_size * 1024.0 * 1024.0
    } else {
        // Default is 32 Mb
        1024.0 * 1024.0 * 32.0
    };

    log::info!(
        "Max trace size is {} MB",
        max_trace_size / (1024.0 * 1024.0)
    );

    // The central storage is the exporter
    let factory = ExporterFactory::new(
        profile_prefix,
        !args.inhibit_profile_agreggation,
        max_trace_size as usize,
    )?;

    if let Some(urls) = args.sub_proxies {
        for url in urls.iter() {
            let (url, freq) = parse_period(url);
            log::info!("Inserting scrape {} every {} second(s)", url, freq);
            if let Err(e) = ExporterFactory::add_scrape(factory.clone(), &url, freq) {
                log::error!("Failed add scrape : {}", e);
            }
        }
    }

    let unix = if let Some(unix) = args.unix {
        unix.clone()
    } else {
        get_proxy_path()
    };

    // Create the UNIX proxy with a reference to the exporter
    let proxy = UnixProxy::new(unix, factory.clone())?;

    // Run the proxy detached with a ref to the exporter data
    thread::spawn(move || proxy.run());

    // Start the webserver part with a reference to the exporter
    let web = Web::new(args.port, factory.clone());

    let web_url = web.url();

    thread::spawn(move || {
        /* Wait for server to start before joining as the server will back-connect  */
        sleep(Duration::from_secs(3));
        if let Some(root) = args.root_proxy {
            let (url, period) = parse_period(&root);

            if let Err(e) = ExporterFactory::join(&url, &web_url, period) {
                log::error!("Failed to register in root server {}: {}", root, e);
                exit(1);
            }
        }
    });

    web.run_blocking();

    Ok(())
}
