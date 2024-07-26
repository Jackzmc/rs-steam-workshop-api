# rs-steam-workshop-api
A rust api library for the steam web workshop api

## Basic Usage

```rs
use steam_workshop_api::Workshop;

let mut workshop = Workshop::new();
workshop.set_api_key(Some("yourapikey".to_string()));
let fileids = vec!['121221044', '1643520526'];
let details: Vec<WorkshopItem> = match workshop.get_published_file_details(&fileids) {
    Ok(details) => details,
    Err(err) => { 
        eprintln!("Failed to get file info");
    }
};
```