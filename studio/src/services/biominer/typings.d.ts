declare namespace API {
  type addAliasToFileParams = {
    id: string;
  };

  type AddFileAlias = {
    alias: string;
  };

  type AddFileHash = {
    hash: string;
  };

  type AddFileTag = {
    field_name: string;
    field_value: string;
  };

  type AddFileUrl = {
    url: string;
    status?: string;
    uploader?: string;
  };

  type addHashToFileParams = {
    id: string;
  };

  type addTagToFileParams = {
    id: string;
  };

  type addUrlToFileParams = {
    id: string;
  };

  type CreateFile = {
    filename?: string;
    uploader?: string;
    md5sum: string;
    size: number;
    alias?: string;
    url?: string;
  };

  type DataDictionary = {
    fields: DataDictionaryField[];
  };

  type DataDictionaryField = {
    key: string;
    name: string;
    data_type: string;
    description: string;
    notes: string;
    allowed_values: any;
    order: number;
  };

  type DatasetDataResponse = {
    records: any[];
    total: number;
    page: number;
    page_size: number;
  };

  type DatasetMetadata = {
    key: string;
    name: string;
    description: string;
    citation: string;
    pmid: string;
    groups: string[];
    tags: string[];
    total: number;
    is_filebased: boolean;
  };

  type DatasetsResponse = {
    records: DatasetMetadata[];
    total: number;
    page: number;
    page_size: number;
  };

  type ErrorMessage = {
    msg: string;
  };

  type fetchFilesParams = {
    page?: number;
    page_size?: number;
    guid?: string;
    filename?: string;
    baseid?: string;
    status?: string;
    uploader?: string;
    hash?: string;
    alias?: string;
    url?: string;
    field_name?: string;
    field_value?: string;
    contain_alias?: boolean;
    contain_url?: boolean;
    contain_tag?: boolean;
  };

  type File = {
    guid: string;
    filename: string;
    size: number;
    created_at: number;
    updated_at: number;
    status: string;
    baseid: string;
    rev: string;
    version: number;
    uploader: string;
    access: string;
    acl?: string;
    urls?: any;
    hashes?: any;
    aliases?: any;
    tags?: any;
  };

  type FileStatResponse = {
    total_size: number;
    num_of_files: number;
    num_of_baseid: number;
    version: string;
    registry_id: string;
  };

  type FileTagsResponse = {
    field_names: string[];
  };

  type getDataDictionaryParams = {
    key: string;
  };

  type getDatasetDataParams = {
    key: string;
    query?: string;
    page?: number;
    page_size?: number;
    order_by?: string;
  };

  type getDatasetsParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type getFileParams = {
    id: string;
  };

  type GuidResponse = {
    guid: string;
  };

  type Hash = {
    id: number;
    hash_type: string;
    hash: string;
    file?: string;
  };

  type MessageResponse = {
    msg: string;
  };

  type RecordResponse = {
    /** data */
    records: File[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type SignData = {
    header: string[];
    data: string[];
    baseurl: string;
    method: string;
    params: string[];
  };

  type signFileParams = {
    id: string;
    which_repo?: string;
  };

  type signFileWithHashParams = {
    hash: string;
    which_repo?: string;
  };

  type SignResponse = {
    sign: SignData;
    size: number;
    hashes: Hash[];
    filename: string;
  };
}
