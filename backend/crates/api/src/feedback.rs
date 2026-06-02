//! `/feedbacks` CRUD, guarded by RBAC. Feedback lives in the tenant schema and
//! references a collaborator, so every handler queries `ctx.tenant_db` after the
//! `ctx.require(...)` guard. `create` validates the referenced collaborator exists
//! and is active (a dangling reference → `422`). Manager/sector are not stored;
//! they are derived from the collaborator at read time when a view needs them.
//! Removal is a soft delete (`active = false`); `PATCH` writes only the fields
//! present in the body.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use entity::{collaborator, feedback};
use sea_orm::prelude::{DateTimeWithTimeZone, Uuid};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use service::permission::Resource;

use crate::extract::{AuthRejection, TenantContext};

#[derive(Serialize)]
struct FeedbackView {
    id: String,
    collaborator_id: String,
    feedback_date: DateTimeWithTimeZone,
    next_feedback_date: Option<DateTimeWithTimeZone>,
    expectation_contract_observation: Option<String>,
    expectation_contract_observation_private: Option<String>,
    status: Option<String>,
    active: bool,
}

impl From<feedback::Model> for FeedbackView {
    fn from(model: feedback::Model) -> Self {
        Self {
            id: model.id.to_string(),
            collaborator_id: model.collaborator_id.to_string(),
            feedback_date: model.feedback_date,
            next_feedback_date: model.next_feedback_date,
            expectation_contract_observation: model.expectation_contract_observation,
            expectation_contract_observation_private: model
                .expectation_contract_observation_private,
            status: model.status,
            active: model.active,
        }
    }
}

#[derive(Deserialize)]
pub struct FeedbackQuery {
    #[serde(default)]
    collaborator_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateFeedback {
    collaborator_id: Uuid,
    feedback_date: DateTimeWithTimeZone,
    #[serde(default)]
    next_feedback_date: Option<DateTimeWithTimeZone>,
    #[serde(default)]
    expectation_contract_observation: Option<String>,
    #[serde(default)]
    expectation_contract_observation_private: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateFeedback {
    #[serde(default)]
    feedback_date: Option<DateTimeWithTimeZone>,
    #[serde(default)]
    next_feedback_date: Option<DateTimeWithTimeZone>,
    #[serde(default)]
    expectation_contract_observation: Option<String>,
    #[serde(default)]
    expectation_contract_observation_private: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    active: Option<bool>,
}

/// `GET /feedbacks` — lists the tenant's active feedback, newest first; an
/// optional `?collaborator_id=` narrows to one collaborator. Requires
/// `feedback.read`.
pub async fn list(
    ctx: TenantContext,
    Query(query): Query<FeedbackQuery>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::FeedbackRead).await?;

    let mut find = feedback::Entity::find().filter(feedback::Column::Active.eq(true));
    if let Some(collaborator_id) = query.collaborator_id {
        find = find.filter(feedback::Column::CollaboratorId.eq(collaborator_id));
    }
    let feedbacks = find
        .order_by_desc(feedback::Column::FeedbackDate)
        .all(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    let body: Vec<FeedbackView> = feedbacks.into_iter().map(FeedbackView::from).collect();
    Ok(Json(body).into_response())
}

/// `POST /feedbacks` — creates a feedback for a collaborator. Requires
/// `feedback.create`. An unknown/inactive collaborator → `422`.
pub async fn create(
    ctx: TenantContext,
    Json(body): Json<CreateFeedback>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::FeedbackCreate).await?;

    let conn = ctx.tenant_db.as_ref();
    if !collaborator_exists(conn, body.collaborator_id).await? {
        return Ok(unprocessable("unknown collaborator"));
    }

    let created =
        feedback::ActiveModel {
            id: Set(Uuid::new_v4()),
            collaborator_id: Set(body.collaborator_id),
            feedback_date: Set(body.feedback_date),
            next_feedback_date: Set(body.next_feedback_date),
            expectation_contract_observation: Set(body.expectation_contract_observation),
            expectation_contract_observation_private: Set(
                body.expectation_contract_observation_private
            ),
            status: Set(body.status),
            active: Set(true),
            ..Default::default()
        }
        .insert(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok((StatusCode::CREATED, Json(FeedbackView::from(created))).into_response())
}

/// `PATCH /feedbacks/{id}` — updates the fields present in the body. Requires
/// `feedback.update`. Unknown id → `404`.
pub async fn update(
    ctx: TenantContext,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateFeedback>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::FeedbackUpdate).await?;

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = feedback::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: feedback::ActiveModel = model.into();
    if let Some(feedback_date) = body.feedback_date {
        active_model.feedback_date = Set(feedback_date);
    }
    if let Some(next_feedback_date) = body.next_feedback_date {
        active_model.next_feedback_date = Set(Some(next_feedback_date));
    }
    if let Some(value) = body.expectation_contract_observation {
        active_model.expectation_contract_observation = Set(Some(value));
    }
    if let Some(value) = body.expectation_contract_observation_private {
        active_model.expectation_contract_observation_private = Set(Some(value));
    }
    if let Some(status) = body.status {
        active_model.status = Set(Some(status));
    }
    if let Some(active) = body.active {
        active_model.active = Set(active);
    }
    let updated = active_model
        .update(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(Json(FeedbackView::from(updated)).into_response())
}

/// `DELETE /feedbacks/{id}` — deactivates a feedback (soft delete). Requires
/// `feedback.delete`. Unknown id → `404`; success → `204`.
pub async fn delete(ctx: TenantContext, Path(id): Path<Uuid>) -> Result<Response, AuthRejection> {
    ctx.require(Resource::FeedbackDelete).await?;

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = feedback::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: feedback::ActiveModel = model.into();
    active_model.active = Set(false);
    active_model
        .update(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Whether an active collaborator with `id` exists in the tenant.
async fn collaborator_exists(
    conn: &impl sea_orm::ConnectionTrait,
    id: Uuid,
) -> Result<bool, AuthRejection> {
    let found = collaborator::Entity::find_by_id(id)
        .filter(collaborator::Column::Active.eq(true))
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;
    Ok(found.is_some())
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "feedback not found" })),
    )
        .into_response()
}

fn unprocessable(message: &str) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": message })),
    )
        .into_response()
}
