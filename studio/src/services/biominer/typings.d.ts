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
  };

  type CreateFile = {
    filename?: string;
    uploader?: string;
    hash: string;
    size: number;
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

  type FilePage = {
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

  type FileStat = {
    total_size: number;
    num_of_files: number;
    num_of_baseid: number;
    version: string;
    registry_id: string;
  };

  type FileTags = {
    field_names: string[];
  };

  type Hash = {
    id: number;
    hash_type: string;
    hash: string;
  };

  type StatusResponse = {
    msg: string;
  };

  type Tag = {
    id: number;
    field_name: string;
    field_value: string;
  };

  type URL = {
    id: number;
    url: string;
    created_at: number;
    status: string;
    uploader: string;
  };

  type getApiV1FilesParams = {
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

  type putByIdUrlParams = {
    id: string;
  };

  type putByIdAliasParams = {
    id: string;
  };

  type putByIdHashParams = {
    id: string;
  };

  type putByIdTagParams = {
    id: string;
  };
}
