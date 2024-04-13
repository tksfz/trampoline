use erased_serde::serialize_trait_object;
use serde::{Deserialize, Serialize};

use axum::{
    routing::get,
    routing::post,
    Router,
    extract::Json,
};

struct EmailPipelineTaskTypes; 
impl EmailPipelineTaskTypes {
    #[allow(dead_code)]
    const START_BATCH: &str = "email-pipeline-start";

    const FETCH_USERS: &str = "email-pipeline-fetch-users";
    const SEND_EMAIL: &str = "email-pipeline-send-email";
    const GENERATE_EMAIL: &str = "email-pipeline-generate-email";
    const RECORD_SEND_RESULT: &str = "email-pipeline-record-send-result";
}

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
struct StartTask {
    email_subject: Option<String>,
    // TODO: a better example would be scheduling
}

#[derive(Serialize, Deserialize)]
struct FetchUsersTask {
    email_subject: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct GenerateEmailTask {
    email_address: String,
    email_subject: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SendEmailTask {
    email_address: String,
    email_subject: String,
    email_body: String,
}

#[derive(Serialize, Deserialize)]
struct RecordSendResultTask {
    email_address: String,
    successful: bool,
}

async fn start_batch(Json(payload): Json<StartTask>) -> Json<TrampolineResponse> {
    Json::from(TrampolineResponse {
        tasks: vec![Box::new(TrampolineTask {
            task_type: EmailPipelineTaskTypes::FETCH_USERS.to_owned(),
            task: FetchUsersTask {
                email_subject: payload.email_subject,
            }
        })]
    })
}

async fn fetch_users(Json(payload): Json<FetchUsersTask>) -> Json<TrampolineResponse> {
    let users = vec!["foo@nowhere.nowhere", "bar@nowhere.nowhere", "nobody@nowhere.nowhere"];
    Json::from(TrampolineResponse {
        tasks: users.into_iter().map(|addr| {
            Box::new(TrampolineTask {
                task_type: EmailPipelineTaskTypes::GENERATE_EMAIL.to_owned(),
                task: GenerateEmailTask {
                    email_address: addr.to_owned(),
                    email_subject: payload.email_subject.to_owned(),
                }
            }) as Box<dyn AnyTask>
        }).collect()
    })
}

async fn generate_email(Json(payload): Json<GenerateEmailTask>) -> Json<TrampolineResponse> {
    Json::from(TrampolineResponse {
        tasks: vec![Box::new(TrampolineTask {
            task_type: EmailPipelineTaskTypes::SEND_EMAIL.to_owned(),
            task: SendEmailTask {
                email_body: format!("Hello {}", payload.email_address),
                email_address: payload.email_address,
                email_subject: payload.email_subject.unwrap_or("Hello World!".to_owned()),
            }
        })]
    })
}

async fn send_email(Json(payload): Json<SendEmailTask>) -> Json<TrampolineResponse> {
    // Pretend to send the email, and move on to the next step
    log::info!("Pretending to send email to {} with subject {} and body {}", &payload.email_address, &payload.email_subject, &payload.email_body);
    Json::from(TrampolineResponse {
        tasks: vec![Box::new(TrampolineTask {
            task_type: EmailPipelineTaskTypes::RECORD_SEND_RESULT.to_owned(),
            task: RecordSendResultTask {
                email_address: payload.email_address,
                successful: true,
            }
        })]
    })
}

async fn record_send_result(Json(payload): Json<RecordSendResultTask>) -> Json<TrampolineResponse> {
    log::info!("Recording send result for email to {}, successful {}", &payload.email_address, &payload.successful);
    Json::from(TrampolineResponse {
        tasks: Vec::new()
    })
}

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/start", post(start_batch))
        .route("/fetch-users", post(fetch_users))
        .route("/generate-email", post(generate_email))
        .route("/send-email", post(send_email))
        .route("/record-send-result", post(record_send_result));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}