#![feature(slicing_syntax)]

extern crate scgi;
extern crate url;

#[allow(unused_imports)]
use scgi::{SCGIEnv, SCGIBind, TcpSCGIServer, UnixSCGIServer};
use std::io::IoResult;

fn process(w: &mut Writer, env: &SCGIEnv) -> IoResult<()> {
    try!(w.write_str("Status: 200 OK\r\n"));
    try!(w.write_str("Content-Type: text/html\r\n"));
    try!(w.write_str("\r\n"));

    try!(w.write_str("<!DOCTYPE html5><html><body>\n"));
    try!(w.write_str("<h1>Headers</h1>\n"));
    try!(w.write_str("<table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody>"));
    for (k, v) in env.env.iter() {
        try!(w.write_str("<tr><td>"));
        try!(w.write_str(k[]));
        try!(w.write_str("</td><td>"));
        try!(w.write_str(v[]));
        try!(w.write_str("</td></tr>"));
    }
    try!(w.write_str("</tbody></table>"));

    try!(w.write_str("<h1>Query string</h1>"));
    try!(w.write_str("<table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody>"));
    for (k, v) in env.query().unwrap().iter() {
        try!(w.write_str("<tr><td>"));
        try!(w.write_str(k[]));
        try!(w.write_str("</td><td>"));
        try!(w.write_str(v[]));
        try!(w.write_str("</td></tr>"));
    }
    try!(w.write_str("</tbody></table>"));

    try!(w.write_str("</pre>"));
    try!(w.write_str("</body></html>"));

    Ok(())
}


fn main() {
    let server : TcpSCGIServer = SCGIBind::new("localhost:9000").unwrap();
    //let server : UnixSCGIServer = SCGIBind::new("/tmp/rust-scgi-server").unwrap();
    server.run(process);
}
