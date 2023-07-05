//! # steam_workshop_api
//!
//! This library provides access to the steam web apis. Uses reqwest::blocking under the hood
//! # Getting Started
//! To access any web api that requires no authentication (file details) you need to create a new instance:
//! ```rust
//! use steam_workshop_api::Workshop;
//! 
//! //Either pass in a Some(reqwest::blocking::Client) or leave None for it to be autocreated
//! let wsclient = Workshop::new(None);
//! wsclient.get_published_file_details(&["fileid1"]);
//! ```
//! 
//! # Using Authorized Methods 
//! 
//! Authorized methods are behind the AuthedWorkshop struct, which can be generated from a Workshop instance:
//! ```rust
//! use steam_workshop_api::{Workshop, AuthedWorkshop};
//! 
//! let wsclient = Workshop::new(None);
//! let authed = wsclient.login("MY_API_KEY");
//! authed.search_ids(...);
//! ```
//! # Using Proxied Methods 
//! 
//! Proxied methods are identical to AuthedWorkshop, except can use a third party server to proxy (and keep the appkey private)
//! ```rust
//! use steam_workshop_api::{Workshop, ProxyWorkshop};
//! 
//! let wsclient = Workshop::new(None);
//! let proxy = wsclient.proxy("https://jackz.me/l4d2/scripts/search_public.php");
//! proxy.search_ids(...);
//! ```


use lazy_static::lazy_static;

lazy_static! {
    static ref USER_AGENT: String = format!("{}/v{}", "rs-steamwebapi", env!("CARGO_PKG_VERSION"));
}


