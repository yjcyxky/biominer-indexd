// import {
//   default as axios,
//   AxiosRequestConfig,
//   AxiosRequestHeaders,
// } from 'axios';
import { map } from 'lodash';
import { DownloadOutlined } from '@ant-design/icons';
import { Button, message, Drawer, Typography, Divider, Tag, Row, Col, notification, Spin } from 'antd';
import { LockOutlined, UnlockOutlined } from '@ant-design/icons';
import React, { useState, useRef, useEffect } from 'react';
import { useIntl, FormattedMessage } from 'umi';
import { PageContainer, FooterToolbar } from '@ant-design/pro-layout';
import type { ProColumns, ActionType } from '@ant-design/pro-table';
import type { SortOrder } from 'antd/lib/table/interface';
import ProTable from '@ant-design/pro-table';
import type { ProDescriptionsItemProps } from '@ant-design/pro-descriptions';
import type { ProFormInstance } from '@ant-design/pro-form';
import ProDescriptions from '@ant-design/pro-descriptions';
import CustomPageHeader from './components/CustomPageHeader';
import biominerAPI from '@/services/biominer';
import './index.less';

const isValidGuid = (guid: string | null) => {
  if (guid) {
    if (guid.length <= 36) {
      return false;
    }
    const regex =
      /^biominer.fudan-pgx\/[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;
    return regex.test(guid);
  } else {
    return false;
  }
};

const determinRepo = (url?: API.URL) => {
  if (typeof url !== 'undefined') {
    return url.url.split(':')[0]
  } else {
    return 'node'
  }
}

const assign: any = (arr: Object[]) => {
  let m: any = {};
  arr.forEach(item => {
    m = Object.assign(m, item)
  });
  return m;
}

type SignData = {
  headers: Object;
  data: Object;
  baseurl: string;
  method: string;
  params: Object;
};

const FileList: React.FC = () => {
  /**
   * @en-US Pop-up window of new window
   * @zh-CN 新建窗口的弹窗
   *  */
  const formRef = useRef<ProFormInstance>();
  const [showDetail, setShowDetail] = useState<boolean>(false);
  const [enableSearch, setEnableSearch] = useState<boolean>(false);

  const [fileStat, setFileStat] = useState<API.FileStatResponse>({
    total_size: -1,
    version: '',
    num_of_files: -1,
    num_of_baseid: -1,
    registry_id: '',
  });
  const [params, setParams] = useState<API.fetchFilesParams>({});

  const actionRef = useRef<ActionType>();
  const [currentRow, setCurrentRow] = useState<API.File>();
  const [selectedRowsState, setSelectedRows] = useState<API.File[]>([]);
  const [signData, setSignData] = useState<SignData>({
    method: 'POST',
    baseurl: '',
    data: {},
    params: {},
    headers: {}
  });

  // ?guid=biominer.fudan-pgx/1aea5c61-4a83-45a8-852e-dfd57a89b388
  const search = window.location.search;
  let queryParams = new URLSearchParams(search);
  let guid_query: string | null = queryParams.get('guid');

  // pathname: /biominer.fudan-pgx/1aea5c61-4a83-45a8-852e-dfd57a89b388
  let guid_path = window.location.pathname.replace(/^\//, '');
  // guid_path always exists, so we need to use guid_query firstly.
  let guid: string | null = guid_query ? guid_query : guid_path;

  const downloadSelectedFiles = (selectedRowsState: API.File[]) => {
    if (selectedRowsState.length === 0) {
      message.info('Please select the file you want to download');
      return;
    } else {
      // TODO: download selected files
      message.info('Comming soon...');
    }
  };

  const downloadFile = (apiSignData: API.SignData, filename: string, repo: string) => {
    // console.log("Download selected file: ", apiSignData);

    const customHeaders: Object[] = map(apiSignData.header, (item: string) => {
      const m: Object = {}
      const itemLst = item.split(":")
      m[itemLst[0]] = itemLst[1].trim()
      return m
    });

    const data: Object[] = map(apiSignData.data, (item: string) => {
      const m: Object = {}
      const itemLst = item.split("=")
      m[itemLst[0]] = itemLst[1].trim()
      return m
    });

    const params: Object[] = map(apiSignData.params, (item: string) => {
      const m: Object = {}
      const itemLst = item.split("=")
      m[itemLst[0]] = itemLst[1].trim()
      return m
    });

    setSignData({
      headers: assign(customHeaders),
      data: assign(data),
      params: assign(params),
      baseurl: apiSignData.baseurl,
      method: apiSignData.method
    })

    if (apiSignData.method === 'GET' && repo === 'gsa') {
      message.info(`Downloading ${filename}, please wait a minute.`);
      const paramsStr = new URLSearchParams(assign(params)).toString();
      const link = `${apiSignData.baseurl}?${paramsStr}`;
      const downloadAnchorNode = document.createElement('a');
      downloadAnchorNode.download = filename;
      downloadAnchorNode.style.display = 'none';
      downloadAnchorNode.setAttribute('href', link);
      downloadAnchorNode.setAttribute('target', '_blank');
      document.body.appendChild(downloadAnchorNode);
      downloadAnchorNode.click();
      downloadAnchorNode.remove();
      return;
    } else if (apiSignData.method === 'POST' && repo === 'node') {
      const downloadAnchorNode = document.querySelector('#nodeDownloadForm') as HTMLFormElement;
      if (downloadAnchorNode) {
        downloadAnchorNode.submit();
        return;
      }
    }

    notification.warn({
      message: 'Warning',
      description: <p>Cannot be downloaded via browser, you can use <a href='http://indexd.org/about' target='_blank'>biominer-aget</a>.</p>
    })
  }

  const downloadSelectedFile = (entity: API.File) => {
    console.log('Download file ', entity);
    // TODO: How to select prefered repo?
    const url = entity.urls ? entity.urls[0] : undefined
    const which_repo = determinRepo(url);
    const id = entity.guid.split('/')[1];
    biominerAPI.File.signFile({
      id: id,
      which_repo: which_repo
    }).then((response: API.SignResponse) => {
      message.info(`Downloading the file ${entity.filename}, please wait a moment...`)
      downloadFile(response.sign, response.filename, which_repo);
    }).catch(error => {
      console.log("Download selected file(error): ", error);
      message.error(`Cannot download ${entity.filename}, ${error}`)
    })
  }

  const setQueryParams = (guid: string | null) => {
    if (guid && isValidGuid(guid)) {
      setParams({
        guid: guid,
      });
      formRef.current?.setFieldsValue({
        guid: guid,
      });
    }
  };

  useEffect(() => {
    setQueryParams(guid);
  }, [guid]);

  useEffect(() => {
    // Avoid request frequently, only request when the data is empty
    if (fileStat.total_size === -1 || fileStat.num_of_files === -1) {
      biominerAPI.Files.getFileStat()
        .then((res) => {
          setFileStat(res);
        })
        .catch((err) => {
          console.log(err);
        });
    }
  });

  /**
   * @en-US International configuration
   * @zh-CN 国际化配置
   * */
  const intl = useIntl();

  const ListFiles = async (
    params: API.fetchFilesParams & { current?: number; pageSize?: number },
    sort: Record<string, SortOrder>,
    filter: Record<string, React.ReactText[] | null>,
  ) => {
    let { current, pageSize, ...newParams } = params;
    newParams['page'] = current ? current : 1;
    newParams['page_size'] = pageSize ? pageSize : 10;
    let response = await biominerAPI.Files.fetchFiles(newParams);
    return {
      data: response.records,
      total: response.total,
      success: true,
      current: response.page_no,
      pageSize: response.page_size,
    };
  };

  const columns: ProColumns<API.File>[] = [
    {
      title: <FormattedMessage id="pages.dataRepo.access" defaultMessage="Access" />,
      sorter: false,
      align: 'center',
      width: 120,
      hideInSearch: true,
      dataIndex: 'access',
      valueType: 'textarea',
      render: (dom, entity) => {
        if (entity.access === 'public') {
          return <span><UnlockOutlined /> Public</span>
        } else {
          return <span><LockOutlined /> Controlled</span>
        }
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.guid" defaultMessage="GUID" />,
      dataIndex: 'guid',
      fixed: 'left',
      copyable: true,
      width: 220,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.guidTip',
        defaultMessage: 'A global unique identifier for the file',
      }),
      render: (dom, entity) => {
        return (
          <a
            onClick={() => {
              setCurrentRow(entity);
              setShowDetail(true);
            }}
          >
            {dom}
          </a>
        );
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.md5sum" defaultMessage="MD5SUM" />,
      dataIndex: 'hash',
      align: 'center',
      width: 200,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.md5sumTip',
        defaultMessage: 'The MD5 checksum of the file',
      }),
      render: (dom, entity) => {
        return entity.hashes
          ?.filter((hash) => hash.hash_type === 'md5')
          .map((hash) => {
            return (
              <Typography.Text key={hash.hash} copyable>
                {hash.hash}
              </Typography.Text>
            );
          });
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.filename" defaultMessage="File Name" />,
      dataIndex: 'filename',
      copyable: true,
      width: 250,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.filenameTip',
        defaultMessage: 'The name of the file',
      }),
      align: 'center',
      valueType: 'textarea',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.fieldName" defaultMessage="Field Name" />,
      dataIndex: 'field_name',
      width: 250,
      hideInSetting: true,
      hideInForm: true,
      hideInTable: true,
      hideInDescriptions: true,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.fieldNameTip',
        defaultMessage: 'Which tag the file belongs to',
      }),
      align: 'center',
      valueType: 'textarea',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.fieldValue" defaultMessage="Field Value" />,
      dataIndex: 'field_value',
      width: 250,
      hideInSetting: true,
      hideInForm: true,
      hideInTable: true,
      hideInDescriptions: true,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.fieldValueTip',
        defaultMessage: 'Which tag the file belongs to',
      }),
      align: 'center',
      valueType: 'textarea',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.filesize" defaultMessage="File Size" />,
      dataIndex: 'size',
      align: 'center',
      width: 150,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.filesizeTip',
        defaultMessage: 'The size of the file',
      }),
      sorter: false,
      hideInSearch: true,
      renderText: (val: number) =>
        `${(val / (1024 * 1024 * 1024)).toFixed(3)} ${intl.formatMessage({
          id: 'pages.dataRepo.gigaBytes',
          defaultMessage: 'GB',
        })}`,
    },
    {
      title: <FormattedMessage id="pages.dataRepo.tags" defaultMessage="Tags" />,
      dataIndex: 'tags',
      align: 'center',
      width: 200,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.tagsTip',
        defaultMessage: 'The tags of the file',
      }),
      hideInTable: true,
      hideInForm: true,
      hideInSearch: true,
      hideInSetting: true,
      render: (dom, entity) => {
        return (
          <Row style={{ display: 'flex', flexDirection: 'row', width: '100%' }}>
            {entity.tags?.map((tag) => {
              return (
                <Col
                  key={tag.field_name}
                  span={8}
                  style={{ marginBottom: '5px', marginRight: '10px' }}
                >
                  <Tag>{tag.field_name}</Tag>
                  <span>{tag.field_value}</span>
                </Col>
              );
            })}
          </Row>
        );
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.status" defaultMessage="Status" />,
      dataIndex: 'status',
      width: 100,
      align: 'center',
      hideInForm: true,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.statusTip',
        defaultMessage: 'The status of the file',
      }),
      valueEnum: {
        pending: {
          text: <FormattedMessage id="pages.dataRepo.status.pending" defaultMessage="Pending" />,
        },
        processing: {
          text: (
            <FormattedMessage id="pages.dataRepo.status.processing" defaultMessage="Processing" />
          ),
        },
        validated: {
          text: (
            <FormattedMessage id="pages.dataRepo.status.validated" defaultMessage="Validated" />
          ),
        },
        failed: {
          text: <FormattedMessage id="pages.dataRepo.status.failed" defaultMessage="Failed" />,
        },
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.createdAt" defaultMessage="Created At" />,
      sorter: false,
      align: 'center',
      width: 200,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.createdAtTip',
        defaultMessage: 'The time when the file was created',
      }),
      hideInSearch: true,
      dataIndex: 'created_at',
      valueType: 'dateTime',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.updatedAt" defaultMessage="Updated At" />,
      dataIndex: 'updated_at',
      tip: intl.formatMessage({
        id: 'pages.dataRepo.updatedAtTip',
        defaultMessage: 'The time when the file was updated',
      }),
      width: 200,
      align: 'center',
      hideInSearch: true,
      valueType: 'dateTime',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.baseid" defaultMessage="BaseId" />,
      dataIndex: 'baseid',
      tip: intl.formatMessage({
        id: 'pages.dataRepo.baseidTip',
        defaultMessage: 'The base id of the file',
      }),
      width: 200,
      copyable: true,
      align: 'center',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.revision" defaultMessage="Revision" />,
      hideInSearch: true,
      hideInTable: true,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.revisionTip',
        defaultMessage: 'The revision of the file',
      }),
      width: 100,
      align: 'center',
      dataIndex: 'rev',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.version" defaultMessage="Version" />,
      hideInSearch: true,
      width: 100,
      tip: intl.formatMessage({
        id: 'pages.dataRepo.versionTip',
        defaultMessage: 'The version of the file',
      }),
      align: 'center',
      dataIndex: 'version',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.uploader" defaultMessage="Uploader" />,
      dataIndex: 'uploader',
      tip: intl.formatMessage({
        id: 'pages.dataRepo.uploaderTip',
        defaultMessage: 'The uploader of the file',
      }),
      width: 100,
      align: 'center',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.alias" defaultMessage="Alias" />,
      dataIndex: 'alias',
      tip: intl.formatMessage({
        id: 'pages.dataRepo.aliasTip',
        defaultMessage: 'The alias of the file',
      }),
      width: 200,
      hideInSetting: true,
      hideInDescriptions: true,
      copyable: true,
      align: 'center',
      render: (dom, entity) => {
        let alias = entity.aliases ? entity.aliases[0] : undefined;
        return <span>{alias?.name}</span>;
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.containTag" defaultMessage="Contain Tag?" />,
      dataIndex: 'contain_tag',
      width: 100,
      align: 'center',
      initialValue: 'true',
      hideInSearch: true,
      hideInTable: true,
      hideInDescriptions: true,
      hideInSetting: true,
      valueEnum: {
        true: {
          text: <FormattedMessage id="pages.dataRepo.containTag.true" defaultMessage="Yes" />,
        },
        false: {
          text: <FormattedMessage id="pages.dataRepo.containTag.false" defaultMessage="No" />,
        },
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.containAlias" defaultMessage="Contain Alias?" />,
      dataIndex: 'contain_alias',
      width: 100,
      initialValue: 'true',
      align: 'center',
      hideInSearch: true,
      hideInTable: true,
      hideInDescriptions: true,
      hideInSetting: true,
      valueEnum: {
        true: {
          text: <FormattedMessage id="pages.dataRepo.containAlias.true" defaultMessage="Yes" />,
        },
        false: {
          text: <FormattedMessage id="pages.dataRepo.containAlias.false" defaultMessage="No" />,
        },
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.containURL" defaultMessage="Contain URL?" />,
      dataIndex: 'contain_url',
      width: 100,
      initialValue: 'true',
      align: 'center',
      hideInSearch: true,
      hideInTable: true,
      hideInDescriptions: true,
      hideInSetting: true,
      valueEnum: {
        true: {
          text: <FormattedMessage id="pages.dataRepo.containURL.true" defaultMessage="Yes" />,
        },
        false: {
          text: <FormattedMessage id="pages.dataRepo.containURL.false" defaultMessage="No" />,
        },
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.aliases" defaultMessage="Aliases" />,
      dataIndex: 'aliases',
      tip: intl.formatMessage({
        id: 'pages.dataRepo.aliasesTip',
        defaultMessage: 'The aliases of the file',
      }),
      align: 'center',
      width: 200,
      hideInTable: true,
      hideInForm: true,
      hideInSearch: true,
      hideInSetting: true,
      render: (dom, entity) => {
        return (
          <Row style={{ display: 'flex', flexDirection: 'column', width: '100%' }}>
            {entity.aliases?.map((alias) => {
              return (
                <Col key={alias.name} style={{ marginBottom: '5px' }}>
                  <Tag>{alias.name}</Tag>
                </Col>
              );
            })}
          </Row>
        );
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.hashes" defaultMessage="Hashes" />,
      dataIndex: 'hashes',
      tip: intl.formatMessage({
        id: 'pages.dataRepo.hashesTip',
        defaultMessage: 'hashes of the file',
      }),
      align: 'center',
      width: 200,
      hideInTable: true,
      hideInForm: true,
      hideInSearch: true,
      hideInSetting: true,
      render: (dom, entity) => {
        return (
          <Row style={{ display: 'flex', flexDirection: 'column', width: '100%' }}>
            {entity.hashes?.map((hash) => {
              return (
                <Col key={hash.hash} style={{ marginBottom: '5px' }}>
                  <Tag>{hash.hash_type}</Tag>
                  <Typography.Text key={hash.hash} copyable>
                    {hash.hash}
                  </Typography.Text>
                </Col>
              );
            })}
          </Row>
        );
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.urls" defaultMessage="URLs" />,
      dataIndex: 'urls',
      tip: intl.formatMessage({
        id: 'pages.dataRepo.urlsTip',
        defaultMessage: 'URLs of the file',
      }),
      align: 'center',
      width: 200,
      hideInTable: true,
      hideInForm: true,
      hideInSearch: true,
      hideInSetting: true,
      render: (dom, entity) => {
        return (
          <Row style={{ display: 'flex', flexDirection: 'column', width: '100%' }}>
            {entity.urls?.map((url) => {
              return (
                <Col key={url.url} style={{ marginBottom: '5px' }}>
                  <Tag>{url.uploader}</Tag>
                  <Typography.Text key={url.url} copyable>
                    {url.url}
                  </Typography.Text>
                </Col>
              );
            })}
          </Row>
        );
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.actions" defaultMessage="Actions" />,
      dataIndex: 'actions',
      align: 'center',
      fixed: 'right',
      width: 160,
      valueType: 'option',
      render: (_, entity) => {
        return (
          <>
            <a
              onClick={() => {
                setCurrentRow(entity);
                setShowDetail(true);
              }}
            >
              <FormattedMessage id="pages.dataRepo.view" defaultMessage="View" />
            </a>
            <Divider type="vertical" />
            <a
              onClick={() => {
                downloadSelectedFile(entity);
              }}
            >
              <FormattedMessage id="pages.dataRepo.download" defaultMessage="Download" />
            </a>
          </>
        );
      },
    },
  ];

  return (
    <PageContainer
      className="datarepo"
      header={{
        title: undefined,
      }}
      content={
        <CustomPageHeader
          fileStat={fileStat}
          setEnableSearch={() => {
            setEnableSearch(!enableSearch);
            setQueryParams(guid);
          }}
          enableSearch={enableSearch}
        ></CustomPageHeader>
      }
    >
      <ProTable<API.File, API.fetchFilesParams>
        scroll={{ x: 1500 }}
        pagination={{ position: ['topLeft'] }}
        actionRef={actionRef}
        rowKey="guid"
        search={
          enableSearch
            ? {
              labelWidth: 120,
            }
            : false
        }
        formRef={formRef}
        toolBarRender={() => []}
        params={params}
        beforeSearchSubmit={(params: API.fetchFilesParams) => {
          setParams(params);
        }}
        request={ListFiles}
        columns={columns}
        rowSelection={{
          onChange: (_, selectedRows) => {
            setSelectedRows(selectedRows);
          },
        }}
      />
      {selectedRowsState?.length > 0 && (
        <FooterToolbar
          extra={
            <div>
              <FormattedMessage id="pages.dataRepo.chosen" defaultMessage="Chosen" />{' '}
              <a style={{ fontWeight: 600 }}>{selectedRowsState.length}</a>{' '}
              <FormattedMessage id="pages.dataRepo.items" defaultMessage="Items" />
              &nbsp;&nbsp;
              <span>
                <FormattedMessage id="pages.dataRepo.totalSize" defaultMessage="Total Size" />{' '}
                {(
                  selectedRowsState.reduce((pre, item) => pre + item.size!, 0) /
                  (1024 * 1024 * 1024)
                ).toFixed(3)}{' '}
                <FormattedMessage id="pages.dataRepo.gigaBytes" defaultMessage="GB" />
              </span>
            </div>
          }
        >
          <Button
            type="primary"
            key="primary"
            onClick={() => {
              downloadSelectedFiles(selectedRowsState);
            }}
          >
            <DownloadOutlined />
            <FormattedMessage id="pages.dataRepo.download" defaultMessage="Download" />
          </Button>
        </FooterToolbar>
      )}
      <Drawer
        width={'70%'}
        visible={showDetail}
        onClose={() => {
          setCurrentRow(undefined);
          setShowDetail(false);
        }}
        closable={false}
      >
        {currentRow?.guid && (
          <ProDescriptions<API.File>
            column={1}
            bordered
            title={currentRow?.filename}
            request={async () => ({
              data: currentRow || {},
            })}
            params={{
              id: currentRow?.guid,
            }}
            columns={columns as ProDescriptionsItemProps<API.File>[]}
          />
        )}
      </Drawer>
      <form action={signData.baseurl} method={signData.method} id="nodeDownloadForm">
        {
          Object
            .entries(signData.data)
            .map(([key, value]) => (
              <input hidden name={key} value={value} id={key} key={key} onChange={() => { }} />
            ))
        }
      </form>
    </PageContainer>
  );
};

export default FileList;
