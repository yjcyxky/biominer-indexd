use crate::database::{query_files, Config, File, FilePage, FileStat, FileTags};
use crate::util;
use log::{debug, info};
use poem::web::Data;
use poem_openapi::{
  param::Path,
  param::Query,
  payload::{Json, PlainText},
  ApiResponse, Object, OpenApi, Tags,
};
use rbatis::{rbatis::Rbatis, PageRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Tags)]
enum FileApiTags {
  GetFile,
  ListFiles,
  CreateFile,
  AddUrl,
  AddAlias,
  AddHash,
  AddTag,
  ListTags,
  DeleteFile,
  GetStat,
}

#[derive(ApiResponse)]
enum GetResponse {
  #[oai(status = 200)]
  Ok(Json<FilePage>),

  #[oai(status = 404)]
  NotFound(PlainText<String>),
}

#[derive(ApiResponse)]
enum GetTagsResponse {
  #[oai(status = 200)]
  Ok(Json<FileTags>),

  #[oai(status = 500)]
  InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum GetStatResponse {
  #[oai(status = 200)]
  Ok(Json<FileStat>),

  #[oai(status = 500)]
  InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum PutResponse {
  #[oai(status = 201)]
  Ok(Json<StatusResponse>),

  #[oai(status = 400)]
  BadRequest(PlainText<String>),
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
  #[oai(
    path = "/api/v1/files",
    method = "post",
    tag = "FileApiTags::CreateFile"
  )]
  async fn create_file(
    &self,
    rb: Data<&Arc<Rbatis>>,
    config: Data<&Arc<Config>>,
    params: Json<CreateFile>,
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

  #[oai(path = "/api/v1/files", method = "get", tag = "FileApiTags::ListFiles")]
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
    field_name: Query<Option<String>>,
    field_value: Query<Option<String>>,
    contain_alias: Query<Option<bool>>,
    contain_url: Query<Option<bool>>,
    contain_tag: Query<Option<bool>>,
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
    let field_name = field_name.clone().unwrap_or_else(|| "".to_string());
    let field_value = field_value.clone().unwrap_or_else(|| "".to_string());
    let contain_alias = contain_alias.clone().unwrap_or_else(|| false) as usize;
    let contain_url = contain_url.clone().unwrap_or_else(|| false) as usize;
    let contain_tag = contain_tag.clone().unwrap_or_else(|| false) as usize;

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
      &field_name,
      &field_value,
      &contain_alias,
      &contain_url,
      &contain_tag,
    )
    .await
    .unwrap();

    debug!("Files: {:?}", files);
    GetResponse::Ok(Json(FilePage::from(files)))
  }

  #[oai(
    path = "/api/v1/files/:id/url",
    method = "put",
    tag = "FileApiTags::AddUrl"
  )]
  async fn add_url(
    &self,
    rb: Data<&Arc<Rbatis>>,
    id: Path<uuid::Uuid>,
    params: Json<AddFileUrl>,
  ) -> PutResponse {
    let rb_arc = rb.clone();
    info!("Updating file ({:?}) with params: {:?}", id.0, params);

    let status = if let Some(status) = &params.status {
      status.clone()
    } else {
      "pending".to_string()
    };

    let uploader = if let Some(uploader) = &params.uploader {
      uploader.clone()
    } else {
      "biominer-admin".to_string()
    };

    match File::add_url(&rb_arc, &id.0, &params.url, &uploader, &status).await {
      Ok(()) => PutResponse::Ok(Json(StatusResponse {
        msg: "Success".to_string(),
      })),
      Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  #[oai(
    path = "/api/v1/files/:id/alias",
    method = "put",
    tag = "FileApiTags::AddAlias"
  )]
  async fn add_alias(
    &self,
    rb: Data<&Arc<Rbatis>>,
    id: Path<uuid::Uuid>,
    params: Json<AddFileAlias>,
  ) -> PutResponse {
    let rb_arc = rb.clone();
    info!("Updating file ({:?}) with params: {:?}", id.0, params);

    match File::add_alias(&rb_arc, &id.0, &params.alias).await {
      Ok(()) => PutResponse::Ok(Json(StatusResponse {
        msg: "Success".to_string(),
      })),
      Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  #[oai(
    path = "/api/v1/files/:id/hash",
    method = "put",
    tag = "FileApiTags::AddHash"
  )]
  async fn add_hash(
    &self,
    rb: Data<&Arc<Rbatis>>,
    id: Path<uuid::Uuid>,
    params: Json<AddFileHash>,
  ) -> PutResponse {
    let rb_arc = rb.clone();
    info!("Updating file ({:?}) with params: {:?}", id.0, params);

    match File::add_hash(&rb_arc, &id.0, &params.hash).await {
      Ok(()) => PutResponse::Ok(Json(StatusResponse {
        msg: "Success".to_string(),
      })),
      Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  #[oai(
    path = "/api/v1/files/:id/tag",
    method = "put",
    tag = "FileApiTags::AddTag"
  )]
  async fn add_tag(
    &self,
    rb: Data<&Arc<Rbatis>>,
    id: Path<uuid::Uuid>,
    params: Json<AddFileTag>,
  ) -> PutResponse {
    let rb_arc = rb.clone();
    info!("Updating file ({:?}) with params: {:?}", id.0, params);

    match File::add_tag(&rb_arc, &id.0, &params.field_name, &params.field_value).await {
      Ok(()) => PutResponse::Ok(Json(StatusResponse {
        msg: "Success".to_string(),
      })),
      Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  #[oai(
    path = "/api/v1/files/tags",
    method = "get",
    tag = "FileApiTags::ListTags"
  )]
  async fn list_tags(&self, rb: Data<&Arc<Rbatis>>) -> GetTagsResponse {
    let rb_arc = rb.clone();

    match FileTags::get_fields(&rb_arc).await {
      Ok(tags) => GetTagsResponse::Ok(Json(tags)),
      Err(e) => GetTagsResponse::InternalError(PlainText(e.to_string())),
    }
  }

  #[oai(
    path = "/api/v1/files/stat",
    method = "get",
    tag = "FileApiTags::GetStat"
  )]
  async fn get_stat(&self, rb: Data<&Arc<Rbatis>>) -> GetStatResponse {
    let rb_arc = rb.clone();

    match FileStat::get_stat(&rb_arc).await {
      Ok(stat) => GetStatResponse::Ok(Json(stat)),
      Err(e) => GetStatResponse::InternalError(PlainText(e.to_string())),
    }
  }
}

#[derive(Debug, Default, Clone, Serialize, Eq, PartialEq, Deserialize, Object)]
pub struct CreateFile {
  pub filename: Option<String>,
  pub uploader: Option<String>,
  pub hash: String,
  pub size: u64,
}

#[derive(Debug, Default, Clone, Serialize, Eq, PartialEq, Deserialize, Object)]
pub struct AddFileUrl {
  pub url: String,
  pub status: Option<String>,
  pub uploader: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Eq, PartialEq, Deserialize, Object)]
pub struct AddFileAlias {
  pub alias: String,
}

#[derive(Debug, Default, Clone, Serialize, Eq, PartialEq, Deserialize, Object)]
pub struct AddFileHash {
  pub hash: String,
}

#[derive(Debug, Default, Clone, Serialize, Eq, PartialEq, Deserialize, Object)]
pub struct AddFileTag {
  pub field_name: String,
  pub field_value: String,
}

#[derive(Debug, Default, Clone, Serialize, Eq, PartialEq, Deserialize, Object)]
pub struct StatusResponse {
  pub msg: String,
}
