use steam_workshop_api::*;
use crate::common::get_workshop;

mod common;

#[test]
fn test_one_workshop_item() -> Result<(), Error> {
    let ws = SteamWorkshop::new();
    let ids = vec!["121090376".to_string()];
    let mut details = ws.get_published_file_details(&ids)?;
    assert_eq!(details.len(), 1);
    let addon = details.remove(0);
    assert_eq!(addon.publishedfileid, "121090376");

    Ok(())
}

#[test]
fn test_multiple_items() -> Result<(), Error> {
    let ws = SteamWorkshop::new();
    let ids = vec!["121090376".to_string(), "2764154633".to_string()];
    let mut details = ws.get_published_file_details(&ids)?;
    assert_eq!(details.len(), 2);
    let addon = details.remove(0);
    assert_eq!(addon.publishedfileid, "121090376");

    Ok(())
}

#[test]
fn test_search() -> Result<(), Error> {
    let ws = get_workshop();
    ws.search_items(&SearchOptions {
        query: "test".to_string(),
        count: 10,
        app_id: 550,
        cursor: None,
        required_tags: None,
        excluded_tags: None,
    }).and(Ok(()))
}

#[test]
fn test_subscribe_unsubscribe() {
    let ws = get_workshop();
    ws.subscribe("2855027013", false).unwrap();
    ws.unsubscribe("2855027013").unwrap();
}