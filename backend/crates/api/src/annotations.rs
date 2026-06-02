//! `/annotations` CRUD, guarded by RBAC. An annotation (a quick scored note about
//! a collaborator) lives in the tenant schema, so every handler queries
//! `ctx.tenant_db` after the `ctx.require(...)` guard. `create` validates the
//! referenced collaborator exists and is active (a dangling reference → `422`).
//! Manager is derived from the collaborator at read time (not stored). Removal is
//! a soft delete; `PATCH` writes only the fields present in the body.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use entity::{annotation, collaborator};
use sea_orm::prelude::{Date, Uuid};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use service::permission::Resource;

use crate::extract::{AuthRejection, TenantContext};

#[derive(Serialize)]
struct AnnotationView {
    id: String,
    collaborator_id: String,
    note_date: sea_orm::prelude::DateTimeWithTimeZone,
    score1_number: i32,
    score1_description: Option<String>,
    score1_type: String,
    ask_amount_days: bool,
    score2_number: Option<i32>,
    score2_description: Option<String>,
    score2_type: Option<String>,
    amount_days: Option<i32>,
    main_note: Option<String>,
    period_start_date: Option<Date>,
    observation: Option<String>,
    recorded_on_mobile: bool,
    active: bool,
}

impl From<annotation::Model> for AnnotationView {
    fn from(model: annotation::Model) -> Self {
        Self {
            id: model.id.to_string(),
            collaborator_id: model.collaborator_id.to_string(),
            note_date: model.note_date,
            score1_number: model.score1_number,
            score1_description: model.score1_description,
            score1_type: model.score1_type,
            ask_amount_days: model.ask_amount_days,
            score2_number: model.score2_number,
            score2_description: model.score2_description,
            score2_type: model.score2_type,
            amount_days: model.amount_days,
            main_note: model.main_note,
            period_start_date: model.period_start_date,
            observation: model.observation,
            recorded_on_mobile: model.recorded_on_mobile,
            active: model.active,
        }
    }
}

#[derive(Deserialize)]
pub struct AnnotationQuery {
    #[serde(default)]
    collaborator_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateAnnotation {
    collaborator_id: Uuid,
    note_date: sea_orm::prelude::DateTimeWithTimeZone,
    score1_number: i32,
    score1_type: String,
    #[serde(default)]
    score1_description: Option<String>,
    #[serde(default)]
    ask_amount_days: bool,
    #[serde(default)]
    score2_number: Option<i32>,
    #[serde(default)]
    score2_description: Option<String>,
    #[serde(default)]
    score2_type: Option<String>,
    #[serde(default)]
    amount_days: Option<i32>,
    #[serde(default)]
    main_note: Option<String>,
    #[serde(default)]
    period_start_date: Option<Date>,
    #[serde(default)]
    observation: Option<String>,
    #[serde(default)]
    recorded_on_mobile: bool,
}

#[derive(Deserialize)]
pub struct UpdateAnnotation {
    #[serde(default)]
    note_date: Option<sea_orm::prelude::DateTimeWithTimeZone>,
    #[serde(default)]
    score1_number: Option<i32>,
    #[serde(default)]
    score1_type: Option<String>,
    #[serde(default)]
    score1_description: Option<String>,
    #[serde(default)]
    ask_amount_days: Option<bool>,
    #[serde(default)]
    score2_number: Option<i32>,
    #[serde(default)]
    score2_description: Option<String>,
    #[serde(default)]
    score2_type: Option<String>,
    #[serde(default)]
    amount_days: Option<i32>,
    #[serde(default)]
    main_note: Option<String>,
    #[serde(default)]
    period_start_date: Option<Date>,
    #[serde(default)]
    observation: Option<String>,
    #[serde(default)]
    recorded_on_mobile: Option<bool>,
    #[serde(default)]
    active: Option<bool>,
}

/// `GET /annotations` — lists active annotations newest-first; optional
/// `?collaborator_id=` filter. Requires `annotation.read`.
pub async fn list(
    ctx: TenantContext,
    Query(query): Query<AnnotationQuery>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::AnnotationRead).await?;

