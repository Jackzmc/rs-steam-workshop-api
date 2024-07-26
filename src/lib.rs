//! # steam_workshop_api
//!
//! This library provides access to the steam web apis. Uses reqwest::blocking under the hood
//! # Getting Started
//! To access any web api that requires no authentication (file details) you need to create a new instance:
//! ```rust
//! use steam_workshop_api::SteamWorkshop;
//! 
//! //Either pass in a Some(reqwest::blocking::Client) or leave None for it to be autocreated
//! let wsclient = SteamWorkshop::new(None);
//! wsclient.get_published_file_details(&["fileid1".to_string()]);
//! ```
//! 
//! # Using Authorized Methods 
//! 
//! Authorized methods are behind the AuthedWorkshop struct, which can be generated from a Workshop instance:
//! ```rust
//! use steam_workshop_api::{SteamWorkshop, AuthedWorkshop};
//! 
//! let wsclient = SteamWorkshop::new(None);
//! let authed = wsclient.login("MY_API_KEY");
//! authed.search_ids(...);
//! ```
//! # Using Proxied Methods 
//! 
//! Proxied methods are identical to AuthedWorkshop, except can use a third party server to proxy (and keep the appkey private)
//! ```rust
//! use steam_workshop_api::{SteamWorkshop, ProxyWorkshop};
//! 
//! let wsclient = SteamWorkshop::new(None);
//! let proxy = wsclient.proxy("https://jackz.me/l4d2/scripts/search_public.php");
//! proxy.search_ids(...);
//! ```

use lazy_static::lazy_static;

lazy_static! {
    static ref USER_AGENT: String = format!("{}/v{}", "rs-steamwebapi", env!("CARGO_PKG_VERSION"));
}

use serde::{Deserialize, Serialize};
use std::{fs, path::Path, collections::HashMap, fmt};
use std::fmt::{Debug, Display, Formatter};
use std::fs::DirEntry;
use reqwest::blocking::Client;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct WorkshopItem {
    pub result: i8,
    pub publishedfileid: String,
    pub creator: String,
    #[serde(alias = "creator_appid")]
    pub creator_app_id: u32,
    #[serde(alias = "consumer_appid")]
    pub consumer_app_id: u32,
    pub filename: String,
    pub file_size: u64,
    pub file_url: String,
    pub preview_url: String,
    pub hcontent_preview: String,
    pub title: String,
    #[serde(alias = "file_description")]
    pub description: String,
    pub time_created: usize,
    pub time_updated: usize,
    pub subscriptions: u32,
    pub favorited: u32,
    pub views: u32,
    pub tags: Vec<WorkshopItemTag>
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ItemResponse {
    pub result: i8,
    pub publishedfileid: String,
}

impl fmt::Display for WorkshopItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.title, self.publishedfileid)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct WorkshopItemTag {
    tag: String
}

// WORKSHOP ITEMS:
#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSItemResponse<T> {
    response: WSItemResponseBody<T>
}
#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSItemResponseBody<T> {
    publishedfiledetails: Vec<T>
}
#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSSearchIdBody {
    result: u8,
    publishedfileid: String,
}

// SEARCH ITEMs
#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSSearchResponse<T> {
    response: Option<WSItemResponseBody<T>>,
    total: u8
}


// WORKSHOP COLLECTIONS:
#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSCollectionResponse {
    response: WSCollectionResponseBody
}
#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSCollectionResponseBody {
    result: u8,
    resultcount: u8,
    collectiondetails: Vec<WSCollectionBody>
}
#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSCollectionBody {
    publishedfileid: String,
    result: u8,
    children: Vec<WSCollectionChildren>
}
#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSCollectionChildren {
    publishedfileid: String,
    sortorder: u8,
    filetype: u8
}
#[derive(Clone)]
pub struct SteamWorkshop {
    client: Client,
    apikey: Option<String>,
    request_domain: String
}

pub enum Error {
    /// Request requires authorization either via an apikey, or using a domain proxy that uses their own key
    NotAuthorized,
    RequestError(reqwest::Error)
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAuthorized => write!(f, "Request is not authorized, please use .set_apikey, or .set_proxy_domain"),
            Error::RequestError(e) => write!(f, "request error: {}", e)
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAuthorized => write!(f, "Not authorized"),
            Error::RequestError(e) => write!(f, "Request Error: {}", e)
        }
    }
}

impl std::error::Error for Error {}

#[allow(dead_code)]
impl SteamWorkshop {
    ///Creates a new workshop instance, client will be auto created if None
    pub fn new() -> SteamWorkshop {
        let client= Client::new();
        SteamWorkshop::new_with_client(client)
    }
    pub fn new_with_client(client: Client) -> SteamWorkshop {
        SteamWorkshop {
            client,
            request_domain: "api.steampowered.com".to_string(),
            apikey: None
        }
    }

    ///Gets an authorized workshop, allows access to methods that require api keys. 
    ///Get api keys from https://steamcommunity.com/dev/apikey
    pub fn set_apikey(&mut self, apikey: Option<String>) {
        self.apikey = apikey;
    }

    /// Will change the domain that requests are made to, allowing you to proxy api.steampowered.com
    pub fn set_proxy_domain(&mut self, proxy_domain: Option<String>) {
        self.request_domain = proxy_domain.unwrap_or("api.steampowered.com".to_string());
    }

