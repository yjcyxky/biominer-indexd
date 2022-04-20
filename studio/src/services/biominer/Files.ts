// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** 此处后端没有提供注释 GET /api/v1/files */
export async function fetchFiles(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.fetchFilesParams,
  options?: { [key: string]: any },
) {
  return request<API.FilePageResponse>('/api/v1/files', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** 此处后端没有提供注释 POST /api/v1/files */
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

/** 此处后端没有提供注释 GET /api/v1/files/tags */
export async function getTags(options?: { [key: string]: any }) {
  return request<API.FileTagsResponse>('/api/v1/files/tags', {
    method: 'GET',
    ...(options || {}),
  });
}

/** 此处后端没有提供注释 GET /api/v1/files/stat */
export async function getFileStat(options?: { [key: string]: any }) {
  return request<API.FileStatResponse>('/api/v1/files/stat', {
    method: 'GET',
    ...(options || {}),
  });
}
