use lambda_http::{handler, lambda, Context, IntoResponse, Request};
use lambda_http::{Response, StatusCode};
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDB, DynamoDbClient, ScanInput};
use serde_json::json;
use std::collections::HashMap;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler(world)).await?;
    Ok(())
}

async fn cats(_: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let client = DynamoDbClient::new(Region::EuCentral1);

    let scan_input = ScanInput {
        table_name: "hani_catdex".to_string(),
        limit: Some(100),
        ..Default::default()
    };

    let response = match client.scan(scan_input).await {
        Ok(output) => match output.items {
            Some(items) => json!(items
                .into_iter()
                .map(|item| item.into_iter().map(|(k, v)| (k, v.s.unwrap())).collect())
                .collect::<Vec<HashMap<String, String>>>())
            .into_response(),
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("No cat yet".into())
                .expect("Failed to render response."),
        },
        Err(error) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("{:?}", error).into())
            .expect("Failed to render reponse"),
    };

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn world_handles() {
        let request = Request::default();
        let expected = json!({
        "message": "Go Serverless v1.0! Your function executed successfully!"
        })
        .into_response();
        let response = world(request, Context::default())
            .await
            .expect("expected Ok(_) value")
            .into_response();
        assert_eq!(response.body(), expected.body())
    }
}
