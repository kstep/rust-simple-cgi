#![feature(slicing_syntax)]

extern crate scgi;
extern crate url;

#[allow(unused_imports)]
use scgi::{SCGIEnv, TcpSCGIServer, UnixSCGIServer, SCGIServer};
use std::io::{IoResult, Stream};

fn process(w: &mut Stream, env: &SCGIEnv) -> IoResult<()> {
    try!(w.write_str("Status: 200 OK\r\n"));
    try!(w.write_str("Content-Type: text/html\r\n"));
    try!(w.write_str("Set-Cookie: rust=rulez\r\n"));
    try!(w.write_str("Set-Cookie: ssid=123\r\n"));
    try!(w.write_str("\r\n"));

    try!(w.write_str("<!DOCTYPE html5><html>"));
    try!(w.write_str("<head>"));
    try!(w.write_str("<title>Rust SCGI test server</title>"));
    try!(w.write_str("</head>"));
    try!(w.write_str("<body>"));
    try!(w.write_str("<h1>Headers</h1>"));
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
    if let Some(query) = env.query() {
        for (k, v) in query.iter() {
            try!(w.write_str("<tr><td>"));
            try!(w.write_str(k[]));
            try!(w.write_str("</td><td>"));
            try!(w.write_str(v[]));
            try!(w.write_str("</td></tr>"));
        }
    }
    try!(w.write_str("</tbody></table>"));

    try!(w.write_str("<h1>Cookies</h1>"));
    try!(w.write_str("<table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody>"));
    if let Some(cookies) = env.cookies() {
        for (k, v) in cookies.iter() {
            try!(w.write_str("<tr><td>"));
            try!(w.write_str(k[]));
            try!(w.write_str("</td><td>"));
            try!(w.write_str(v[]));
            try!(w.write_str("</td></tr>"));
        }
    }

    try!(w.write_str("</tbody></table>"));

    try!(w.write_str("<h1>Form</h1>"));
    try!(w.write_str("<form method=\"POST\"><input type=\"text\" placeholder=\"text input\" name=\"text\" /><br><input type=\"date\" name=\"date\" placeholder=\"date input\" /><br><input type=\"submit\" /></form>"));

    let content_length = env.content_length();
    if content_length > 0 {
        try!(w.write_str("<hr><pre>"));
        let data = try!(w.read_exact(content_length));
        try!(w.write(data[]));
        try!(w.write_str("</pre>"));
    }

    try!(w.write_str("</body></html>"));

    Ok(())
}


fn main() {
    let server = TcpSCGIServer::new("localhost:9000").unwrap();
    //let server = UnixSCGIServer::new("/tmp/rust-scgi-server").unwrap();
    server.run(process);
}
