#![feature(slicing_syntax)]

extern crate url;

use std::collections::BTreeMap;
use std::io::{IoResult, BytesReader, standard_error, InvalidInput, Acceptor, Listener, Stream};
use std::io::net::tcp::{TcpListener, TcpStream, TcpAcceptor};
use std::io::net::pipe::{UnixListener, UnixStream, UnixAcceptor};
use std::path::Path;
use std::thread::Thread;
use url::form_urlencoded;
use url::Url;

#[derive(Show)]
pub struct SCGIEnv {
    pub env: BTreeMap<String, String>
}

pub struct MapResultIter<'a, A, B, E, I: 'a + Iterator<Item=Result<A, E>>, F: Fn(&'a A) -> B> {
    inner: &'a mut I,
    f: F
}

impl<'a, A: 'a, B, E, I: Iterator<Item=Result<A, E>>, F: Fn(&A) -> B> Iterator for MapResultIter<'a, A, B, E, I, F> {
    type Item = Result<B, E>;
    fn next(&mut self) -> Option<Result<B, E>> {
        match self.inner.next() {
            Some(Ok(ref v)) => Some(Ok((self.f)(v))),
            Some(Err(e)) => Some(Err(e)),
            None => None
        }
    }
}

trait MapResultExt<'a, A, E> {
    fn result_map<B, F: Fn(&'a A) -> B>(&'a mut self, f: F) -> MapResultIter<'a, A, B, E, Self, F>;
}

impl<'a, A, E, I: Iterator<Item=Result<A, E>> + 'a> MapResultExt<'a, A, E> for I {
    fn result_map<B, F: Fn(&'a A) -> B>(&'a mut self, f: F) -> MapResultIter<'a, A, B, E, Self, F> {
        MapResultIter {
            inner: self,
            f: f
        }
    }
}



impl SCGIEnv {
    pub fn from_reader<T: Reader>(input: &mut T) -> IoResult<SCGIEnv> {
        let length = try!(input.bytes().take_while(|c| match *c { Ok(b) => b != 0x3a, Err(_) => false }).fold(Ok(0u),
            |a, c| match (a, c) {
                (Ok(s), Ok(b)) => Ok((b as uint & 0x0f) + s * 10),
                (_, Err(e)) | (Err(e), _) => Err(e)
            }));

        let headers = try!(input.read_exact(length));
        let mut iter = headers.split(|b| *b == 0x0);
        let mut map = BTreeMap::new();

        while let (Some(name), Some(value)) = (iter.next(), iter.next()) {
            map.insert(String::from_utf8_lossy(name).to_string(), String::from_utf8_lossy(value).to_string());
        }

        match input.read_byte() {
            Ok(0x2c) => Ok(SCGIEnv{ env: map }),
            Err(e) => Err(e),
            _ => Err(standard_error(InvalidInput))
        }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        match self.env.get(name) {
            Some(value) => Some(value.to_string()),
            None => None
        }
    }

    pub fn method(&self) -> String {
        self.env.get("REQUEST_METHOD").unwrap().to_string()
    }

    pub fn query(&self) -> Option<BTreeMap<String, String>> {
        match self.get("QUERY_STRING") {
            Some(s) => Some(form_urlencoded::parse(s.as_bytes()).into_iter().collect::<BTreeMap<String, String>>()),
            None => None
        }
    }

    pub fn query_vec(&self) -> Option<Vec<(String, String)>> {
        match self.get("QUERY_STRING") {
            Some(s) => Some(form_urlencoded::parse(s.as_bytes())),
            None => None
        }
    }

    pub fn content_length(&self) -> uint {
        self.get("CONTENT_LENGTH").and_then(|v| v.parse()).unwrap_or(0u)
    }

    pub fn port(&self, name: &str) -> Option<u16> {
        self.get(name).and_then(|v| v.parse())
    }

    pub fn path(&self, name: &str) -> Option<Path> {
        self.get(name).and_then(|v| Path::new_opt(v))
    }

    pub fn url(&self, name: &str) -> Option<Url> {
        self.get(name).and_then(|v| Url::parse(v[]).ok())
    }

    pub fn cookies(&self) -> Option<BTreeMap<String, String>> {
        self.get("HTTP_COOKIE").map(|v| v.split(';').map(|c| c.splitn(1, '=')).map(|mut s| (s.next().unwrap().trim_left_matches(' ').to_string(), s.next().unwrap().to_string())).collect::<BTreeMap<String, String>>())
    }
}

pub struct SCGIServer<L, S, A> where A: Acceptor<S>, L: Listener<S, A>, S: Stream {
    listener: L
}

impl<L, S, A> SCGIServer<L, S, A> where A: Acceptor<S>, L: Listener<S, A>, S: Stream + Send {
    pub fn new(listener: L) -> SCGIServer<L, S, A> {
        SCGIServer { listener: listener }
    }

    pub fn run(self, process: fn(&mut Stream, &SCGIEnv) -> IoResult<()>) {
        let mut server = self.listener.listen().unwrap();

        for conn in server.incoming() {
            Thread::spawn(move || {
                let mut stream = conn.unwrap();
                let headers = SCGIEnv::from_reader(&mut stream).unwrap();
                process(&mut stream, &headers).unwrap();
            }).detach()
        }
    }
}

pub type TcpSCGIServer = SCGIServer<TcpListener, TcpStream, TcpAcceptor>;
pub type UnixSCGIServer = SCGIServer<UnixListener, UnixStream, UnixAcceptor>;

#[cfg(test)]
mod tests {
    use std::io::MemReader;
    use std::vec::as_vec;

    #[test]
    fn test_read_header() {
        let mut scgi_data = Vec::new();
        for b in b"70:CONTENT_LENGTH\027\0SCGI\01\0REQUEST_METHOD\0POST\0REQUEST_URI\0/deepthought\0,What is the answer to life?".iter() {
            scgi_data.push(*b);
        }

        let mut reader = MemReader::new(scgi_data);
        let headers = read_scgi_headers(&mut reader).unwrap();

        let mut expected = BTreeMap::new();
        expected.insert("CONTENT_LENGTH".to_string(), "27".to_string());
        expected.insert("SCGI".to_string(), "1".to_string());
        expected.insert("REQUEST_METHOD".to_string(), "POST".to_string());
        expected.insert("REQUEST_URI".to_string(), "/deepthought".to_string());

        assert_eq!(headers, expected);

        let body = reader.read_exact(headers["CONTENT_LENGTH".to_string()].parse().unwrap()).unwrap();
        assert_eq!(body[], b"What is the answer to life?");
    }
}
