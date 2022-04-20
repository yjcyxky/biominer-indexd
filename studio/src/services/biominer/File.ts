// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** 此处后端没有提供注释 PUT /api/v1/files/${param0}/url */
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

/** 此处后端没有提供注释 PUT /api/v1/files/${param0}/alias */
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

/** 此处后端没有提供注释 PUT /api/v1/files/${param0}/hash */
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

/** 此处后端没有提供注释 PUT /api/v1/files/${param0}/tag */
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
