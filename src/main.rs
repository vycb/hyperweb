#![deny(warnings)]
#![feature(custom_derive)]
#![plugin(tojson_macros)]
#![feature(plugin)]
extern crate hyper;
#[macro_use]
extern crate mime;
extern crate handlebars;
extern crate rustc_serialize;
extern crate formdata;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::collections::BTreeMap;

use handlebars::{Handlebars, RenderError, RenderContext, Helper, Context};
use rustc_serialize::json::{Json, ToJson};
// use std::io::copy;

use hyper::{Get, Post};
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

#[derive(ToJson)]
struct Team {
    name: String,
    pts: u16
}

fn make_data () -> BTreeMap<String, Json> {
    let mut data = BTreeMap::new();

    data.insert("year".to_string(), "2015".to_json());

    let teams = vec![ Team { name: "Jiangsu Sainty".to_string(),
                             pts: 43u16 },
                      Team { name: "Beijing Guoan".to_string(),
                             pts: 27u16 },
                      Team { name: "Guangzhou Evergrand".to_string(),
                             pts: 22u16 },
                      Team { name: "Shandong Luneng".to_string(),
                             pts: 12u16 } ];

    data.insert("teams".to_string(), teams.to_json());
    data
}

macro_rules! try_return(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { println!("Error: {}", e); return; }
        }
    }}
);

fn index(req: & Request, res: & Response) {
    match req.uri {
      AbsolutePath(ref path) => match (&req.method, &path[..]) {
        (&Get, "/") | (&Get, "/echo") => {

          let mut handlebars = Handlebars::new();
          let t = load_template("tpl/template.html").ok().unwrap();
          handlebars.register_template_string("table", t).ok().unwrap();
          handlebars.register_helper("format", Box::new(format_helper));
          let data = make_data();

          try_return!(res.send(handlebars.render("table", &data).ok().unwrap().as_bytes()));
          return;
        },
        (&Post, "/") | (&Post, "/echo") => {
          let boundary = formdata::get_multipart_boundary(&req.headers).unwrap();

          match formdata::parse_multipart(&mut req.body, boundary) {
              Ok(form_data) => {
                  assert_eq!(form_data.fields.len(), 1);
                  for (key, val) in form_data.fields {
                      if &key == "field1" {
                          assert_eq!(&val, "data1");
                      }
                  }

                  assert_eq!(form_data.files.len(), 2);
                  for (key, file) in form_data.files {
                      if &key == "field2" {
                          assert_eq!(file.size, 30);
                          assert_eq!(file.filename.as_ref().unwrap(), "image.gif");
                          assert_eq!(file.content_type, mime!(Image/Gif));
                      } else if &key == "field3" {
                          assert_eq!(file.size, 14);
                          assert_eq!(file.filename.as_ref().unwrap(), "file.txt");
                          assert_eq!(file.content_type, mime!(Text/Plain; Charset=Utf8));
                      }
                  }
              },
              Err(err) => panic!("{}", err),
          }
          return;
        }, // fall through, fighting mutable borrows
        _ => {
            *res.status_mut() = hyper::NotFound;
            return;
        }
      },
      _ => {
          return;
      }
    };
    // let mut res = try_return!(res.start());
    // try_return!(copy(&mut req, &mut res));
}

fn format_helper (c: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let param = h.params().get(0).unwrap();
    let rendered = format!("{} pts", c.navigate(rc.get_path(), param));
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

fn load_template(name: &str) -> io::Result<String> {
    let path = Path::new(name);

    let mut file = try!(File::open(path));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(s)
}

fn main() {
    let server = Server::http("wram:8080").unwrap();
    let _guard = server.handle(index);
    println!("Listening on http://wram:8080");
}
