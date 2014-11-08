#![feature(slicing_syntax)]

extern crate scgi;
extern crate url;

use scgi::{SCGIEnv, SCGIBind, TcpSCGIServer};

fn process(writer: &mut Writer, env: &SCGIEnv) {
    writer.write_str("Status: 200 OK\r\n");
    writer.write_str("Content-Type: text/plain\r\n");
    writer.write_str("\r\n");
    writer.write_str("Headers:\n");
    for (k, v) in env.env.iter() {
        writer.write_str(k[]);
        writer.write_str(" = ");
        writer.write_str(v[]);
        writer.write_str("\n");
    }

    writer.write_str("\n");
    writer.write_str("Query string:\n");
    for (k, v) in env.query().unwrap().iter() {
        writer.write_str(k[]);
        writer.write_str(" = ");
        writer.write_str(v[]);
        writer.write_str("\n");
    }
}


fn main() {
    let mut server : TcpSCGIServer = SCGIBind::new("127.0.0.1:9000").unwrap();
    server.run(process);
}