    /// Returns DirEntry for all *.vpk files in a directory.
    pub fn get_vpks_in_folder(dir: &Path) -> Result<Vec<DirEntry>, String> {
        let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
        let mut files: Vec<DirEntry> = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if file_name.ends_with(".vpk") {
                files.push(entry)
            }
        }
        return Ok(files);
    }

    /// Fetches the latest WorkshopItem per each addon id
    /// Steam API only allows 100 entries at once, will have an api error if more given
    pub fn get_published_file_details(&self, fileids: &[String]) -> Result<Vec<WorkshopItem>, Error> {
        let mut params = HashMap::new();
        let length = fileids.len().to_string();
        params.insert("itemcount".to_string(), length);
        for (i, vpk) in fileids.iter().enumerate() {
            if !vpk.parse::<u64>().is_ok() {
                panic!("Item is not valid publishedfileid: {}", vpk);
            }
            let name = format!("publishedfileids[{i}]", i=i);
            params.insert(name, vpk.to_string());
        }
        let mut details = self.client
            .post(format!("https://{}/ISteamRemoteStorage/GetPublishedFileDetails/v1/", self.request_domain))
            .header("User-Agent", &USER_AGENT.to_string())
            .form(&params)
            .send().map_err(|e| Error::RequestError(e))?
            .error_for_status().map_err(|e| Error::RequestError(e))?
            .json::<Value>().map_err(|e| Error::RequestError(e))?;

        Ok(details["response"]["publishedfiledetails"].as_array_mut().unwrap().iter_mut()
            .filter(|v| v["result"] == 1)
            .map(|v| serde_json::from_value(v.take()).unwrap())
            .collect()
        )

        // let mut details_final: Vec<WorkshopItem> = Vec::new();
        //
        // for detail in details["response"]["publishedfiledetails"] {
        //     if detail.result == 1 {
        //         details_final.push(serde_json::from_value(detail).unwrap());
        //     }
        // }
        //
        // Ok(details_final)
    }

    /// Gets the collection details (all the children of this item).
    /// Returns a list of children fileids which can be sent directly to get_published_file_details()
    /// Will return Ok(None) if the item is not a collection.
    pub fn get_collection_details(&self, fileid: &str) -> Result<Option<Vec<String>>, Error> {
        let mut params = HashMap::new();
        params.insert("collectioncount", "1");
        params.insert("publishedfileids[0]", &fileid);
        let details: WSCollectionResponse = self.client
            .post(format!("https://{}/ISteamRemoteStorage/GetCollectionDetails/v1/", self.request_domain))
            .header("User-Agent", USER_AGENT.to_string())
            .form(&params)
            .send().map_err(|e| Error::RequestError(e))?
            .error_for_status().map_err(|e| Error::RequestError(e))?
            .json::<WSCollectionResponse>().map_err(|e| Error::RequestError(e))?;
           
        if details.response.resultcount > 0 { 
            let mut ids: Vec<String>  = Vec::new();
            for children in &details.response.collectiondetails[0].children {
                ids.push(children.publishedfileid.to_string());
            }
            Ok(Some(ids))
        } else {
            Ok(None)
        }
    }

    /// Searches for workshop items, returns their file ids.
    /// REQUIRES steam apikey or a proxy domain
    pub fn search_items(&self, appid: u64, query: &str, count: usize) -> Result<Vec<WorkshopItem>, Error> {
        if self.apikey.is_none() || self.request_domain != "api.steampowered.com" {
            return Err(Error::NotAuthorized)
        }
        let apikey: &str = self.apikey.as_deref().unwrap_or("");
        let details = self.client.get(format!("https://{}/IPublishedFileService/QueryFiles/v1/?", self.request_domain))
            .header("User-Agent", USER_AGENT.to_string())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", &count.to_string()),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("return_metadata", "1"),
                ("key", apikey),
            ])
            .send().map_err(|e| Error::RequestError(e))?
            .json::<WSSearchResponse<WorkshopItem>>().map_err(|e| Error::RequestError(e))?;

        if details.total > 0 {
            Ok(details.response.unwrap().publishedfiledetails)
        } else {
            Ok(vec!())
        }
    }

    /// Check if the user (of apikey) can subscribe to the published file
    /// REQUIRES apikey, cannot use proxy.
    pub fn can_subscribe(&self, fileid: &str) -> Result<bool, Error> {
        if self.apikey.is_none() {
            return Err(Error::NotAuthorized)
        }

        let details: Value = self.client
            .get("https://api.steampowered.com/IPublishedFileService/CanSubscribe/v1/?key=7250BBE4BC2ECA0E16197B38E3675988&publishedfileid=122447941")
            .header("User-Agent", USER_AGENT.to_string())
            .query(&[
                "key", &self.apikey.as_ref().unwrap(),
                "publishedfileid", fileid
            ])
            .send().map_err(|e| Error::RequestError(e))?
            .error_for_status().map_err(|e| Error::RequestError(e))?
            .json().map_err(|e| Error::RequestError(e))?;
        Ok(details["response"]["can_subscribe"].as_bool().unwrap_or(false))
    }

}
