// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** 此处后端没有提供注释 GET /api/v1/files */
export async function getApiV1Files(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getApiV1FilesParams,
  options?: { [key: string]: any },
) {
  return request<API.FilePage>('/api/v1/files', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}
