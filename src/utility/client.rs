extern crate hyper;

use std::io::Read;
use self::hyper::Client;
use self::hyper::header::Connection;

use std;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::cell::Cell;
use std::sync::Arc;
use std::thread;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

#[derive(Clone)]
pub enum ChannelItemType {
    Show(ChannelShowItem),
    Index(ChannelIndexItem)
}

#[derive(Clone)]
pub struct ChannelIndexItem { }

#[derive(Clone)]
pub struct ChannelShowItem {
    pub postid: String,
    pub page: usize,
}

pub struct ChannelItem {
    pub extra: ChannelItemType,
    pub result: String
}

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
        match self.client.get(url).send() {
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
