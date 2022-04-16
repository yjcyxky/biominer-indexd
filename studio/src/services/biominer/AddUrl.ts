// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** 此处后端没有提供注释 PUT /api/v1/files/${param0}/url */
export async function putUrl(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.putUrlParams,
  body: API.PutFileUrl,
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
