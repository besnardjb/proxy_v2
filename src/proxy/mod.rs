use std::io::Read;
use std::sync::Arc;
use std::error::Error;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;

use serde_json;

use crate::proxywireprotocol::JobDesc;

use super::proxy_common::ProxyErr;
use super::exporter::{Exporter, ExporterFactory};

use super::proxywireprotocol::ProxyCommand;

/********************
 * UNIX DATA SERVER *
 ********************/

 pub(crate) struct UnixProxy
 {
	listener : UnixListener,
	factory : Arc<ExporterFactory>
}

struct PerClientState
{
	factory : Arc<ExporterFactory>,
	main_exporter : Arc<Exporter>,
	job_exporter : Option<Arc<Exporter>>,
	job_desc : Option<JobDesc>
}

impl UnixProxy
{
	fn handle_command(per_client_state :&mut PerClientState, command : ProxyCommand) ->  Result<(), Box<dyn Error>>
	{
		log::debug!("{:?}", command);
		match command
		{
			ProxyCommand::Desc(desc) => {
	
				per_client_state.main_exporter.push(desc.name.as_str(), desc.doc.as_str(), desc.ctype.clone())?;
				if let Some(e) = per_client_state.job_exporter.clone()
				{
					e.push(desc.name.as_str(), desc.doc.as_str(), desc.ctype)?;
				}
			},
			ProxyCommand::Value(value) => {
				per_client_state.main_exporter.accumulate(value.name.as_str(), value.value)?;
				if let Some(e) = per_client_state.job_exporter.clone()
				{
					e.accumulate(value.name.as_str(), value.value)?;
				}
			}
			ProxyCommand::JobDesc(d) => {
				per_client_state.job_desc = Some(d);

				if let Some(desc) = &mut per_client_state.job_desc
				{
					if desc.jobid != ""
					{
						/* No need to start the exporter if the jobid is empty */
						per_client_state.job_exporter = Some(per_client_state.factory.resolve_job(&desc));
					}
				}


				}
		}
		Ok(())
	}


	fn handle_client(factory : Arc<ExporterFactory>, mut stream : UnixStream) -> Result<(), Box<dyn Error>>
	{
		let mut received_data : Vec<u8> = Vec::new();

		let mut per_client_state =PerClientState{
			factory : factory.clone(),
			main_exporter : factory.get_main(),
			job_exporter : None,
			job_desc : None
		};

		loop {
			let mut buff : [u8; 1024] = [0; 1024];
			let len = stream.read(& mut buff)?;

         if len == 0
			{
				break;
			}

			for i in 0..len
			{
				if buff[i] == 0
				{
					/* Full command */
					let cmd : ProxyCommand = serde_json::from_slice(&received_data)?;
					UnixProxy::handle_command(&mut per_client_state, cmd)?;
					received_data.clear();
				}
				else
				{
					received_data.push(buff[i]);
				}
			}
		}

		if let Some(desc) = per_client_state.job_desc
		{
			if desc.jobid != ""
			{
				per_client_state.factory.relax_job(&desc)?;
			}
		}

		Ok(())
	}

	pub(crate) fn run(&self) -> Result<(), ProxyErr>
	{
		for stream in self.listener.incoming() {
			match stream {
				Ok(stream) => {
					log::debug!("New connection");

					let factory = self.factory.clone();

					// Handle the connection in a new thread.
					thread::spawn(move || {
						match UnixProxy::handle_client(factory, stream) {
							Ok(_) => {log::debug!("Client left");}
							Err(e) => {log::error!("Proxy server closing on client : {}", e.to_string());}
						}
					});
				}
				Err(err) => {
					log::error!("Error accepting connection: {:?}", err);
				}
			}
		}

	  Ok(())
	}

	pub(crate) fn new(socket_path : String, factory : Arc<ExporterFactory>) -> Result<UnixProxy, Box<dyn Error>>
	{
		let path = Path::new(&socket_path);

		if path.exists()
		{
			std::fs::remove_file(path)?;
		}

		let listener = UnixListener::bind(path)?;

		let proxy = UnixProxy{
			listener,
			factory
		};

		Ok(proxy)
	}
 }