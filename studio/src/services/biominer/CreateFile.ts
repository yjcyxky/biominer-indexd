// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** 此处后端没有提供注释 POST /api/v1/files */
export async function postApiV1Files(body: API.CreateFile, options?: { [key: string]: any }) {
  return request<any>('/api/v1/files', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}
