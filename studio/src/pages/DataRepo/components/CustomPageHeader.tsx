import React, { ReactNode } from 'react';
import { useIntl } from 'umi';
import { FileSearchOutlined, ImportOutlined, ShareAltOutlined } from '@ant-design/icons';
import { Tag, Button, Row, PageHeader } from 'antd';
import HeaderContent from './HeaderContent';
import DataStatisticsCard from './DataStatisticsCard';

const Content = ({ children, extraContent }: { children: ReactNode; extraContent: ReactNode }) => (
  <Row>
    <div style={{ flex: 1 }}>{children}</div>
    <div className="image">{extraContent}</div>
  </Row>
);

export type PageHeaderProps = {
  fileStat: API.FileStatResponse;
  enableSearch: boolean;
  setEnableSearch: (enableSearch: boolean) => void;
};

const CustomPageHeader: React.FC<PageHeaderProps> = (props: PageHeaderProps) => {
  const intl = useIntl();

  return (
    <PageHeader
      title={intl.formatMessage({
        id: 'data-repo.custom-page-header.title',
        defaultMessage: 'BioMiner DataRepo',
      })}
      className="datarepo-page-header"
      subTitle={intl.formatMessage({
        id: 'data-repo.custom-page-header.subtitle',
        defaultMessage: 'For Managing and Mining Cancer Associated Omics Data.',
      })}
      tags={<Tag color="blue">{props.fileStat.registry_id}</Tag>}
      extra={[
        <Button
          icon={<FileSearchOutlined />}
          onClick={() => {
            props.setEnableSearch(props.enableSearch);
          }}
          key="3"
          type="primary"
        >
          {props.enableSearch
            ? intl.formatMessage({
                id: 'data-repo.custom-page-header.hideQuery',
                defaultMessage: 'Hide Query',
              })
            : intl.formatMessage({
                id: 'data-repo.custom-page-header.showQuery',
                defaultMessage: 'Show Query',
              })}
        </Button>,
        <Button icon={<ImportOutlined />} disabled key="2" type="primary">
          {intl.formatMessage({
            id: 'data-repo.custom-page-header.batchImport',
            defaultMessage: 'Batch Import',
          })}
        </Button>,
        <Button icon={<ShareAltOutlined />} disabled key="1" type="primary">
          {intl.formatMessage({
            id: 'data-repo.custom-page-header.share',
            defaultMessage: 'Share',
          })}
        </Button>,
      ]}
      avatar={{ src: require('@/assets/images/database.png'), shape: 'square' }}
    >
      <Content
        extraContent={
          <DataStatisticsCard
            totalSize={props.fileStat.total_size}
            numOfFiles={props.fileStat.num_of_files}
            version={props.fileStat.version}
          ></DataStatisticsCard>
        }
      >
        <HeaderContent></HeaderContent>
      </Content>
    </PageHeader>
  );
};

export default CustomPageHeader;
