use crate::database::{
  query_files, Config, File, FilePageResponse, FileStatResponse, FileTagsResponse, Hash,
};
use crate::repo_config::{RepoConfig, SignData};
use crate::util;
use log::{debug, info, warn};
use poem::web::Data;
use poem_openapi::{
  param::Path,
  param::Query,
  param::Header,
  payload::{Json, PlainText},
  ApiResponse, Object, OpenApi, Tags,
};
use rbatis::{rbatis::Rbatis, PageRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Tags)]
enum FileApiTags {
  Files,
  File,
}

#[derive(ApiResponse)]
enum GetResponse {
  #[oai(status = 200)]
  Ok(Json<FilePageResponse>),

  #[oai(status = 404)]
  NotFound(PlainText<String>),
}

#[derive(ApiResponse)]
enum GetFileResponse {
  #[oai(status = 200)]
  Ok(Json<File>),

  #[oai(status = 404)]
  NotFound(PlainText<String>),

  #[oai(status = 500)]
  InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum PostSignResponse {
  #[oai(status = 201)]
  Ok(Json<SignResponse>),

  #[oai(status = 404)]
  NotFound(PlainText<String>),

  #[oai(status = 400)]
  BadRequest(PlainText<String>),

  #[oai(status = 401)]
  Unauthorized(PlainText<String>),

  #[oai(status = 500)]
  InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum GetTagsResponse {
  #[oai(status = 200)]
  Ok(Json<FileTagsResponse>),

  #[oai(status = 500)]
  InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum GetStatResponse {
  #[oai(status = 200)]
  Ok(Json<FileStatResponse>),

  #[oai(status = 500)]
  InternalError(PlainText<String>),
}

#[derive(ApiResponse)]
enum PutResponse {
  #[oai(status = 201)]
  Ok(Json<MessageResponse>),

  #[oai(status = 400)]
  BadRequest(PlainText<String>),
}

#[derive(ApiResponse)]
enum PostResponse {
  #[oai(status = 201)]
  Ok(Json<GuidResponse>),

  #[oai(status = 400)]
  BadRequest(PlainText<String>),
}

pub struct FilesApi;

#[OpenApi]
impl FilesApi {
  /// Call `/api/v1/files` to create a file instance.
  #[oai(
    path = "/api/v1/files",
    method = "post",
    tag = "FileApiTags::Files",
    operation_id = "createFile"
  )]
  async fn create_file(
    &self,
    rb: Data<&Arc<Rbatis>>,
    config: Data<&Arc<Config>>,
    params: Json<CreateFile>,
  ) -> PostResponse {
    let rb_arc = rb.clone();
    info!("Creating file with params: {:?}", params);

    let registry_id = &config.registry_id;
    let filename = match &params.filename {
      Some(filename) => filename,
      None => "",
    };
    let uploader = match &params.uploader {
      Some(uploader) => uploader,
      None => "biominer-admin",
    };
    let size = params.size;

    let hash = &params.md5sum;
    if !util::validate_hash(hash, "md5") {
      return PostResponse::BadRequest(PlainText("Invalid hash value.".to_string()));
    };

    let url = match &params.url {
      Some(url) => {
        if util::which_protocol(url).is_none() {
          return PostResponse::BadRequest(PlainText("Invalid url protocol.".to_string()));
        } else {
          Some(url.as_str())
        }
      }
      _ => None,
    };

    let alias = match &params.alias {
      Some(alias) => Some(alias.as_str()),
      None => None,
    };

    let mut file = File::new(filename, size, uploader, registry_id);
    match file.add(&rb_arc, &hash, url, alias).await {
      Ok(()) => PostResponse::Ok(Json(GuidResponse { guid: file.guid })),
      Err(e) => PostResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  /// Call `/api/v1/files` with query params to fetch files.
  #[oai(
    path = "/api/v1/files",
    method = "get",
    tag = "FileApiTags::Files",
    operation_id = "fetchFiles"
  )]
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

    let guid = match &guid.0 {
      Some(guid) => guid,
      None => "",
    };
    let filename = match &filename.0 {
      Some(filename) => filename,
      None => "",
    };
    let baseid = match &baseid.0 {
      Some(baseid) => baseid,
      None => "",
    };
    let status = match &status.0 {
      Some(status) => status,
      None => "",
    };
    let uploader = match &uploader.0 {
      Some(uploader) => uploader,
      None => "",
    };
    let hash = match &hash.0 {
      Some(hash) => hash,
      None => "",
    };
    let alias = match &alias.0 {
      Some(alias) => alias,
      None => "",
    };
    let url = match &url.0 {
      Some(url) => url,
      None => "",
    };
    let field_name = match &field_name.0 {
      Some(field_name) => field_name,
      None => "",
    };
    let field_value = match &field_value.0 {
      Some(field_value) => field_value,
      None => "",
    };
    let contain_alias = contain_alias.clone().unwrap_or_else(|| true) as usize;
    let contain_url = contain_url.clone().unwrap_or_else(|| true) as usize;
    let contain_tag = contain_tag.clone().unwrap_or_else(|| true) as usize;

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
    GetResponse::Ok(Json(FilePageResponse::from(files)))
  }

