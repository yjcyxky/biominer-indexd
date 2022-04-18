// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** 此处后端没有提供注释 GET /api/v1/files/stat */
export async function getStat(options?: { [key: string]: any }) {
  return request<API.FileStat>('/api/v1/files/stat', {
    method: 'GET',
    ...(options || {}),
  });
}
