use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub data: Option<T>,
    pub error: Option<ApiErrorBody>,
    pub meta: Option<Meta>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Meta {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub total: Option<u64>,
    pub total_pages: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl Meta {
    pub fn paginated(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        Self {
            page: Some(page),
            per_page: Some(per_page),
            total: Some(total),
            total_pages: Some(total_pages),
            request_id: None,
        }
    }
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
            meta: None,
        }
    }

    pub fn with_meta(mut self, meta: Meta) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl ApiResponse<()> {
    pub fn error(message: String) -> Self {
        Self::error_with_code("ERROR".to_string(), message)
    }

    pub fn error_with_code(code: String, message: String) -> Self {
        Self {
            data: None,
            error: Some(ApiErrorBody {
                code,
                message,
                details: None,
            }),
            meta: None,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self)).into_response()
    }
}

/// Wrapper to send ApiResponse with custom status code
pub struct WithStatus<T: Serialize>(pub StatusCode, pub ApiResponse<T>);

impl<T: Serialize> IntoResponse for WithStatus<T> {
    fn into_response(self) -> Response {
        (self.0, axum::Json(self.1)).into_response()
    }
}
