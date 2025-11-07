use std::sync::OnceLock;
use steam_workshop_api::SteamWorkshop;

static WS: OnceLock<SteamWorkshop> = OnceLock::new();
pub fn get_workshop() -> &'static SteamWorkshop {
    WS.get_or_init(|| {
        let mut client = SteamWorkshop::new();
        client.set_apikey(Some(env!("STEAM_API_KEY").to_string()));
        client
    })
}