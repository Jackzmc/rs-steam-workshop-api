//! # steamwebapi
//!
//! This library provides access to the steam web apis. Uses reqwest::blocking under the hood
//! # Getting Started
//! To access any web api that requires no authentication (file details) you need to create a new instance:
//! ```rust
//! use steamwebapi::Workshop;
//! 
//! //Either pass in a Some(reqwest::blocking::Client) or leave None for it to be autocreated
//! let wsclient = Workshop::new(None);
//! wsclient.get_file_details(&["fileid1"]);
//! ```
//! 
//! # Using Authorized Methods 
//! 
//! Authorized methods are behind the AuthedWorkshop struct, which can be generated from a Workshop instance:
//! ```rust
//! use steamwebapi::{Workshop, AuthedWorkshop};
//! 
//! let wsclient = Workshop::new(None);
//! let authed = wsclient.login("MY_API_KEY");
//! authed.search_ids(...);
//! ```

use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf, path::Path, collections::HashMap, fmt};
use reqwest::blocking::Client;

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkshopItem {
    pub publishedfileid: String,
    result: i8,
    creator: String,
    creator_app_id: u32,
    consumer_app_id: u32,
    pub filename: String,
    pub file_size: usize,
    pub file_url: String,
    preview_url: String,
    hcontent_preview: String,
    pub title: String,
    pub description: String,
    pub time_created: usize,
    pub time_updated: usize,
    subscriptions: u32,
    favorited: u32,
    views: u32,
    tags: Vec<WorkshopItemTag>
}

