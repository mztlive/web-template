use super::errors::Result;
use mongodb::{Client, Database};

/// connect to the database
///
/// database name and uri are provided in the `AppConfig` struct
///
/// # Errors
///
/// This function will return an error if connection to the database fails.
pub async fn connect(uri: &str, db_name: &str) -> Result<(Client, Database)> {
    let client = mongodb::Client::with_uri_str(uri).await?;
    let database = client.database(db_name);
    Ok((client, database))
}
