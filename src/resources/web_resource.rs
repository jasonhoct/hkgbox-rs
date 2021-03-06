use std::io::Read;
use ::hyper::Client;

use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use ::hyper::header::{Headers, UserAgent};

pub struct WebResource {
     pub pages: HashMap<String, String>,
     client: Client
}

impl WebResource {

    pub fn new() -> Self {
        WebResource {
            pages: HashMap::new(),
            client: Client::new(),
        }
    }

    // let fetch_page = move |url: &str| -> String {
    //     download_map.entry(String::from(url))
    //                 .or_insert_with(move || {
    //                     match download_page(&client, &String::from(url)) {
    //                         Ok(s) => s,
    //                         Err(e) => format!("{:?}", e),
    //                     }
    //                 })
    //                 .clone()
    // };
    //

    pub fn fetch(&mut self, url: &String) -> Result<String, Error> {
        info!("web resource #fetch");
        let mut headers = Headers::new();
        headers.set(UserAgent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/56.0.2924.87 Safari/537.36".to_owned()));
        let mut client = Client::new();
        client.set_read_timeout(Some(::std::time::Duration::from_secs(5)));
        client.set_write_timeout(Some(::std::time::Duration::from_secs(5)));
        match client.get(url).headers(headers).send() {
            Ok(mut resp) => {
                let mut s = String::new();
                match resp.read_to_string(&mut s) {
                    Ok(size) => Ok(s),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }

    pub fn fetch_safe(&mut self, url: &str) -> String {
        match self.fetch(&String::from(url)) {
            Ok(s) => s,
            Err(e) => format!("{:?}", e),
        }
    }

    pub fn find(&mut self, url: &str) -> String {
        match self.pages.get(url) {
            Some(page) => page.clone(),
            None => { String::from("None") }
        }
    }

    pub fn get(&mut self, url: &str) -> String {
        if !self.pages.contains_key(url) {
            let res = self.fetch_safe(url);
            self.pages.insert(String::from(url), res);
        }
        self.find(url)
    }
}
