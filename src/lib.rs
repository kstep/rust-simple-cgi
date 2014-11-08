#![feature(while_let, slicing_syntax)]

extern crate url;

use std::collections::TreeMap;
use std::io::{IoResult, BytesReader, standard_error, InvalidInput};
use url::form_urlencoded::parse_str;

#[cfg(test)] use std::io::MemReader;
#[cfg(test)] use std::vec::as_vec;

#[deriving(Show)]
pub struct SCGIEnv {
    pub env: TreeMap<String, String>
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
        let mut map = TreeMap::new();

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
        match self.env.get(&name.to_string()) {
            Some(value) => Some(value.to_string()),
            None => None
        }
    }

    pub fn query(&self) -> Option<TreeMap<String, String>> {
        match self.get("QUERY_STRING") {
            Some(s) => Some(parse_str(s[]).into_iter().collect::<TreeMap<String, String>>()),
            None => None
        }
    }

    pub fn query_vec(&self) -> Option<Vec<(String, String)>> {
        match self.get("QUERY_STRING") {
            Some(s) => Some(parse_str(s[])),
            None => None
        }
    }

    pub fn content_length(&self) -> uint {
        from_str(self.get("CONTENT_LENGTH").unwrap()[]).unwrap()
    }

    pub fn get_port(&self, name: &str) -> Option<u16> {
        self.get(name).and_then(|v| from_str(v[]))
    }
}

#[test]
fn test_read_header() {
    let mut scgi_data = Vec::new();
    for b in b"70:CONTENT_LENGTH\027\0SCGI\01\0REQUEST_METHOD\0POST\0REQUEST_URI\0/deepthought\0,What is the answer to life?".iter() {
        scgi_data.push(*b);
    }

    let mut reader = MemReader::new(scgi_data);
    let headers = read_scgi_headers(&mut reader).unwrap();

    let mut expected = TreeMap::new();
    expected.insert("CONTENT_LENGTH".to_string(), "27".to_string());
    expected.insert("SCGI".to_string(), "1".to_string());
    expected.insert("REQUEST_METHOD".to_string(), "POST".to_string());
    expected.insert("REQUEST_URI".to_string(), "/deepthought".to_string());

    assert_eq!(headers, expected);

    let body = reader.read_exact(from_str(headers["CONTENT_LENGTH".to_string()][]).unwrap()).unwrap();
    assert_eq!(body[], b"What is the answer to life?");
}