  /// Call `/api/v1/files/:id` to fetch the file.
  #[oai(
    path = "/api/v1/files/:id",
    method = "get",
    tag = "FileApiTags::File",
    operation_id = "getFile"
  )]
  async fn get_file(&self, rb: Data<&Arc<Rbatis>>, id: Path<uuid::Uuid>) -> GetFileResponse {
    let rb_arc = rb.clone();
    let guid = id.0.to_string();
    info!("Get file ({:?}) with params", guid);

    match File::get_file(&rb_arc, &id).await {
      Some(file) => GetFileResponse::Ok(Json(file)),
      None => return GetFileResponse::NotFound(PlainText("File not found.".to_string())),
    }
  }

  /// Call `/api/v1/files/:id` to sign the file and get the downloading link.
  #[oai(
    path = "/api/v1/files/hash/:hash",
    method = "post",
    tag = "FileApiTags::File",
    operation_id = "signFileWithHash"
  )]
  async fn sign_file_with_hash(
    &self,
    rb: Data<&Arc<Rbatis>>,
    config: Data<&Arc<RepoConfig>>,
    hash: Path<String>,
    which_repo: Query<Option<String>>,
    #[oai(name = "X-Auth-Groups", deprecated)]
    auth_groups: Header<Option<String>>,
  ) -> PostSignResponse {
    let rb_arc = rb.clone();
    let config_arc = config.clone();
    let hash = hash.0.to_string();
    let which_repo = match which_repo.0 {
      Some(which_repo) => which_repo,
      // TODO: Need to set a best repo, select gsa or select one based on the user's position.
      None => "node".to_string(),
    };
    let auth_groups = auth_groups.0;

    match util::which_hash_type(&hash) {
      Some(_) => {}
      None => {
        return PostSignResponse::BadRequest(PlainText("Invalid hash type.".to_string()));
      }
    };

    info!("Sign file with {:?}", hash);

    match File::get_file_with_hash(&rb_arc, &hash).await {
      Some(file) => {
        if file.access == "private".to_string() {
          if auth_groups.is_none() || !util::has_permission(&auth_groups.unwrap()[..], &file.acl.unwrap()[..]) {
            return PostSignResponse::Unauthorized(PlainText(format!(
              "The data is private and you do not have permission to access."
            )));
          }
        }

        match &file.urls {
          Some(urls) => {
            if let Some(idx) = urls.iter().position(|item| item.url.contains(&which_repo)) {
              let url = &urls[idx];
              let identity = url.get_identity();
              match config_arc.fetch_config(&which_repo, &identity) {
                Some(c) => {
                  let sign_data = c.sign(&url.url);
                  let sign_response = SignResponse {
                    sign: sign_data,
                    size: file.size,
                    hashes: file.hashes.clone().unwrap(),
                    filename: file.filename.clone(),
                  };
                  return PostSignResponse::Ok(Json(sign_response));
                }
                None => {
                  return PostSignResponse::InternalError(PlainText(
                    "The data has not been released, please contact the administrator for more details.".to_string(),
                  ));
                }
              }
            }
          }
          _ => {}
        }

        // Last arm of the match
        return PostSignResponse::NotFound(PlainText(format!(
          "The data has not been released on {} repo, please contact the administrator to add it.",
          which_repo
        )));
      }
      None => return PostSignResponse::NotFound(PlainText("File not found.".to_string())),
    }
  }

  /// Call `/api/v1/files/:id` to sign the file and get the downloading link.
  #[oai(
    path = "/api/v1/files/:id",
    method = "post",
    tag = "FileApiTags::File",
    operation_id = "signFile"
  )]
  async fn sign_file(
    &self,
    rb: Data<&Arc<Rbatis>>,
    config: Data<&Arc<RepoConfig>>,
    id: Path<uuid::Uuid>,
    which_repo: Query<Option<String>>,
    #[oai(name = "X-Auth-Groups", deprecated)]
    auth_groups: Header<Option<String>>,
  ) -> PostSignResponse {
    let rb_arc = rb.clone();
    let config_arc = config.clone();
    let guid = id.0.to_string();
    let which_repo = match which_repo.0 {
      Some(which_repo) => which_repo,
      // TODO: Need to set a best repo, select gsa or select one based on the user's position.
      None => "node".to_string(),
    };
    let auth_groups = auth_groups.0;

    info!("Sign file {:?}", guid);

    match File::get_file(&rb_arc, &id).await {
      Some(file) => {
        if file.access == "private" {
          if auth_groups.is_none() || !util::has_permission(&auth_groups.unwrap()[..], &file.acl.unwrap()[..]) {
            return PostSignResponse::Unauthorized(PlainText(format!(
              "The data is private and you do not have permission to access."
            )));
          }
        }

        match &file.urls {
          Some(urls) => {
            if let Some(idx) = urls.iter().position(|item| item.url.contains(&which_repo)) {
              let url = &urls[idx];
              let identity = url.get_identity();
              match config_arc.fetch_config(&which_repo, &identity) {
                Some(c) => {
                  let sign_data = c.sign(&url.url);
                  let sign_response = SignResponse {
                    sign: sign_data,
                    size: file.size,
                    hashes: file.hashes.clone().unwrap(),
                    filename: file.filename.clone(),
                  };
                  return PostSignResponse::Ok(Json(sign_response));
                }
                None => {
                  return PostSignResponse::InternalError(PlainText(
                    "The data has not been released, please contact the administrator for more details.".to_string(),
                  ));
                }
              }
            }
          }
          _ => {}
        }

        // Last arm of the match
        return PostSignResponse::NotFound(PlainText(format!(
          "The data has not been released on {} repo, please contact the administrator to add it.",
          which_repo
        )));
      }
      None => return PostSignResponse::NotFound(PlainText("File not found.".to_string())),
    }
  }

  /// Call `/api/v1/files/:id/url` to add url for the file.
  #[oai(
    path = "/api/v1/files/:id/url",
    method = "put",
    tag = "FileApiTags::File",
    operation_id = "addUrlToFile"
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

    let url = &params.url;

    match util::which_protocol(url) {
      Some(protocol) => protocol,
      None => {
        warn!("Cannot find the protocol {}", url);
        // TODO: error message should be customized
        return PutResponse::BadRequest(PlainText("Invalid url.".to_string()));
      }
    };

    match File::add_url(&rb_arc, &id.0, url, &uploader, &status).await {
      Ok(()) => PutResponse::Ok(Json(MessageResponse {
        msg: "Success".to_string(),
      })),
      Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  /// Call `/api/v1/files/:id/alias` to add alias for the file.
  #[oai(
    path = "/api/v1/files/:id/alias",
    method = "put",
    tag = "FileApiTags::File",
    operation_id = "addAliasToFile"
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
      Ok(()) => PutResponse::Ok(Json(MessageResponse {
        msg: "Success".to_string(),
      })),
      Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  /// Call `/api/v1/files/:id/hash` to add hash for the file.
  #[oai(
    path = "/api/v1/files/:id/hash",
    method = "put",
    tag = "FileApiTags::File",
    operation_id = "addHashToFile"
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
      Ok(()) => PutResponse::Ok(Json(MessageResponse {
        msg: "Success".to_string(),
      })),
      Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  /// Call `/api/v1/files/:id/tag` to add tag for the file.
  #[oai(
    path = "/api/v1/files/:id/tag",
    method = "put",
    tag = "FileApiTags::File",
    operation_id = "addTagToFile"
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
      Ok(()) => PutResponse::Ok(Json(MessageResponse {
        msg: "Success".to_string(),
      })),
      Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
    }
  }

  /// Call `/api/v1/files/tags` to fetch all tags.
  #[oai(
    path = "/api/v1/files/tags",
    method = "get",
    tag = "FileApiTags::Files",
    operation_id = "getTags"
  )]
  async fn list_tags(&self, rb: Data<&Arc<Rbatis>>) -> GetTagsResponse {
    let rb_arc = rb.clone();

    match FileTagsResponse::get_fields(&rb_arc).await {
      Ok(tags) => GetTagsResponse::Ok(Json(tags)),
      Err(e) => GetTagsResponse::InternalError(PlainText(e.to_string())),
    }
  }

  /// Call `/api/v1/files/stat` to get the statistics data.
  #[oai(
    path = "/api/v1/files/stat",
    method = "get",
    tag = "FileApiTags::Files",
    operation_id = "getFileStat"
  )]
  async fn get_stat(&self, rb: Data<&Arc<Rbatis>>) -> GetStatResponse {
    let rb_arc = rb.clone();

    match FileStatResponse::get_stat(&rb_arc).await {
      Ok(stat) => GetStatResponse::Ok(Json(stat)),
      Err(e) => GetStatResponse::InternalError(PlainText(e.to_string())),
    }
  }
}

#[derive(Debug, Default, Clone, Serialize, Eq, PartialEq, Deserialize, Object)]
pub struct CreateFile {
  pub filename: Option<String>,
  pub uploader: Option<String>,
  pub md5sum: String,
  pub size: u64,
  pub alias: Option<String>,
  pub url: Option<String>,
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
pub struct GuidResponse {
  pub guid: String,
}

#[derive(Debug, Default, Clone, Serialize, Eq, PartialEq, Deserialize, Object)]
pub struct MessageResponse {
  pub msg: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Object)]
pub struct SignResponse {
  pub sign: SignData,
  pub size: u64,
  // At least one of the hashes exists.
  pub hashes: Vec<Hash>,
  pub filename: String,
}
