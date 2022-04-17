import { CopyFilled, DownloadOutlined } from '@ant-design/icons';
import { Button, message, Drawer, Typography, Divider, Tag, Row, Col } from 'antd';
import React, { useState, useRef } from 'react';
import { useIntl, FormattedMessage } from 'umi';
import { PageContainer, FooterToolbar } from '@ant-design/pro-layout';
import type { ProColumns, ActionType } from '@ant-design/pro-table';
import type { SortOrder } from 'antd/lib/table/interface';
import ProTable from '@ant-design/pro-table';
import type { ProDescriptionsItemProps } from '@ant-design/pro-descriptions';
import ProDescriptions from '@ant-design/pro-descriptions';
import CustomPageHeader from './components/CustomPageHeader';
import biominerAPI from '@/services/biominer';
import './index.less';

const FileList: React.FC = () => {
  /**
   * @en-US Pop-up window of new window
   * @zh-CN 新建窗口的弹窗
   *  */
  const [showDetail, setShowDetail] = useState<boolean>(false);
  const [enableSearch, setEnableSearch] = useState<boolean>(false);

  const [params, setParams] = useState<API.getApiV1FilesParams>({});

  const actionRef = useRef<ActionType>();
  const [currentRow, setCurrentRow] = useState<API.File>();
  const [selectedRowsState, setSelectedRows] = useState<API.File[]>([]);

  const downloadSelectedFiles = (selectedRowsState: API.File[]) => {
    if (selectedRowsState.length === 0) {
      message.info('Please select the file you want to download');
      return;
    } else {
    }
  };

  /**
   * @en-US International configuration
   * @zh-CN 国际化配置
   * */
  const intl = useIntl();

  const ListFiles = async (
    params: API.getApiV1FilesParams & { current?: number; pageSize?: number },
    sort: Record<string, SortOrder>,
    filter: Record<string, React.ReactText[] | null>,
  ) => {
    let { current, pageSize, ...newParams } = params;
    newParams['page'] = current ? current : 1;
    newParams['page_size'] = pageSize ? pageSize : 10;
    let response = await biominerAPI.ListFiles.getApiV1Files(newParams);
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
      title: <FormattedMessage id="pages.dataRepo.guid" defaultMessage="GUID" />,
      dataIndex: 'guid',
      fixed: 'left',
      copyable: true,
      width: 200,
      tip: 'The guid is the unique key',
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
      dataIndex: 'hashes',
      align: 'center',
      width: 200,
      render: (dom, entity) => {
        return entity.hashes
          ?.filter((hash) => hash.hash_type === 'md5')
          .map((hash) => {
            return <Typography.Text copyable>{hash.hash}</Typography.Text>;
          });
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.filename" defaultMessage="File Name" />,
      dataIndex: 'filename',
      copyable: true,
      width: 200,
      align: 'center',
      valueType: 'textarea',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.filesize" defaultMessage="File Size" />,
      dataIndex: 'size',
      align: 'center',
      width: 150,
      sorter: true,
      hideInSearch: true,
      renderText: (val: string) =>
        `${val} ${intl.formatMessage({
          id: 'pages.dataRepo.bytes',
          defaultMessage: ' B',
        })}`,
    },
    {
      title: <FormattedMessage id="pages.dataRepo.status" defaultMessage="Status" />,
      dataIndex: 'status',
      width: 100,
      align: 'center',
      hideInForm: true,
      valueEnum: {
        0: {
          text: <FormattedMessage id="pages.dataRepo.status.pending" defaultMessage="Pending" />,
          status: 'pending',
        },
        1: {
          text: (
            <FormattedMessage id="pages.dataRepo.status.processing" defaultMessage="Processing" />
          ),
          status: 'processing',
        },
        2: {
          text: (
            <FormattedMessage id="pages.dataRepo.status.validated" defaultMessage="Validated" />
          ),
          status: 'validated',
        },
        3: {
          text: <FormattedMessage id="pages.dataRepo.status.failed" defaultMessage="Failed" />,
          status: 'failed',
        },
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.createdAt" defaultMessage="Created At" />,
      sorter: true,
      align: 'center',
      width: 200,
      hideInSearch: true,
      dataIndex: 'created_at',
      valueType: 'dateTime',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.updatedAt" defaultMessage="Updated At" />,
      dataIndex: 'updated_at',
      width: 200,
      align: 'center',
      hideInSearch: true,
      valueType: 'dateTime',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.baseid" defaultMessage="BaseId" />,
      dataIndex: 'baseid',
      width: 200,
      copyable: true,
      align: 'center',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.revision" defaultMessage="Revision" />,
      hideInSearch: true,
      hideInTable: true,
      width: 100,
      align: 'center',
      dataIndex: 'rev',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.version" defaultMessage="Version" />,
      hideInSearch: true,
      width: 100,
      align: 'center',
      dataIndex: 'version',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.uploader" defaultMessage="Uploader" />,
      dataIndex: 'uploader',
      width: 100,
      align: 'center',
    },
    {
      title: <FormattedMessage id="pages.dataRepo.alias" defaultMessage="Alias" />,
      dataIndex: 'aliases',
      width: 200,
      copyable: true,
      align: 'center',
      render: (dom, entity) => {
        return entity.aliases?.map((alias) => {
          return <span>{alias.name};</span>;
        });
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.containAlias" defaultMessage="Contain Alias?" />,
      dataIndex: 'contain_alias',
      width: 100,
      align: 'center',
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
      align: 'center',
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
      title: <FormattedMessage id="pages.dataRepo.hashes" defaultMessage="Hashes" />,
      dataIndex: 'hashes',
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
                <Col style={{ marginBottom: '5px' }}>
                  <Tag>{hash.hash_type}</Tag>
                  <span>{hash.hash}</span>
                </Col>
              );
            })}
          </Row>
        );
      },
    },
    {
      title: <FormattedMessage id="pages.dataRepo.aliases" defaultMessage="Aliases" />,
      dataIndex: 'aliases',
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
                <Col style={{ marginBottom: '5px' }}>
                  <Tag>{alias.name}</Tag>
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
                <Col style={{ marginBottom: '5px' }}>
                  <Tag>{url.uploader}</Tag>
                  <span>{url.url}</span>
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
      width: 200,
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
                setCurrentRow(entity);
                setEnableSearch(true);
              }}
            >
              <FormattedMessage id="pages.dataRepo.search" defaultMessage="Search" />
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
          setEnableSearch={() => {
            setEnableSearch(!enableSearch);
          }}
          enableSearch={enableSearch}
        ></CustomPageHeader>
      }
    >
      <ProTable<API.File, API.getApiV1FilesParams>
        scroll={{ x: 1500 }}
        headerTitle={intl.formatMessage({
          id: 'pages.dataRepo.title',
          defaultMessage: 'Data Repo',
        })}
        actionRef={actionRef}
        rowKey="guid"
        search={
          enableSearch
            ? {
                labelWidth: 120,
              }
            : false
        }
        toolBarRender={() => []}
        params={params}
        beforeSearchSubmit={(params: API.getApiV1FilesParams) => {
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
                {selectedRowsState.reduce((pre, item) => pre + item.size!, 0)}{' '}
                <FormattedMessage id="pages.dataRepo.bytes" defaultMessage="B" />
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
        width={800}
        visible={showDetail}
        onClose={() => {
          setCurrentRow(undefined);
          setShowDetail(false);
        }}
        closable={false}
      >
        {currentRow?.guid && (
          <ProDescriptions<API.File>
            column={2}
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
    </PageContainer>
  );
};

export default FileList;
