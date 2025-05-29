use crate::model::datafile::{
    Config, File, FileStatResponse, FileTagsResponse, Hash, QueryFilter, RecordResponse, URL,
};
use crate::model::dataset::{
    DataDictionary, DatasetDataResponse, Datasets, DatasetsResponse, FieldGroupSummary,
};
use crate::query_builder::sql_builder::ComposeQuery;
use crate::repo_config::{RepoConfig, SignData};
use crate::util;
use log::{debug, info, warn};
use poem::web::Data;
use poem_openapi::{
    param::Header,
    param::Path,
    param::Query,
    payload::{Json, PlainText},
    ApiResponse, Object, OpenApi, Tags,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Tags)]
enum FileApiTags {
    Files,
    File,
}

#[derive(Tags)]
enum DatasetApiTags {
    Datasets,
    Dataset,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct ErrorMessage {
    msg: String,
}

#[derive(ApiResponse)]
pub enum GetRecordsResponse<
    S: Serialize
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
> {
    #[oai(status = 200)]
    Ok(Json<RecordResponse<S>>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),

    #[oai(status = 500)]
    InternalServerError(Json<ErrorMessage>),
}

impl<
        S: Serialize
            + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > GetRecordsResponse<S>
{
    pub fn ok(record_response: RecordResponse<S>) -> Self {
        Self::Ok(Json(record_response))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }

    pub fn internal_server_error(msg: String) -> Self {
        Self::InternalServerError(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum GetRecordResponse<
    S: Serialize
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
> {
    #[oai(status = 200)]
    Ok(Json<S>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),

    #[oai(status = 500)]
    InternalServerError(Json<ErrorMessage>),
}

impl<
        S: Serialize
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > GetRecordResponse<S>
{
    pub fn ok(record_response: S) -> Self {
        Self::Ok(Json(record_response))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }

    pub fn internal_server_error(msg: String) -> Self {
        Self::InternalServerError(Json(ErrorMessage { msg }))
    }
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

#[derive(ApiResponse)]
enum GetDatasetsResponse {
    #[oai(status = 200)]
    Ok(Json<DatasetsResponse>),

    #[oai(status = 500)]
    InternalError(PlainText<String>),

    #[oai(status = 400)]
    BadRequest(PlainText<String>),

    #[oai(status = 404)]
    NotFound(PlainText<String>),
}

#[derive(ApiResponse)]
enum GetDatasetDataDictionaryResponse {
    #[oai(status = 200)]
    Ok(Json<DataDictionary>),

    #[oai(status = 404)]
    NotFound(PlainText<String>),

    #[oai(status = 500)]
    InternalError(PlainText<String>),

    #[oai(status = 400)]
    BadRequest(PlainText<String>),
}

#[derive(ApiResponse)]
enum GetDatasetDataResponse {
    #[oai(status = 200)]
    Ok(Json<DatasetDataResponse>),

    #[oai(status = 404)]
    NotFound(PlainText<String>),

    #[oai(status = 500)]
    InternalError(PlainText<String>),

    #[oai(status = 400)]
    BadRequest(PlainText<String>),
}

#[derive(ApiResponse)]
enum GetDatasetGroupsResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<FieldGroupSummary>>),

    #[oai(status = 404)]
    NotFound(PlainText<String>),

    #[oai(status = 500)]
    InternalError(PlainText<String>),

    #[oai(status = 400)]
    BadRequest(PlainText<String>),
}

pub struct BioMinerIndexdApi;

#[OpenApi(prefix_path = "/api/v1")]
impl BioMinerIndexdApi {
    /// Call `/api/v1/files` to create a file instance.
    #[oai(
        path = "/files",
        method = "post",
        tag = "FileApiTags::Files",
        operation_id = "createFile"
    )]
    async fn create_file(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        config: Data<&Arc<Config>>,
        params: Json<CreateFile>,
    ) -> PostResponse {
        let pool = pool.clone();
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
        let size = params.size as i64;

        let hash = &params.md5sum;
        if !util::validate_hash(hash, "md5") {
            return PostResponse::BadRequest(PlainText("Invalid hash value.".to_string()));
        };

        let url = match &params.url {
            Some(url) => {
                if util::which_protocol(url).is_none() {
                    return PostResponse::BadRequest(PlainText(
                        "Invalid url protocol.".to_string(),
                    ));
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
        match file.add(&pool, &hash, url, alias).await {
            Ok(()) => PostResponse::Ok(Json(GuidResponse { guid: file.guid })),
            Err(e) => PostResponse::BadRequest(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/files` with query params to fetch files.
    #[oai(
        path = "/files",
        method = "get",
        tag = "FileApiTags::Files",
        operation_id = "fetchFiles"
    )]
    async fn fetch_files(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
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
    ) -> GetRecordsResponse<File> {
        let pool = pool.clone();
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
        let contain_alias = contain_alias.clone().unwrap_or_else(|| true);
        let contain_url = contain_url.clone().unwrap_or_else(|| true);
        let contain_tag = contain_tag.clone().unwrap_or_else(|| true);

        info!(
            "Query with (guid: {:?}, filename: {:?}, baseid: {:?}, status: {:?}, 
           uploader: {:?}, hash: {:?}, alias: {:?}, url: {:?}), page: {:?}, page_size: {:?}",
            guid, filename, baseid, status, uploader, hash, alias, url, page, page_size
        );

        let files = RecordResponse::<File>::query_files(
            &pool,
            QueryFilter::new(
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
            ),
            page,
            page_size,
            contain_alias,
            contain_url,
            contain_tag,
        )
        .await
        .unwrap();

        debug!("Files: {:?}", files);
        GetRecordsResponse::ok(files)
    }

    /// Call `/api/v1/files/:id` to fetch the file.
    #[oai(
        path = "/files/:id",
        method = "get",
        tag = "FileApiTags::File",
        operation_id = "getFile"
    )]
    async fn get_file(&self, pool: Data<&sqlx::PgPool>, id: Path<uuid::Uuid>) -> GetFileResponse {
        let pool = pool.clone();
        let guid = id.0.to_string();
        info!("Get file ({:?}) with params", guid);

        match File::get_file(&pool, &id).await {
            Ok(file) => GetFileResponse::Ok(Json(file)),
            Err(e) => return GetFileResponse::NotFound(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/files/:id` to sign the file and get the downloading link.
    #[oai(
        path = "/files/hash/:hash",
        method = "post",
        tag = "FileApiTags::File",
        operation_id = "signFileWithHash"
    )]
    async fn sign_file_with_hash(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        config: Data<&Arc<RepoConfig>>,
        hash: Path<String>,
        which_repo: Query<Option<String>>,
        #[oai(name = "X-Auth-Groups", deprecated)] auth_groups: Header<Option<String>>,
    ) -> PostSignResponse {
        let pool = pool.clone();
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

        match File::get_file_with_hash(&pool, &hash).await {
            Ok(file) => {
                if file.access == "private".to_string() {
                    if auth_groups.is_none()
                        || !util::has_permission(&auth_groups.unwrap()[..], &file.acl.unwrap()[..])
                    {
                        return PostSignResponse::Unauthorized(PlainText(format!(
                            "The data is private and you do not have permission to access."
                        )));
                    }
                }

                if let Some(hashes) = &file.hashes {
                    let hashes: Vec<Hash> = serde_json::from_value(hashes.clone()).unwrap();

                    match &file.urls {
                        Some(urls) => {
                            let urls: Vec<URL> = serde_json::from_value(urls.clone()).unwrap();
                            if let Some(idx) =
                                urls.iter().position(|item| item.url.contains(&which_repo))
                            {
                                let url = &urls[idx];
                                let identity = url.get_identity();
                                match config_arc.fetch_config(&which_repo, &identity) {
                                    Some(c) => {
                                        let sign_data = c.sign(&url.url);
                                        let sign_response = SignResponse {
                                            sign: sign_data,
                                            size: file.size as u64,
                                            hashes: hashes.clone(),
                                            filename: file.filename.clone(),
                                        };
                                        return PostSignResponse::Ok(Json(sign_response));
                                    }
                                    None => {
                                        return PostSignResponse::InternalError(PlainText(
                                            format!(
                                                "The data has not been released, please contact the administrator for more details."
                                            )
                                        ));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    return PostSignResponse::InternalError(PlainText(
                        "The data has no hashes, please contact the administrator for more details.".to_string(),
                    ));
                }

                // Last arm of the match
                return PostSignResponse::NotFound(PlainText(format!(
          "The data has not been released on {} repo, please contact the administrator to add it.",
          which_repo
        )));
            }
            Err(e) => return PostSignResponse::NotFound(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/files/:id` to sign the file and get the downloading link.
    #[oai(
        path = "/files/:id",
        method = "post",
        tag = "FileApiTags::File",
        operation_id = "signFile"
    )]
    async fn sign_file(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        config: Data<&Arc<RepoConfig>>,
        id: Path<uuid::Uuid>,
        which_repo: Query<Option<String>>,
        #[oai(name = "X-Auth-Groups", deprecated)] auth_groups: Header<Option<String>>,
    ) -> PostSignResponse {
        let pool = pool.clone();
        let config_arc = config.clone();
        let guid = id.0.to_string();
        let which_repo = match which_repo.0 {
            Some(which_repo) => which_repo,
            // TODO: Need to set a best repo, select gsa or select one based on the user's position.
            None => "node".to_string(),
        };
        let auth_groups = auth_groups.0;

        info!("Sign file {:?}", guid);

        match File::get_file(&pool, &id).await {
            Ok(file) => {
                if file.access == "private" {
                    if auth_groups.is_none()
                        || !util::has_permission(&auth_groups.unwrap()[..], &file.acl.unwrap()[..])
                    {
                        return PostSignResponse::Unauthorized(PlainText(format!(
                            "The data is private and you do not have permission to access."
                        )));
                    }
                }

                if let Some(hashes) = &file.hashes {
                    let hashes: Vec<Hash> = serde_json::from_value(hashes.clone()).unwrap();

                    match &file.urls {
                        Some(urls) => {
                            let urls: Vec<URL> = serde_json::from_value(urls.clone()).unwrap();

                            if let Some(idx) =
                                urls.iter().position(|item| item.url.contains(&which_repo))
                            {
                                let url = &urls[idx];
                                let identity = url.get_identity();
                                match config_arc.fetch_config(&which_repo, &identity) {
                                    Some(c) => {
                                        let sign_data = c.sign(&url.url);
                                        let sign_response = SignResponse {
                                            sign: sign_data,
                                            size: file.size as u64,
                                            hashes: hashes.clone(),
                                            filename: file.filename.clone(),
                                        };
                                        return PostSignResponse::Ok(Json(sign_response));
                                    }
                                    None => {
                                        return PostSignResponse::InternalError(
                                            PlainText(format!(
                                                "The data has not been released, please contact the administrator for more details."
                                            ))
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    return PostSignResponse::InternalError(PlainText(
                        "The data has no hashes, please contact the administrator for more details.".to_string(),
                    ));
                }

                // Last arm of the match
                return PostSignResponse::NotFound(PlainText(format!(
          "The data has not been released on {} repo, please contact the administrator to add it.",
          which_repo
        )));
            }
            Err(e) => return PostSignResponse::NotFound(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/files/:id/url` to add url for the file.
    #[oai(
        path = "/files/:id/url",
        method = "put",
        tag = "FileApiTags::File",
        operation_id = "addUrlToFile"
    )]
    async fn add_url(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<uuid::Uuid>,
        params: Json<AddFileUrl>,
    ) -> PutResponse {
        let pool = pool.clone();
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

        match File::add_url(&pool, &id.0, url, &uploader, &status).await {
            Ok(()) => PutResponse::Ok(Json(MessageResponse {
                msg: "Success".to_string(),
            })),
            Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/files/:id/alias` to add alias for the file.
    #[oai(
        path = "/files/:id/alias",
        method = "put",
        tag = "FileApiTags::File",
        operation_id = "addAliasToFile"
    )]
    async fn add_alias(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<uuid::Uuid>,
        params: Json<AddFileAlias>,
    ) -> PutResponse {
        let pool = pool.clone();
        info!("Updating file ({:?}) with params: {:?}", id.0, params);

        match File::add_alias(&pool, &id.0, &params.alias).await {
            Ok(()) => PutResponse::Ok(Json(MessageResponse {
                msg: "Success".to_string(),
            })),
            Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/files/:id/hash` to add hash for the file.
    #[oai(
        path = "/files/:id/hash",
        method = "put",
        tag = "FileApiTags::File",
        operation_id = "addHashToFile"
    )]
    async fn add_hash(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<uuid::Uuid>,
        params: Json<AddFileHash>,
    ) -> PutResponse {
        let pool = pool.clone();
        info!("Updating file ({:?}) with params: {:?}", id.0, params);

        match File::add_hash(&pool, &id.0, &params.hash).await {
            Ok(()) => PutResponse::Ok(Json(MessageResponse {
                msg: "Success".to_string(),
            })),
            Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/files/:id/tag` to add tag for the file.
    #[oai(
        path = "/files/:id/tag",
        method = "put",
        tag = "FileApiTags::File",
        operation_id = "addTagToFile"
    )]
    async fn add_tag(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<uuid::Uuid>,
        params: Json<AddFileTag>,
    ) -> PutResponse {
        let pool = pool.clone();
        info!("Updating file ({:?}) with params: {:?}", id.0, params);

        match File::add_tag(&pool, &id.0, &params.field_name, &params.field_value).await {
            Ok(()) => PutResponse::Ok(Json(MessageResponse {
                msg: "Success".to_string(),
            })),
            Err(e) => PutResponse::BadRequest(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/files/tags` to fetch all tags.
    #[oai(
        path = "/files/tags",
        method = "get",
        tag = "FileApiTags::Files",
        operation_id = "getTags"
    )]
    async fn list_tags(&self, pool: Data<&Arc<sqlx::PgPool>>) -> GetTagsResponse {
        let pool = pool.clone();

        match FileTagsResponse::get_fields(&pool).await {
            Ok(tags) => GetTagsResponse::Ok(Json(tags)),
            Err(e) => GetTagsResponse::InternalError(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/files/stat` to get the statistics data.
    #[oai(
        path = "/files/stat",
        method = "get",
        tag = "FileApiTags::Files",
        operation_id = "getFileStat"
    )]
    async fn get_stat(&self, pool: Data<&Arc<sqlx::PgPool>>) -> GetStatResponse {
        let pool = pool.clone();

        match FileStatResponse::get_stat(&pool).await {
            Ok(stat) => GetStatResponse::Ok(Json(stat)),
            Err(e) => GetStatResponse::InternalError(PlainText(e.to_string())),
        }
    }

    /// Call `/api/v1/datasets` to get the datasets.
    #[oai(
        path = "/datasets",
        method = "get",
        tag = "DatasetApiTags::Datasets",
        operation_id = "getDatasets"
    )]
    async fn get_datasets(
        &self,
        page: Query<Option<usize>>,
        page_size: Query<Option<usize>>,
        query_str: Query<Option<String>>,
    ) -> GetDatasetsResponse {
        let base_path = std::env::var("BIOMINER_INDEXD_DATA_DIR").unwrap();
        let page = page.0.unwrap_or(1);
        let page_size = page_size.0.unwrap_or(10);
        let query_str = query_str.0;

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetDatasetsResponse::BadRequest(PlainText(err));
                }
            },
            None => None,
        };

        let datasets =
            match Datasets::search(&base_path.into(), &query, Some(page), Some(page_size), None) {
                Ok(datasets) => datasets,
                Err(e) => {
                    warn!("Failed to search datasets: {}", e);
                    return GetDatasetsResponse::InternalError(PlainText(e.to_string()));
                }
            };

        GetDatasetsResponse::Ok(Json(datasets))
    }

    /// Call `/api/v1/datasets` to get the datasets.
    #[oai(
        path = "/datasets/:key/data-dictionary",
        method = "get",
        tag = "DatasetApiTags::Datasets",
        operation_id = "getDataDictionary"
    )]
    async fn get_data_dictionary(&self, key: Path<String>) -> GetDatasetDataDictionaryResponse {
        let base_path = match std::env::var("BIOMINER_INDEXD_DATA_DIR") {
            Ok(path) => path,
            Err(e) => {
                warn!("Failed to get BIOMINER_INDEXD_DATA_DIR: {}", e);
                return GetDatasetDataDictionaryResponse::InternalError(PlainText(e.to_string()));
            }
        };

        let datasets = match Datasets::load(&base_path.into()) {
            Ok(datasets) => datasets,
            Err(e) => {
                warn!("Failed to load datasets: {}", e);
                return GetDatasetDataDictionaryResponse::InternalError(PlainText(e.to_string()));
            }
        };

        let dataset = match datasets.get(&key.0) {
            Ok(dataset) => dataset,
            Err(e) => {
                warn!("Failed to get dataset: {}", e);
                return GetDatasetDataDictionaryResponse::NotFound(PlainText(e.to_string()));
            }
        };

        let data_dictionary = match dataset.load_data_dictionary() {
            Ok(data_dictionary) => data_dictionary,
            Err(e) => {
                warn!("Failed to load data dictionary: {}", e);
                return GetDatasetDataDictionaryResponse::InternalError(PlainText(e.to_string()));
            }
        };

        GetDatasetDataDictionaryResponse::Ok(Json(data_dictionary))
    }

    /// Call `/api/v1/datasets/:key/groups` to get the dataset groups.
    #[oai(
        path = "/datasets/:key/groups",
        method = "get",
        tag = "DatasetApiTags::Datasets",
        operation_id = "getDatasetGroups"
    )]
    async fn get_dataset_groups(
        &self,
        key: Path<String>,
        field_key: Query<Option<String>>,
        query: Query<Option<String>>,
    ) -> GetDatasetGroupsResponse {
        let base_path = match std::env::var("BIOMINER_INDEXD_DATA_DIR") {
            Ok(path) => path,
            Err(e) => {
                warn!("Failed to get BIOMINER_INDEXD_DATA_DIR: {}", e);
                return GetDatasetGroupsResponse::InternalError(PlainText(e.to_string()));
            }
        };

        // TODO: Whether we should check the field is in the data dictionary?
        if field_key.0.is_none() {
            warn!("Field is required");
            return GetDatasetGroupsResponse::BadRequest(PlainText("Field is required".to_string()));
        }

        let field_key = match field_key.0 {
            Some(field_key) => {
                if field_key.is_empty() {
                    warn!("Field is required");
                    return GetDatasetGroupsResponse::BadRequest(PlainText("Field is required".to_string()));
                }
                field_key
            }
            None => {
                warn!("Field is required");
                return GetDatasetGroupsResponse::BadRequest(PlainText("Field is required".to_string()));
            }
        };

        let query = match query.0 {
            Some(query) => match ComposeQuery::from_str(&query) {
                Ok(query) => query,
                Err(e) => {
                    warn!("Failed to parse query string: {}", e);
                    return GetDatasetGroupsResponse::BadRequest(PlainText(e.to_string()));
                }
            },
            None => None,
        };

        let datasets = match Datasets::load(&base_path.into()) {
            Ok(datasets) => datasets,
            Err(e) => {
                warn!("Failed to load datasets: {}", e);
                return GetDatasetGroupsResponse::InternalError(PlainText(e.to_string()));
            }
        };

        let dataset = match datasets.get(&key.0) {
            Ok(dataset) => dataset,
            Err(e) => {
                warn!("Failed to get dataset: {}", e);
                return GetDatasetGroupsResponse::NotFound(PlainText(e.to_string()));
            }
        };

        let groups = match dataset.group_by(&field_key, &query) {
            Ok(groups) => groups,
            Err(e) => {
                warn!("Failed to group dataset: {}", e);
                return GetDatasetGroupsResponse::InternalError(PlainText(e.to_string()));
            }
        };

        GetDatasetGroupsResponse::Ok(Json(groups))
    }

    /// Call `/api/v1/datasets/:key/data` to get the dataset data.
    #[oai(
        path = "/datasets/:key/data",
        method = "get",
        tag = "DatasetApiTags::Datasets",
        operation_id = "getDatasetData"
    )]
    async fn get_dataset_data(
        &self,
        key: Path<String>,
        query: Query<Option<String>>,
        page: Query<Option<usize>>,
        page_size: Query<Option<usize>>,
        order_by: Query<Option<String>>,
    ) -> GetDatasetDataResponse {
        let base_path = match std::env::var("BIOMINER_INDEXD_DATA_DIR") {
            Ok(path) => path,
            Err(e) => {
                warn!("Failed to get BIOMINER_INDEXD_DATA_DIR: {}", e);
                return GetDatasetDataResponse::InternalError(PlainText(e.to_string()));
            }
        };

        let query = match query.0 {
            Some(query) => match ComposeQuery::from_str(&query) {
                Ok(query) => query,
                Err(e) => {
                    warn!("Failed to parse query string: {}", e);
                    return GetDatasetDataResponse::BadRequest(PlainText(e.to_string()));
                }
            },
            None => None,
        };

        let page = page.0.unwrap_or(1);
        let page_size = page_size.0.unwrap_or(10);
        let order_by = order_by.0;

        let datasets = match Datasets::load(&base_path.into()) {
            Ok(datasets) => datasets,
            Err(e) => {
                warn!("Failed to load datasets: {}", e);
                return GetDatasetDataResponse::InternalError(PlainText(e.to_string()));
            }
        };

        let dataset = match datasets.get(&key.0) {
            Ok(dataset) => dataset,
            Err(e) => {
                warn!("Failed to get dataset: {}", e);
                return GetDatasetDataResponse::NotFound(PlainText(e.to_string()));
            }
        };

        let data = match dataset.search(
            &query,
            Some(page as u64),
            Some(page_size as u64),
            order_by.as_deref(),
        ) {
            Ok(data) => data,
            Err(e) => {
                warn!("Failed to search dataset: {}", e);
                return GetDatasetDataResponse::InternalError(PlainText(e.to_string()));
            }
        };

        GetDatasetDataResponse::Ok(Json(data))
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
