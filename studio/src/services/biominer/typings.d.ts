declare namespace API {
  type Alias = {
    id: number;
    name: string;
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

  type Hash = {
    id: number;
    hash_type: string;
    hash: string;
  };

  type PostFile = {
    filename?: string;
    uploader?: string;
    hash: string;
    size: number;
  };

  type PutFileAlias = {
    alias: string;
  };

  type PutFileHash = {
    hash: string;
  };

  type PutFileUrl = {
    url: string;
    status?: string;
    uploader?: string;
  };

  type StatusResponse = {
    msg: string;
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
    contain_alias?: boolean;
    contain_url?: boolean;
  };

  type putUrlParams = {
    id: string;
  };

  type putAliasParams = {
    id: string;
  };

  type putHashParams = {
    id: string;
  };
}
