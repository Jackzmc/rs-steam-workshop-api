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
    assert_eq!(details.len(), 1);
    let addon = details.remove(0);
    assert_eq!(addon.publishedfileid, "121090376");

    Ok(())
}