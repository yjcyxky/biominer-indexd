// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** 此处后端没有提供注释 GET /api/v1/files/tags */
export async function getTags(options?: { [key: string]: any }) {
  return request<API.FileTags>('/api/v1/files/tags', {
    method: 'GET',
    ...(options || {}),
  });
}
