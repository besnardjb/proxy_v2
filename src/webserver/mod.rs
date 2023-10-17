use rouille::{Response, Request};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use super::exporter::Exporter;


/*************
 * WEBSERVER *
 *************/

pub(crate) struct Web
{
	port : u32,
	exporter : Arc<Exporter>
}

#[derive(Serialize)]
struct ApiResponse
{
	operation : String,
	success : bool
}


enum WebResponse {
	HTML(String),
	Text(String),
	BadReq(String),
	Success(String),
	NoSuchDoc()
}

impl WebResponse
{
	fn serialize(self : WebResponse) -> Response
	{
		match self
		{
			WebResponse::HTML(s) => 
			{
				Response::html(s)
			}
			WebResponse::Text(s) => 
			{
				Response::text(s)
			}
			WebResponse::BadReq(operation) => 
			{
				let r = ApiResponse{
					operation,
					success : false
				};
				Response::json(&r).with_status_code(400)
			}
			WebResponse::Success(operation) =>
			{
				let r = ApiResponse{
					operation,
					success : true
				};
				Response::json(&r)
			}
			WebResponse::NoSuchDoc() => 
			{
				Response::empty_404()
			}
		}
	}
}



impl Web
{
	pub(crate) fn new(port : u32, exporter : Arc<Exporter>) -> Web
	{
		Web 
		{
			port: port,
			exporter : exporter
		}
	}

	fn default_doc() -> String {
		"".to_string()
  }

	fn parse_key_value(req : &Request) -> (Option<String>, Option<f64>, String, Option<String>)
	{
		let key : Option<String>;
		let svalue : Option<String>;
		let parsed_value : Option<f64>;
		let error : Option<String>;
		let doc : String; // Optionnal
		#[derive(Deserialize)]
		struct KeyValue
		{
			key : String,
			value : String,
			#[serde(default = "Web::default_doc")]
			doc : String
		}

		match req.method()
		{
			"GET" =>
			{
				key = req.get_param("key");
				svalue = req.get_param("value");
				match req.get_param("doc")
				{
					Some(e) => {doc = e;},
					None => { doc = Web::default_doc();}
				}
				error = None;
			}
			"POST" =>
			{
				let kv : Result<KeyValue, rouille::input::json::JsonError> = rouille::input::json_input(req);
				match kv
				{
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
			_ =>
			{
				key = None;
				svalue = None;
				doc = Web::default_doc();
				error = Some(format!("No such request type {}", req.method()));
			}
		}

		if svalue.is_some()
		{
			match svalue.unwrap().parse::<f64>() {
				Ok(val) => {
					parsed_value = Some(val);
				}
				Err(e) => {
					return (key, None, doc, Some(e.to_string()));
				}
			}
		}
		else
		{
			parsed_value = None;
		}

		(key, parsed_value, doc, error)
	}

	fn handle_set(&self, req : &Request) -> WebResponse
	{
		let key : Option<String>;
		let error : Option<String>;
		let value : Option<f64>;
		let _doc : String;

		(key, value, _doc, error) = Web::parse_key_value(req);

		if error.is_some()
		{
			return WebResponse::BadReq(format!("Error parsing parameters: {}", error.unwrap()));
		}

		let key = key.unwrap();
		let value = value.unwrap();

		match self.exporter.set(key.as_str(), value)
		{
			Ok(_) => {
				WebResponse::Success("set".to_string())
			}
			Err(e) => {
				return WebResponse::BadReq(e.to_string());
			}
		}

	}

	fn handle_accumulate(&self, req : &Request) -> WebResponse
	{
		let key : Option<String>;
		let error : Option<String>;
		let value : Option<f64>;
		let _doc : String;

		(key, value, _doc, error) = Web::parse_key_value(req);

		if error.is_some()
		{
			return WebResponse::BadReq(format!("Error parsing parameters: {}", error.unwrap()));
		}

		let key = key.unwrap();
		let value = value.unwrap();

		match self.exporter.accumulate(key.as_str(), value)
		{
			Ok(_) => {
				WebResponse::Success("inc".to_string())
			}
			Err(e) => {
				return WebResponse::BadReq(e.to_string());
			}
		}
	}

	fn handle_push(&self, req : &Request) -> WebResponse
	{
		let key : Option<String>;
		let error : Option<String>;
		let doc : String;

		(key, _, doc, error) = Web::parse_key_value(req);

		if error.is_some()
		{
			return WebResponse::BadReq(format!("Error parsing parameters: {}", error.unwrap()));
		}

		let key = key.unwrap();

		match self.exporter.push(key.as_str(), doc.as_str())
		{
			Ok(_) => {
				WebResponse::Success("push".to_string())
			}
			Err(e) => {
				return WebResponse::BadReq(e.to_string());
			}
		}
	}

	fn handle_metrics(&self, _req : &Request) -> WebResponse
	{
		match self.exporter.serialize()
		{
			Ok(v) => {
				WebResponse::Text(v)
			}
			Err(e) => {
				return WebResponse::BadReq(e.to_string());
			}
		}
	}

	pub(crate) fn run_blocking(self)
	{
		rouille::start_server(format!("0.0.0.0:{}",self.port), move | request| {

			let resp : WebResponse;

			match request.url().as_str()
			{
				"/" => {
					resp = WebResponse::HTML("<html><h1>Metric Proxy</h1></html>".to_string());
				}
				"/set" => {
					resp = self.handle_set(&request);
				},
				"/accumulate" => {
					resp = self.handle_accumulate(&request);
				},
				"/push" => {
					resp = self.handle_push(&request);
				},
				"/metrics" => {
					resp = self.handle_metrics(&request);
				},
				_ => {
					resp = WebResponse::NoSuchDoc();
				}
			}

			resp.serialize()
	  });
	}
}