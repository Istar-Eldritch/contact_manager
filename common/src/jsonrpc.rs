/*!
Helpers that keep the responses standardized
*/

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tide::http::mime;
use tide::{Body, Response, StatusCode};
use uuid::Uuid;

/// Representaiton of a success response
///
/// https://www.jsonrpc.org/specification#response_object
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JSONRPCSuccess<T: Serialize> {
    /// Id of the request
    pub id: Option<Uuid>,
    /// The result
    pub result: T,
    /// The http code
    http_code: u16,
}

impl<T: Serialize> From<JSONRPCSuccess<T>> for Response {
    fn from(value: JSONRPCSuccess<T>) -> Self {
        Response::builder(value.http_code)
            .body(Body::from_json(&value).unwrap())
            .content_type(mime::JSON)
            .header("X-Content-Type", "application/json-rpc")
            .build()
    }
}

impl<T: Serialize, E> From<JSONRPCSuccess<T>> for Result<JSONRPCSuccess<T>, E> {
    fn from(json: JSONRPCSuccess<T>) -> Self {
        Ok(json)
    }
}

/// Helper to cast any value in a JSONRPCSucess struct
pub trait IntoRPCSuccess: Sized + Serialize {
    /// Get a 200 Ok JSONRPCSuccess
    fn into_ok(self) -> JSONRPCSuccess<Self>;
    /// Get a 201 Created JSONRPCSuccess
    fn into_created(self) -> JSONRPCSuccess<Self>;
}

impl<T: Serialize> IntoRPCSuccess for T {
    fn into_ok(self) -> JSONRPCSuccess<T> {
        JSONRPCSuccess::ok(self)
    }
    fn into_created(self) -> JSONRPCSuccess<T> {
        JSONRPCSuccess::created(self)
    }
}

// impl<T: Serialize> From<JSONRPCSuccess<T>> for Response {
//     fn from(success: JSONRPCSuccess<T>) -> Self {
//         let mut response = Response::new(success.http_code);

//         response.set_body(Body::from_json(&success).unwrap());

//         response
//     }
// }

impl<T: Serialize> JSONRPCSuccess<T> {
    fn new(http_code: StatusCode, result: T) -> Self {
        JSONRPCSuccess {
            id: None,
            result,
            http_code: http_code.into(),
        }
    }

    /// 200 OK
    pub fn ok(result: T) -> Self {
        Self::new(StatusCode::Ok, result)
    }

    /// 201 CREATED
    pub fn created(result: T) -> Self {
        Self::new(StatusCode::Created, result)
    }

    /// Set the request id of the response
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Changes the result value of the response
    pub fn result(mut self, result: T) -> Self {
        self.result = result;
        self
    }
}

/// Representation of a error response
///
/// https://www.jsonrpc.org/specification#error_object
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JSONRPCErrorObject<D = ()> {
    /// A String providing a short description of the error.
    pub message: String,
    /// A Number that indicates the error type that occurred.
    pub code: u16,
    /// A Primitive or Structured value that contains additional information about the error.
    pub data: D,
}

/// Representation of a error response
///
/// https://www.jsonrpc.org/specification#error_object
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JSONRPCError<E: Serialize = JSONRPCErrorObject<()>> {
    /// The request identifier
    pub id: Option<Uuid>,
    error: E,
    http_code: u16,
}

impl<E: Serialize + DeserializeOwned> From<JSONRPCError<E>> for Response {
    fn from(value: JSONRPCError<E>) -> Self {
        Response::builder(value.http_code)
            .body(Body::from_json(&value).unwrap())
            .content_type(mime::JSON)
            .build()
    }
}

impl<D: Serialize> JSONRPCError<JSONRPCErrorObject<D>> {
    fn new(code: StatusCode, data: D) -> Self {
        JSONRPCError {
            id: None,
            error: JSONRPCErrorObject {
                code: code.into(),
                message: String::from(code.canonical_reason()),
                data,
            },
            http_code: code.into(),
        }
    }

    /// 400 Bad Request
    pub fn bad_request(data: D) -> Self {
        Self::new(StatusCode::BadRequest, data)
    }

    /// 401 Unauthorized
    pub fn unauthorized(data: D) -> Self {
        Self::new(StatusCode::Unauthorized, data)
    }

    /// 403 Forbidden
    pub fn forbidden(data: D) -> Self {
        Self::new(StatusCode::Forbidden, data)
    }

    /// 409 Conflict
    pub fn conflict(data: D) -> Self {
        Self::new(StatusCode::Conflict, data)
    }

    /// 500 Internal Server Error response
    pub fn internal(data: D) -> Self {
        Self::new(StatusCode::InternalServerError, data)
    }

    /// 404 Not Found
    pub fn not_found(data: D) -> Self {
        Self::new(StatusCode::NotFound, data)
    }

    /// Sets the request identifier
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the error code
    pub fn code(mut self, code: impl Into<u16>) -> Self {
        self.error.code = code.into();
        self
    }

    /// Sets the error message
    pub fn message(mut self, message: &str) -> Self {
        self.error.message = String::from(message);
        self
    }

    /// Sets the aditional data
    pub fn data(mut self, data: D) -> Self {
        self.error.data = data;
        self
    }
}