impl fmt::Display for WorkshopItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.title, self.publishedfileid)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkshopItemTag {
    tag: String
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSResponse<T> {
    response: WSResponseBody<T>
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSResponseBody<T> {
    publishedfiledetails: Vec<T>
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSSearchBody {
    result: u8,
    publishedfileid: String,
    language: u8
}

#[derive(Serialize, Deserialize)]
pub struct DownloadEntry {
    pub title: String,
    pub publishedfileid: String,
    pub time_updated: usize
}

pub struct Workshop {
    client: Client,
    apikey: Option<String>,
}

pub struct AuthedWorkshop {
    apikey: String,
    client: Client,
}


#[allow(dead_code)]
impl Workshop {
    ///Creates a new workshop instance
    pub fn new(client: Option<Client>) -> Workshop {
        let client = match client {
            Some(client) => client,
            None => reqwest::blocking::Client::new()
        };
        Workshop {
            client,
            apikey: None,
        }
    }

    ///Gets an authorized workshop, allows access to methods that require api keys. Get api keys from https://steamcommunity.com/dev/apikey
    pub fn login(&mut self, apikey: String) -> AuthedWorkshop {
        AuthedWorkshop {
            apikey: apikey,
            client: self.client.clone()
        }
    }

    /// Gets all *.vpk files in a directory
    pub fn get_vpks_in_folder(dir: &Path) -> Result<Vec<String>, String> {
        let mut entries: Vec<PathBuf> = match fs::read_dir(dir) {
            Ok(file) => {
                match file.map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, io::Error>>() {
                    Ok(files) => files,
                    Err(err) => return Err(err.to_string())
                }
            },
            Err(err) => return Err(err.to_string())
        };
    
        // The order in which `read_dir` returns entries is not guaranteed. If reproducible
        // ordering is required the entries should be explicitly sorted.
    
        entries.sort();
    
        let mut vpks: Vec<String> = Vec::new();
    
        for entry in entries {
            if !entry.is_dir() {
                if let Some("vpk") = entry.extension().and_then(std::ffi::OsStr::to_str) {
                    vpks.push(entry.file_stem().unwrap().to_str().unwrap().to_owned())
                }
            }
        }
        
        Ok(vpks)
    }

    /// Fetches the latest WorkshopItem per each addon id
    pub fn get_file_details(&self, fileids: &[String]) -> Result<Vec<WorkshopItem>, Box<dyn std::error::Error>> {
        let mut params = HashMap::new();
        let length = fileids.len().to_string();
        params.insert("itemcount".to_string(), length);
        for (i, vpk) in fileids.iter().enumerate() {
            let name = format!("publishedfileids[{i}]", i=i);
            params.insert(name, vpk.to_string());
        }
        let details: WSResponse<WorkshopItem> = self.client
            .post("https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/")
            .header("User-Agent", format!("L4D2-Workshop-Downloader/v{}", env!("CARGO_PKG_VERSION")))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()?
            .json::<WSResponse<WorkshopItem>>()?;
    
        let mut details_final: Vec<WorkshopItem> = Vec::new();
    
        for detail in details.response.publishedfiledetails {
            details_final.push(detail);
        }
    
        Ok(details_final)
    }

    //TODO: Extract into builder
    ///Search for workshop items, returns only fileids
    ///Does not require api key by using https://jackz.me/scripts/workshop.php?mode=search
    pub fn search_ids(&self, appid: u64, query: &str) -> Result<Vec<String>, reqwest::Error> {
        if let None = &self.apikey {
            panic!("No Steam Web API key was specified");
        }

        let details = &self.client.get("https://jackz.me/scripts/workshop.php?mode=search")
            .header("User-Agent", format!("L4D2-Workshop-Downloader/v{}", env!("CARGO_PKG_VERSION")))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", "20"),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("v", &env!("CARGO_PKG_VERSION")),
            ])
            .send()?
            .json::<WSResponse<WSSearchBody>>()?;

        let mut fileids: Vec<String> = Vec::new();

        for res in &details.response.publishedfiledetails {
            fileids.push(res.publishedfileid.to_string());
        }
        Ok(fileids)
    }

    ///Searches for workshop items, returns full metadata
    ///Does not require api key by using https://jackz.me/scripts/workshop.php?mode=search
    pub fn search_full(&self, appid: u64, query: &str) -> Result<Vec<WorkshopItem>, reqwest::Error> {
        if let None = &self.apikey {
            panic!("No Steam Web API key was specified");
        }

        let details = &self.client.get("https://jackz.me/scripts/workshop.php?mode=search")
            .header("User-Agent", format!("L4D2-Workshop-Downloader/v{}", env!("CARGO_PKG_VERSION")))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", "20"),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("return_metadata", "1"),
                ("key", &self.apikey.as_ref().unwrap()),
            ])
            .send()?
            .json::<WSResponse<WorkshopItem>>()?;

        Ok(details.response.publishedfiledetails.clone())
    }
}

impl AuthedWorkshop {
    ///Search for workshop items, returns only fileids
    pub fn search_ids(&self, appid: u64, query: &str) -> Result<Vec<String>, reqwest::Error> {
        let details = self.client.get("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?")
            .header("User-Agent", format!("L4D2-Workshop-Downloader/v{}", env!("CARGO_PKG_VERSION")))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", "20"),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("key", &self.apikey),
            ])
            .send()?
            .json::<WSResponse<WSSearchBody>>()?;

        let mut fileids: Vec<String> = Vec::new();

        for res in &details.response.publishedfiledetails {
            fileids.push(res.publishedfileid.to_string());
        }
        Ok(fileids)
    }

    ///Searches for workshop items, returns full metadata
    pub fn search_full(&self, appid: u64, query: &str) -> Result<Vec<WorkshopItem>, reqwest::Error> {
        let details = self.client.get("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?")
            .header("User-Agent", format!("L4D2-Workshop-Downloader/v{}", env!("CARGO_PKG_VERSION")))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", "20"),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("return_metadata", "1"),
                ("key", &self.apikey),
            ])
            .send()?
            .json::<WSResponse<WorkshopItem>>()?;

        Ok(details.response.publishedfiledetails.clone())
    }
}
