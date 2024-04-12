use erased_serde::serialize_trait_object;
use serde::{Deserialize, Serialize};

use axum::{
    routing::get,
    routing::post,
    Router,
    extract::Json,
};

/**
 * trait AnyTask and impl AnyTask for TrampolineTask<T>
 * are needed to support a heterogeneous Vec of TrampolineTask<X>
 * with varying types X i.e. returning a collection of tasks of different
 * task types i.e. vector of trait objects i.e. Vec<Box<dyn AnyTask>>
 * https://stackoverflow.com/questions/67460078/how-do-i-declare-a-vector-of-generic-structs-where-the-generic-field-has-multipl
 * 
 * erased_serde is needed to support making that collection of
 * trait objects serializable. (using ordinary Serialize results in a Rust
 * compile error "trait object does not satisfy object safety rules")
 * https://stackoverflow.com/questions/50021897/how-to-implement-serdeserialize-for-a-boxed-trait-object
 * 
 */
trait AnyTask: erased_serde::Serialize {}
impl<'a, T> AnyTask for TrampolineTask<T> where T: Serialize {}

serialize_trait_object!(AnyTask);

#[derive(Serialize)]
struct TrampolineResponse {
    /// responses can contain multiple tasks, of varying types
    tasks: Vec<Box<dyn AnyTask>>,
}

#[derive(Serialize)]
struct TrampolineTask<T> {
    #[serde(rename = "type")]
    task_type: String,
    task: T,
}

#[derive(Deserialize)]
struct GenerateEmailTask {
    email_address: String,
}

#[derive(Serialize)]
struct SendEmailTask {
    email_address: String,
    email_subject: String,
    email_body: String,
}

async fn generate_email(Json(payload): Json<GenerateEmailTask>) -> Json<TrampolineResponse> {
    Json::from(TrampolineResponse {
        tasks: vec![Box::new(TrampolineTask {
            task_type: "email-pipeline-send-email".to_string(),
            task: SendEmailTask {
                email_body: format!("Hello {}", payload.email_address),
                email_address: payload.email_address,
                email_subject: String::from("Hello world!"),
            }
        })]
    })
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/generate-email", post(generate_email));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}