use super::util;
use crate::database::{query_files, Config, File, FilePage};
use log::{debug, info};
use poem::web::Data;
use poem_openapi::{
  param::Query,
  payload::{Json, PlainText},
  ApiResponse, Object, OpenApi,
};
use rbatis::{rbatis::Rbatis, PageRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(ApiResponse)]
enum GetResponse {
  #[oai(status = 200)]
  Ok(Json<FilePage>),

  #[oai(status = 404)]
  NotFound(PlainText<String>),
}

#[derive(ApiResponse)]
enum PostResponse {
  #[oai(status = 201)]
  Ok(Json<File>),

  #[oai(status = 400)]
  BadRequest(PlainText<String>),
}

pub struct FilesApi;

#[OpenApi]
impl FilesApi {
  #[oai(path = "/api/v1/files", method = "post")]
  async fn create_file(
    &self,
    rb: Data<&Arc<Rbatis>>,
    config: Data<&Arc<Config>>,
    params: Json<PostFile>,
  ) -> PostResponse {
    let rb_arc = rb.clone();
    info!("Creating file with params: {:?}", params);

    let registry_id = config.registry_id.clone();
    let filename = &params.filename.clone().unwrap_or_else(|| "".to_string());
    let uploader = &params
      .uploader
      .clone()
      .unwrap_or_else(|| "biominer-admin".to_string());
    let size = params.size;

    let hash = &params.hash;
    if !util::validate_hash(hash, "md5") {
      return PostResponse::BadRequest(PlainText("Invalid hash value.".to_string()));
    };

    if File::check_hash_exists(&rb_arc, hash).await {
      return PostResponse::BadRequest(PlainText(
        "The hash value already exists or has been registered.".to_string(),
      ));
    }

    let mut file = File::new(&filename, size, &uploader, &registry_id);
    match file.add(&rb_arc, &hash).await {
      Ok(()) => PostResponse::Ok(Json(file)),
      Err(e) => PostResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  #[oai(path = "/api/v1/files", method = "get")]
  async fn fetch_files(
    &self,
    rb: Data<&Arc<Rbatis>>,
    page: Query<Option<u64>>,
    page_size: Query<Option<u64>>,
    guid: Query<Option<String>>,
    filename: Query<Option<String>>,
    baseid: Query<Option<String>>,
    status: Query<Option<String>>,
    uploader: Query<Option<String>>,
    hash: Query<Option<String>>,
    alias: Query<Option<String>>,
    url: Query<Option<String>>,
    contain_alias: Query<Option<bool>>,
    contain_url: Query<Option<bool>>,
  ) -> GetResponse {
    let rb_arc = rb.clone();
    let mut rb = rb_arc.acquire().await.unwrap();
    let page = page.unwrap_or_else(|| 1);
    let page_size = page_size.unwrap_or_else(|| 10);

    let guid = guid.clone().unwrap_or_else(|| "".to_string());
    let filename = filename.clone().unwrap_or_else(|| "".to_string());
    let baseid = baseid.clone().unwrap_or_else(|| "".to_string());
    let status = status.clone().unwrap_or_else(|| "".to_string());
    let uploader = uploader.clone().unwrap_or_else(|| "".to_string());
    let hash = hash.clone().unwrap_or_else(|| "".to_string());
    let alias = alias.clone().unwrap_or_else(|| "".to_string());
    let url = url.clone().unwrap_or_else(|| "".to_string());
    let contain_alias = contain_alias.clone().unwrap_or_else(|| false) as usize;
    let contain_url = contain_url.clone().unwrap_or_else(|| false) as usize;

    info!(
      "Query with (guid: {:?}, filename: {:?}, baseid: {:?}, status: {:?}, 
           uploader: {:?}, hash: {:?}, alias: {:?}, url: {:?}), page: {:?}, page_size: {:?}",
      guid, filename, baseid, status, uploader, hash, alias, url, page, page_size
    );

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
      &contain_alias,
      &contain_url,
    )
    .await
    .unwrap();

    debug!("Files: {:?}", files);
    GetResponse::Ok(Json(FilePage::from(files)))
  }
}

#[derive(Debug, Default, Clone, Serialize, Eq, PartialEq, Deserialize, Object)]
pub struct PostFile {
  pub filename: Option<String>,
  pub uploader: Option<String>,
  pub hash: String,
  pub size: u64,
}
