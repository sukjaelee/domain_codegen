
use axum::http::{Method, StatusCode};

use clean_axum_demo::{
    common::dto::RestApiResponse,
    todo::{
        dto::{CreateTodoDto, TodoDto, UpdateTodoDto},
    },
};

use uuid::Uuid;
mod test_helpers;
use test_helpers::{
    deserialize_json_body, request_with_auth, request_with_auth_and_body, TEST_USER_ID,
};

async fn create_test_todo() -> TodoDto {
    let payload = CreateTodoDto {
        user_id: Default::default(),
        title: Default::default(),
        description: Default::default(),
        status: Default::default(),
        due_date: Default::default(),
        created_by: Default::default(),
        modified_by: Default::default(),
    };

    let response = request_with_auth_and_body(Method::POST, "/todo", &payload);
    let (parts, body) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<TodoDto> =
        deserialize_json_body(body).await.unwrap();
    response_body.0.data.unwrap()
}

#[tokio::test]
async fn test_create_todo() {
    let payload = CreateTodoDto {
        user_id: Default::default(),
        title: Default::default(),
        description: Default::default(),
        status: Default::default(),
        due_date: Default::default(),
        created_by: Default::default(),
        modified_by: Default::default(),
    };

    let response = request_with_auth_and_body(Method::POST, "/todo", &payload);
    let (parts, body) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<TodoDto> =
        deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK);
    let dto = response_body.0.data.unwrap();
    assert_eq!(dto.user_id, payload.user_id);
    assert_eq!(dto.title, payload.title);
    assert_eq!(dto.description, payload.description);
    assert_eq!(dto.status, payload.status);
    assert_eq!(dto.due_date, payload.due_date);
    assert_eq!(dto.created_by, payload.created_by);
    assert_eq!(dto.modified_by, payload.modified_by);
}

#[tokio::test]
async fn test_get_all_todo() {
    // Ensure at least one entity exists
    let _ = create_test_todo().await;

    let response = request_with_auth(Method::GET, "/todo");
    let (parts, body) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<Vec<TodoDto>> =
        deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK);
    let items = response_body.0.data.unwrap();
    assert!(!items.is_empty());
}

#[tokio::test]
async fn test_get_todo_by_id() {
    let entity = create_test_todo().await;
    let id = entity.id.clone();
    let url = format!("/todo/{}", id);
    let response = request_with_auth(Method::GET, url.as_str());
    let (parts, body) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<TodoDto> =
        deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK);
    let dto = response_body.0.data.unwrap();
    assert_eq!(dto.id, id);
}

#[tokio::test]
async fn test_update_todo() {
    let entity = create_test_todo().await;
    let id = entity.id.clone();
    let payload = UpdateTodoDto {
        user_id: Some(Default::default()),
        title: Some(Default::default()),
        description: Some(Default::default()),
        status: Some(Default::default()),
        due_date: Some(Default::default()),
        created_by: Some(Default::default()),
        modified_by: entity.modified_by.clone(),
        modified_at: Some(Default::default()),
    };
    let url = format!("/todo/{}", id);
    let response = request_with_auth_and_body(Method::PUT, url.as_str(), &payload);
    let (parts, body) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<TodoDto> =
        deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK);
    let dto = response_body.0.data.unwrap();
    assert_eq!(dto.id, id);
    assert_eq!(dto.user_id, payload.user_id.unwrap());
    assert_eq!(dto.title, payload.title.unwrap());
    assert_eq!(dto.description, payload.description.unwrap());
    assert_eq!(dto.status, payload.status.unwrap());
    assert_eq!(dto.due_date, payload.due_date.unwrap());
    assert_eq!(dto.created_by, payload.created_by.unwrap());
    assert_eq!(dto.modified_by, payload.modified_by);
    assert_eq!(dto.modified_at, payload.modified_at.unwrap());
}

#[tokio::test]
async fn test_delete_todo_not_found() {
    let non_existent_id = Uuid::new_v4().to_string();
    let url = format!("/todo/{}", non_existent_id);
    let response = request_with_auth(Method::DELETE, url.as_str());
    let (parts, body) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::NOT_FOUND);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_todo() {
    let entity = create_test_todo().await;
    let id = entity.id.clone();
    let url = format!("/todo/{}", id);
    let response = request_with_auth(Method::DELETE, url.as_str());
    let (parts, body) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK);
}