use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf, path::Path, collections::HashMap, fmt};
use reqwest::blocking::Client;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct WorkshopItem {
    pub result: i8,
    pub publishedfileid: String,
    pub creator: String,
    pub creator_app_id: u32,
    pub consumer_app_id: u32,
    pub filename: String,
    pub file_size: u64,
    pub file_url: String,
    pub preview_url: String,
    pub hcontent_preview: String,
    pub title: String,
    pub description: String,
    pub time_created: usize,
    pub time_updated: usize,
    pub subscriptions: u32,
    pub favorited: u32,
    pub views: u32,
    pub tags: Vec<WorkshopItemTag>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkshopSearchItem {
    pub result: i8,
    pub publishedfileid: String,
    pub creator: String,
    pub creator_appid: u32,
    pub consumer_appid: u32,
    pub filename: String,
    pub file_size: String,
    pub file_url: String,
    pub preview_url: String,
    pub hcontent_preview: String,
    pub title: String,
    pub file_description: String,
    pub time_created: usize,
    pub time_updated: usize,
    pub subscriptions: u32,
    pub favorited: u32,
    pub views: u32,
    pub tags: Vec<WorkshopItemTag>
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
// MISC




impl WorkshopSearchItem {
    /// Converts from a WorkshopSearchItem to a WorkshopItem
    pub fn to_item(&self) -> WorkshopItem {
        WorkshopItem {
            result: self.result.clone(),
            publishedfileid: self.publishedfileid.clone(),
            creator: self.creator.clone(),
            creator_app_id: self.creator_appid.clone(),
            consumer_app_id: self.consumer_appid.clone(),
            filename: self.filename.clone(),
            file_size: self.file_size.parse().unwrap(),
            file_url: self.file_url.clone(),
            preview_url: self.preview_url.clone(),
            hcontent_preview: self.hcontent_preview.clone(),
            title: self.title.clone(),
            description: self.file_description.clone(),
            time_created: self.time_created,
            time_updated: self.time_updated,
            subscriptions: self.subscriptions,
            favorited: self.favorited,
            views: self.views,
            tags: self.tags.clone(),
        }
    }
    /// Converts to a WorkshopSearchItem from a WorkshopItem
    pub fn from_item(item: &WorkshopItem) -> WorkshopSearchItem {
        WorkshopSearchItem {
            result: item.result.clone(),
            publishedfileid: item.publishedfileid.clone(),
            creator: item.creator.clone(),
            creator_appid: item.creator_app_id.clone(),
            consumer_appid: item.consumer_app_id.clone(),
            filename: item.filename.clone(),
            file_size: item.file_size.to_string(),
            file_url: item.file_url.clone(),
            preview_url: item.preview_url.clone(),
            hcontent_preview: item.hcontent_preview.clone(),
            title: item.title.clone(),
            file_description: item.description.clone(),
            time_created: item.time_created,
            time_updated: item.time_updated,
            subscriptions: item.subscriptions,
            favorited: item.favorited,
            views: item.views,
            tags: item.tags.clone(),
        }
    }
}

impl WorkshopItem {
    /// Converts from a WorkshopItem to a WorkshopSearchItem
    pub fn to_search_item(&self) -> WorkshopSearchItem {
        WorkshopSearchItem::from_item(&self)
    }
    /// Converts to a WorkshopItem from a WorkshopSearchItem
    pub fn from_search_item(sitem: &WorkshopSearchItem) -> WorkshopItem {
        sitem.to_item()
    }
}

pub struct Workshop {
    client: Client,
}

pub struct AuthedWorkshop {
    apikey: String,
    client: Client,
}

pub struct ProxyWorkshop {
    client: Client,
    url: String
}

#[allow(dead_code)]
impl Workshop {
    ///Creates a new workshop instance, client will be auto created if None
    pub fn new(client: Option<Client>) -> Workshop {
        let client = match client {
            Some(client) => client,
            None => reqwest::blocking::Client::new()
        };
        Workshop {
            client,
        }
    }

    ///Gets an authorized workshop, allows access to methods that require api keys. 
    ///Get api keys from https://steamcommunity.com/dev/apikey
    pub fn login(&mut self, apikey: String) -> AuthedWorkshop {
        AuthedWorkshop {
            apikey: apikey,
            client: self.client.clone()
        }
    }

    /// Allows you to use AuthedWorkshop methods using a proxy to handle.
    /// Public search proxy: https://jackz.me/scripts/workshop.php?mode=search
    pub fn proxy(&self, url: String) -> ProxyWorkshop {
        ProxyWorkshop {
            client: self.client.clone(),
            url: url
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
    pub fn get_published_file_details(&self, fileids: &[String]) -> Result<Vec<WorkshopItem>, reqwest::Error> {
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
            .post("https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/")
            .header("User-Agent", &USER_AGENT.to_string())
            .form(&params)
            .send()?
            .error_for_status()?
            .json::<Value>()?;

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

    /// Gets the collection details (all the children of this item). Returns a list of children fileids which can be sent directly to get_published_file_details()
    pub fn get_collection_details(&self, fileid: &str) -> Result<Option<Vec<String>>, reqwest::Error> {
        let mut params = HashMap::new();
        params.insert("collectioncount", "1");
        params.insert("publishedfileids[0]", &fileid);
        let details: WSCollectionResponse = self.client
            .post("https://api.steampowered.com/ISteamRemoteStorage/GetCollectionDetails/v1/")
            .header("User-Agent", USER_AGENT.to_string())
            .form(&params)
            .send()?
            .error_for_status()?
            .json::<WSCollectionResponse>()?;
           
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

}

impl AuthedWorkshop {
    ///Search for workshop items, returns only fileids
    pub fn search_ids(&self, appid: u64, query: &str, count: usize) -> Result<Vec<String>, reqwest::Error> {
        let details = self.client.get("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?")
            .header("User-Agent", USER_AGENT.to_string())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", &count.to_string()),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("key", &self.apikey),
            ])
            .send()?
            .json::<WSSearchResponse<WSSearchIdBody>>()?;

        let mut fileids: Vec<String> = Vec::new();
        if details.total > 0 {
            for res in &details.response.unwrap().publishedfiledetails {
                fileids.push(res.publishedfileid.to_string());
            }
        }
        Ok(fileids)
    }

    ///Searches for workshop items, returns full metadata
    pub fn search_full(&self, appid: u64, query: &str, count: usize) -> Result<Vec<WorkshopSearchItem>, reqwest::Error> {
        let details = self.client.get("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?")
            .header("User-Agent", USER_AGENT.to_string())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", &count.to_string()),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("return_metadata", "1"),
                ("key", &self.apikey),
            ])
            .send()?
            .json::<WSSearchResponse<WorkshopSearchItem>>()?;

        if details.total > 0 {
            Ok(details.response.unwrap().publishedfiledetails)
        } else {
            Ok(vec!())
        }
    }

    /// Check if the user can subscribe to the published file
    pub fn can_subscribe(&self, fileid: &str) -> Result<bool, reqwest::Error> {
        let details: serde_json::Value = self.client
            .get("https://api.steampowered.com/IPublishedFileService/CanSubscribe/v1/?key=7250BBE4BC2ECA0E16197B38E3675988&publishedfileid=122447941")
            .header("User-Agent", USER_AGENT.to_string())
            .query(&[
                "key", &self.apikey,
                "publishedfileid", fileid
            ])
            .send()?
            .error_for_status()?
            .json()?;
        Ok(details["response"]["can_subscribe"].as_bool().unwrap_or(false))
    }
}


impl ProxyWorkshop {
    ///Searches for workshop items, returns their file ids
    pub fn search_ids(&self, appid: u64, query: &str, count: usize) -> Result<Vec<String>, reqwest::Error> {
        let details = self.client.get(&self.url)
            .header("User-Agent", USER_AGENT.to_string())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", &count.to_string()),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("v", &env!("CARGO_PKG_VERSION")),
            ])
            .send()?
            .json::<WSSearchResponse<WSSearchIdBody>>()?;

        let mut fileids: Vec<String> = Vec::new();
        if details.total > 0 {
            for res in &details.response.unwrap().publishedfiledetails {
                fileids.push(res.publishedfileid.to_string());
            }
        }
        Ok(fileids)
    }

    ///Searches for workshop items, returns full metadata.
    ///Does not require api key by using https://jackz.me/scripts/workshop.php?mode=search
    pub fn search_full(&self, appid: u64, query: &str, count:usize) -> Result<Vec<WorkshopSearchItem>, reqwest::Error> {
        let details = self.client.get(&self.url)
            .header("User-Agent", USER_AGENT.to_string())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", &count.to_string()),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("return_metadata", "1"),
            ])
            .send()?
            .json::<WSSearchResponse<WorkshopSearchItem>>()?;

        if details.total > 0 {
            Ok(details.response.unwrap().publishedfiledetails)
        } else {
            Ok(vec!())
        }
    }
}