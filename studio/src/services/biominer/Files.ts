// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** Call `/api/v1/files` with query params to fetch files. GET /api/v1/files */
export async function fetchFiles(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchFilesParams,
  options?: { [key: string]: any },
) {
  return request<API.RecordResponse>('/api/v1/files', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/files` to create a file instance. POST /api/v1/files */
export async function createFile(body: API.CreateFile, options?: { [key: string]: any }) {
  return request<any>('/api/v1/files', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/files/tags` to fetch all tags. GET /api/v1/files/tags */
export async function getTags(options?: { [key: string]: any }) {
  return request<API.FileTagsResponse>('/api/v1/files/tags', {
    method: 'GET',
    ...(options || {}),
  });
}

/** Call `/api/v1/files/stat` to get the statistics data. GET /api/v1/files/stat */
export async function getFileStat(options?: { [key: string]: any }) {
  return request<API.FileStatResponse>('/api/v1/files/stat', {
    method: 'GET',
    ...(options || {}),
  });
}
