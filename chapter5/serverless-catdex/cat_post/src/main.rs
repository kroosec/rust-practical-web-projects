use lambda_http::{handler, lambda, Context, IntoResponse, Request, RequestExt};
use lambda_http::{HeaderValue, Response, StatusCode};
use rusoto_core::Region;
use rusoto_credential::{ChainProvider, ProvideAwsCredentials};
use rusoto_dynamodb::{AttributeValue, DynamoDB, DynamoDbClient, PutItemInput};
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::PutObjectRequest;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[derive(Deserialiaze)]
struct RequestBody {
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler(world)).await?;
    Ok(())
}

async fn cat_post(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let body: Requestbody = match result.payload() {
        Ok(Some(body)) => body,
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Invalid payload".into())
                .expect("Failed to render response"))
        }
    };

    let client = DynamoDbClient::new(Region::EuCentral1);
    let image_path = format!("image/{}.jpg", &body.name);
    let mut new_cat = HashMap::new();
    new_cat.insert(
        "image_path".to_string(),
        AttributeValue {
            s: Some(image_path.clone()),
            ..Default::default()
        },
    );

    let put_item_input = PutItemInput {
        table_name: "hani_catdex".to_string(),
        item: new_cat,
        ..Default::default()
    };

    match client.put_item(put_item_input).await {
        Ok(_) => Ok(json!(format!("created cat {}", body.name)).into_reponse),
        _ => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Something went wrong when writing to the database".into())
            .expect("Failed to render response")),
    }

    let credentials = ChainProvider::new().credentials().await.unwrap();

    let put_request = PutObjectRequest {
        bucket: "hani-catdex-frontend".to_string(),
        key: image_path,
        content_type: Some("image/jpg".to_string()),
        ..Default::default()
    };

    let presigned_url =
        put_request.get_presigned_url(&Region::EuCentral1, &credentials, &Default::default());

    let mut response = json!({ "upload_url": presigned_url }).into_response();
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