    let mut find = annotation::Entity::find().filter(annotation::Column::Active.eq(true));
    if let Some(collaborator_id) = query.collaborator_id {
        find = find.filter(annotation::Column::CollaboratorId.eq(collaborator_id));
    }
    let annotations = find
        .order_by_desc(annotation::Column::NoteDate)
        .all(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    let body: Vec<AnnotationView> = annotations.into_iter().map(AnnotationView::from).collect();
    Ok(Json(body).into_response())
}

/// `POST /annotations` — creates an annotation for a collaborator. Requires
/// `annotation.create`. An unknown/inactive collaborator → `422`.
pub async fn create(
    ctx: TenantContext,
    Json(body): Json<CreateAnnotation>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::AnnotationCreate).await?;

    let conn = ctx.tenant_db.as_ref();
    if !collaborator_exists(conn, body.collaborator_id).await? {
        return Ok(unprocessable("unknown collaborator"));
    }

    let created = annotation::ActiveModel {
        id: Set(Uuid::new_v4()),
        collaborator_id: Set(body.collaborator_id),
        note_date: Set(body.note_date),
        score1_number: Set(body.score1_number),
        score1_type: Set(body.score1_type),
        score1_description: Set(body.score1_description),
        ask_amount_days: Set(body.ask_amount_days),
        score2_number: Set(body.score2_number),
        score2_description: Set(body.score2_description),
        score2_type: Set(body.score2_type),
        amount_days: Set(body.amount_days),
        main_note: Set(body.main_note),
        period_start_date: Set(body.period_start_date),
        observation: Set(body.observation),
        recorded_on_mobile: Set(body.recorded_on_mobile),
        active: Set(true),
        ..Default::default()
    }
    .insert(conn)
    .await
    .map_err(|_| AuthRejection::Internal)?;

    Ok((StatusCode::CREATED, Json(AnnotationView::from(created))).into_response())
}

/// `PATCH /annotations/{id}` — updates the fields present in the body. Requires
/// `annotation.update`. Unknown id → `404`.
pub async fn update(
    ctx: TenantContext,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAnnotation>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::AnnotationUpdate).await?;

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = annotation::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: annotation::ActiveModel = model.into();
    if let Some(value) = body.note_date {
        active_model.note_date = Set(value);
    }
    if let Some(value) = body.score1_number {
        active_model.score1_number = Set(value);
    }
    if let Some(value) = body.score1_type {
        active_model.score1_type = Set(value);
    }
    if let Some(value) = body.score1_description {
        active_model.score1_description = Set(Some(value));
    }
    if let Some(value) = body.ask_amount_days {
        active_model.ask_amount_days = Set(value);
    }
    if let Some(value) = body.score2_number {
        active_model.score2_number = Set(Some(value));
    }
    if let Some(value) = body.score2_description {
        active_model.score2_description = Set(Some(value));
    }
    if let Some(value) = body.score2_type {
        active_model.score2_type = Set(Some(value));
    }
    if let Some(value) = body.amount_days {
        active_model.amount_days = Set(Some(value));
    }
    if let Some(value) = body.main_note {
        active_model.main_note = Set(Some(value));
    }
    if let Some(value) = body.period_start_date {
        active_model.period_start_date = Set(Some(value));
    }
    if let Some(value) = body.observation {
        active_model.observation = Set(Some(value));
    }
    if let Some(value) = body.recorded_on_mobile {
        active_model.recorded_on_mobile = Set(value);
    }
    if let Some(active) = body.active {
        active_model.active = Set(active);
    }
    let updated = active_model
        .update(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(Json(AnnotationView::from(updated)).into_response())
}

/// `DELETE /annotations/{id}` — deactivates an annotation (soft delete). Requires
/// `annotation.delete`. Unknown id → `404`; success → `204`.
pub async fn delete(ctx: TenantContext, Path(id): Path<Uuid>) -> Result<Response, AuthRejection> {
    ctx.require(Resource::AnnotationDelete).await?;

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = annotation::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: annotation::ActiveModel = model.into();
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
        Json(serde_json::json!({ "error": "annotation not found" })),
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
