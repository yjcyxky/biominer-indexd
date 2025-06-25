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

/** [Deprecated] Call `/api/v1/datasets/:key/:version/data` to get the dataset data. GET /api/v1/datasets/${param0}/${param1}/data */
export async function getDatasetData(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDatasetDataParams,
  options?: { [key: string]: any },
) {
  const { key: param0, version: param1, ...queryParams } = params;
  return request<API.DatasetDataResponse>(`/api/v1/datasets/${param0}/${param1}/data`, {
    method: 'GET',
    params: {
      ...queryParams,
    },
    ...(options || {}),
  });
}

/** Call `/api/v1/datasets/:key/:version/data-dictionary` to get the dataset data dictionary. GET /api/v1/datasets/${param0}/${param1}/data-dictionary */
export async function getDataDictionary(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDataDictionaryParams,
  options?: { [key: string]: any },
) {
  const { key: param0, version: param1, ...queryParams } = params;
  return request<API.DataDictionary>(`/api/v1/datasets/${param0}/${param1}/data-dictionary`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/datasets/:key/:version/data-with-query-plan` to get the dataset data with query plan. GET /api/v1/datasets/${param0}/${param1}/data-with-query-plan */
export async function getDatasetDataWithQueryPlan(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDatasetDataWithQueryPlanParams,
  options?: { [key: string]: any },
) {
  const { key: param0, version: param1, ...queryParams } = params;
  return request<API.DatasetDataResponse>(
    `/api/v1/datasets/${param0}/${param1}/data-with-query-plan`,
    {
      method: 'GET',
      params: {
        ...queryParams,
      },
      ...(options || {}),
    },
  );
}

/** Call `/api/v1/datasets/:key/:version/datafiles/dictionaries` to get the dataset data dictionaries. GET /api/v1/datasets/${param0}/${param1}/datafile-tables */
export async function getDatafileTables(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDatafileTablesParams,
  options?: { [key: string]: any },
) {
  const { key: param0, version: param1, ...queryParams } = params;
  return request<API.DataFileTable[]>(`/api/v1/datasets/${param0}/${param1}/datafile-tables`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/datasets/:key/:version/datafiles` to get the dataset datafiles. GET /api/v1/datasets/${param0}/${param1}/datafiles */
export async function getDatafiles(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDatafilesParams,
  options?: { [key: string]: any },
) {
  const { key: param0, version: param1, ...queryParams } = params;
  return request<Record<string, any>[]>(`/api/v1/datasets/${param0}/${param1}/datafiles`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/datasets/:key/:version/license` to get the dataset license. GET /api/v1/datasets/${param0}/${param1}/license */
export async function getDatasetLicense(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDatasetLicenseParams,
  options?: { [key: string]: any },
) {
  const { key: param0, version: param1, ...queryParams } = params;
  return request<string>(`/api/v1/datasets/${param0}/${param1}/license`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}

/** Call `/api/v1/datasets/:key/:version/readme` to get the dataset readme. GET /api/v1/datasets/${param0}/${param1}/readme */
export async function getDatasetReadme(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getDatasetReadmeParams,
  options?: { [key: string]: any },
) {
  const { key: param0, version: param1, ...queryParams } = params;
  return request<string>(`/api/v1/datasets/${param0}/${param1}/readme`, {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}
