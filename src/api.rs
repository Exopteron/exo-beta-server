pub struct APIHelper {

}
impl APIHelper {
    pub async fn authenticate_user(name: &str, server_id: &str) -> anyhow::Result<bool> {
        let mut builder = reqwest::Client::builder();
        let url = format!("http://session.minecraft.net/game/checkserver.jsp?user={user}&serverId={id}", user = name, id = server_id);
        log::info!("Url: {:?}", url);
        let resp = builder.build()?.get(&url).header("Host", "session.minecraft.net").header("Connection", "close").send().await?;
        log::info!("got");
        match resp.text().await?.as_str() {
            "YES" => {
                return Ok(true);
            }
            _ => {
                return Ok(false);
            }
        }
    }
}