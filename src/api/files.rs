use crate::database::{query_files, QueryParamsFile};
use log::info;
use poem::{
  get, handler, http::StatusCode, post, web::headers::ContentType, web::Data, web::Json,
  web::Query, IntoResponse, Response, Route,
};
use rbatis::{rbatis::Rbatis, PageRequest};
use std::sync::Arc;

pub fn route_config() -> Route {
  Route::new().at("/api/v1/files", get(fetch_files))
}

#[handler]
async fn fetch_files(
  Query(query_params): Query<QueryParamsFile>,
  rb: Data<&Arc<Rbatis>>,
) -> impl IntoResponse {
  let rb_arc = rb.clone();
  let mut rb = rb_arc.acquire().await.unwrap();
  info!("Query Params: {:?}", query_params);
  let page = query_params.page.unwrap_or_else(|| 1);
  let page_size = query_params.page_size.unwrap_or_else(|| 10);
  let guid = query_params.guid.unwrap_or_else(|| "".to_string());
  let filename = query_params.filename.unwrap_or_else(|| "".to_string());
  let baseid = query_params.baseid.unwrap_or_else(|| "".to_string());
  let status = query_params.status.unwrap_or_else(|| "".to_string());
  let uploader = query_params.uploader.unwrap_or_else(|| "".to_string());
  let hash = query_params.hash.unwrap_or_else(|| "".to_string());
  let alias = query_params.alias.unwrap_or_else(|| "".to_string());
  let url = query_params.url.unwrap_or_else(|| "".to_string());
  let files = query_files(
    &mut rb.as_executor(),
    &PageRequest::new(page, page_size),
    &guid,
    &filename,
    &baseid,
    &status,
    &uploader,
    &hash,
    &alias,
    &url,
  )
  .await
  .unwrap();

  info!("Files: {:?}", files);
  Json(files)
}
