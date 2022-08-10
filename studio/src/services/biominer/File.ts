// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** Call `/api/v1/files/:id` to fetch the file. GET /api/v1/files/${param0} */
export async function getFile(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getFileParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<API.File>(`/api/v1/files/${param0}`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/files/:id` to sign the file and get the downloading link. POST /api/v1/files/${param0} */
export async function signFile(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.signFileParams,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/files/${param0}`, {
    method: 'POST',
    params: {
      ...queryParams,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/files/:id` to sign the file and get the downloading link. POST /api/v1/files/hash/${param0} */
export async function signFileWithHash(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.signFileWithHashParams,
  options?: { [key: string]: any },
) {
  const { hash: param0, ...queryParams } = params;
  return request<any>(`/api/v1/files/hash/${param0}`, {
    method: 'POST',
    params: {
      ...queryParams,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/files/:id/url` to add url for the file. PUT /api/v1/files/${param0}/url */
export async function addUrlToFile(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.addUrlToFileParams,
  body: API.AddFileUrl,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/files/${param0}/url`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/files/:id/alias` to add alias for the file. PUT /api/v1/files/${param0}/alias */
export async function addAliasToFile(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.addAliasToFileParams,
  body: API.AddFileAlias,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/files/${param0}/alias`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/files/:id/hash` to add hash for the file. PUT /api/v1/files/${param0}/hash */
export async function addHashToFile(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.addHashToFileParams,
  body: API.AddFileHash,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/files/${param0}/hash`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}

/** Call `/api/v1/files/:id/tag` to add tag for the file. PUT /api/v1/files/${param0}/tag */
export async function addTagToFile(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.addTagToFileParams,
  body: API.AddFileTag,
  options?: { [key: string]: any },
) {
  const { id: param0, ...queryParams } = params;
  return request<any>(`/api/v1/files/${param0}/tag`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}
