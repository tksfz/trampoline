use serde::{Deserialize, Serialize};

use axum::{
    extract::{Json, Query}, routing::{get, post}, Router
};

#[derive(Deserialize)]
struct GenerateEmailParams {
    email_address: String,
    email_subject: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct EmailMessage {
    email_address: String,
    email_subject: String,
    email_body: String,
}

#[derive(Deserialize)]
struct SendResult {
    email_address: String,
    successful: bool,
}

async fn query_users() -> Json<Vec<String>> {
    // Run some query to find users, here we pretend and simply return a hard-coded list
    let users = vec!["foo@nowhere.nowhere", "bar@nowhere.nowhere", "nobody@nowhere.nowhere"];
    return Json::from(users.into_iter().map(|u| u.to_owned()).collect::<Vec<String>>());
}

async fn generate_email(Query(params): Query<GenerateEmailParams>) -> Json<EmailMessage> {
    // Generate an email
    let email_subject = params.email_subject.unwrap_or("Hello World!".to_owned());
    let email_body = format!("Hello {}", params.email_address);
    Json::from(EmailMessage {
                email_body: email_body,
                email_address: params.email_address,
                email_subject: email_subject,
            }
        )
}

async fn send_email(Json(payload): Json<EmailMessage>) -> Json<bool> {
    // Pretend to send the email
    log::info!("Pretending to send email to {} with subject {} and body {}", &payload.email_address, &payload.email_subject, &payload.email_body);
    return Json::from(true);
}

async fn record_send_result(Json(payload): Json<SendResult>) -> () {
    log::info!("Recording send result for email to {}, successful {}", &payload.email_address, &payload.successful);
}

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // build our application with a single route
    let app = Router::new()
        .route("/users", get(query_users))
        .route("/generate-email", get(generate_email))
        .route("/send-email", post(send_email))
        .route("/record-send-result", post(record_send_result));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}