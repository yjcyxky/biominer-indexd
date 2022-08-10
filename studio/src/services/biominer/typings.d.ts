declare namespace API {
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

  type Alias = {
    id: number;
    name: string;
    file?: string;
  };

  type CreateFile = {
    filename?: string;
    uploader?: string;
    md5sum: string;
    size: number;
    alias?: string;
    url?: string;
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
    urls?: URL[];
    hashes?: Hash[];
    aliases?: Alias[];
    tags?: Tag[];
  };

  type FilePageResponse = {
    /** data */
    records: File[];
    /** total num */
    total: number;
    /** pages */
    pages: number;
    /** current page index */
    page_no: number;
    /** default 10 */
    page_size: number;
    /** is search_count */
    search_count: boolean;
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

  type SignData = {
    header: string[];
    data: string[];
    baseurl: string;
    method: string;
    params: string[];
  };

  type SignResponse = {
    sign: SignData;
    size: number;
    hashes: Hash[];
    filename: string;
  };

  type Tag = {
    id: number;
    field_name: string;
    field_value: string;
    file?: string;
  };

  type URL = {
    id: number;
    url: string;
    created_at: number;
    status: string;
    uploader: string;
    file?: string;
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

  type getFileParams = {
    id: string;
  };

  type signFileParams = {
    id: string;
    which_repo?: string;
  };

  type signFileWithHashParams = {
    hash: string;
    which_repo?: string;
  };

  type addUrlToFileParams = {
    id: string;
  };

  type addAliasToFileParams = {
    id: string;
  };

  type addHashToFileParams = {
    id: string;
  };

  type addTagToFileParams = {
    id: string;
  };
}
