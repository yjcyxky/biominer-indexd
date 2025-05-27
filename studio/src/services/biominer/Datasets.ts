// @ts-ignore
/* eslint-disable */
import { request } from 'umi';

/** Call `/api/v1/datasets` to get the datasets. GET /api/v1/datasets */
export async function getDatasets(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDatasetsParams,
  options?: { [key: string]: any },
) {
  return request<API.DatasetsResponse>('/api/v1/datasets', {
    method: 'GET',
    params: {
      ...params,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/datasets/:key/data` to get the dataset data. GET /api/v1/datasets/${param0}/data */
export async function getDatasetData(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDatasetDataParams,
  options?: { [key: string]: any },
) {
  const { key: param0, ...queryParams } = params;
  return request<API.DatasetDataResponse>(`/api/v1/datasets/${param0}/data`, {
    method: 'GET',
    params: {
      ...queryParams,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/datasets` to get the datasets. GET /api/v1/datasets/${param0}/data-dictionary */
export async function getDataDictionary(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDataDictionaryParams,
  options?: { [key: string]: any },
) {
  const { key: param0, ...queryParams } = params;
  return request<API.DataDictionary>(`/api/v1/datasets/${param0}/data-dictionary`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}
