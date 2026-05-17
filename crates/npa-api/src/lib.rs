//! Phase 5 API substrate.
//!
//! This crate is intentionally outside the trusted kernel. It handles wire JSON
//! decoding and request-shape validation for machine-facing endpoints.

mod json;
mod validation;

pub use json::{
    JsonDocument, JsonMember, JsonParseError, JsonParseErrorKind, JsonParseLimits, JsonSpan,
    JsonValue, JsonValueKind,
};
pub use validation::{
    delayed_json_payload, parse_request_body, parse_request_body_with_limits,
    parse_strict_u64_token, validate_json_object, DelayedJsonPayload, FieldSpec, JsonFieldType,
    JsonPath, JsonPathElement, MachineApiErrorKind, MachineApiRequestError,
    MachineApiRequestErrorReason, ObjectSchema, StrictUnsignedIntegerError, ValidatedObject,
};
