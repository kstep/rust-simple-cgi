#![feature(slicing_syntax)]

extern crate scgi;
extern crate url;

#[allow(unused_imports)]
use scgi::{SCGIEnv, SCGIBind, TcpSCGIServer, UnixSCGIServer};
use std::io::IoResult;

fn process(writer: &mut Writer, env: &SCGIEnv) -> IoResult<()> {
    try!(writer.write_str("Status: 200 OK\r\n"));
    try!(writer.write_str("Content-Type: text/plain\r\n"));
    try!(writer.write_str("\r\n"));
    try!(writer.write_str("Headers:\n"));
    for (k, v) in env.env.iter() {
        try!(writer.write_str(k[]));
        try!(writer.write_str(" = "));
        try!(writer.write_str(v[]));
        try!(writer.write_str("\n"));
    }

    try!(writer.write_str("\n"));
    try!(writer.write_str("Query string:\n"));
    for (k, v) in env.query().unwrap().iter() {
        try!(writer.write_str(k[]));
        try!(writer.write_str(" = "));
        try!(writer.write_str(v[]));
        try!(writer.write_str("\n"));
    }

    Ok(())
}


fn main() {
    let server : TcpSCGIServer = SCGIBind::new("localhost:9000").unwrap();
    //let server : UnixSCGIServer = SCGIBind::new("/tmp/rust-scgi-server").unwrap();
    server.run(process);
}
