use web::error::HouseApiError;
use web::house_api::HouseAPI;

#[actix_web::main]
async fn main() -> Result<(), HouseApiError> {
    let server_address = "127.0.0.1:8081";
    let api = HouseAPI::start(server_address).await?;

    //interactions

    api.wait().await?;

    Ok(())
}
