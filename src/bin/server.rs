#![feature(slicing_syntax)]

extern crate scgi;
extern crate url;

use scgi::SCGIEnv;
use std::io::{TcpListener, Listener, Acceptor};
use std::task::spawn;

fn main() {
    let mut server = TcpListener::bind("127.0.0.1:9000").listen().unwrap();

    for conn in server.incoming() {
        spawn(proc() {
            let mut stream = conn.unwrap();
            let headers = SCGIEnv::from_reader(&mut stream).unwrap();
            println!("headers: {}", headers);
            stream.write_str("Status: 200 OK\r\n");
            stream.write_str("Content-Type: text/plain\r\n");
            stream.write_str("\r\n");
            stream.write_str("Headers:\n");
            for (k, v) in headers.env.iter() {
                stream.write_str(k[]);
                stream.write_str(" = ");
                stream.write_str(v[]);
                stream.write_str("\n");
            }

            stream.write_str("\n");
            stream.write_str("Query string:\n");
            for (k, v) in headers.query().unwrap().iter() {
                stream.write_str(k[]);
                stream.write_str(" = ");
                stream.write_str(v[]);
                stream.write_str("\n");
            }
        })
    }
}
