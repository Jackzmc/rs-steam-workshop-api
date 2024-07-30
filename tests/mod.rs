use steam_workshop_api::*;

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
    let mut ws = SteamWorkshop::new();
    ws.set_apikey(Some(env!("STEAM_API_KEY").to_string()));
    ws.search_items(&SearchOptions {
        query: "test".to_string(),
        count: 10,
        app_id: 550,
        cursor: None,
        required_tags: None,
        excluded_tags: None,
    }).and(Ok(()))
}