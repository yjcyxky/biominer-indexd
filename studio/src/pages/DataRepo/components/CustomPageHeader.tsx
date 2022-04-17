import { ReactNode } from 'react';
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
  enableSearch: boolean;
  setEnableSearch: (enableSearch: boolean) => void;
};

const CustomPageHeader: React.FC<PageHeaderProps> = ({ enableSearch, setEnableSearch }) => (
  <PageHeader
    title="BioMiner Data Repo"
    className="datarepo-page-header"
    subTitle="For Managing and Mining Cancer Associated Omics Data."
    tags={<Tag color="blue">Running</Tag>}
    extra={[
      <Button
        icon={<FileSearchOutlined />}
        onClick={() => {
          setEnableSearch(enableSearch);
        }}
        key="3"
        type="primary"
      >
        {enableSearch ? 'Hide Query' : 'Show Query'}
      </Button>,
      <Button icon={<ImportOutlined />} key="2" type="primary">
        Batch Import
      </Button>,
      <Button icon={<ShareAltOutlined />} key="1" type="primary">
        Share
      </Button>,
    ]}
    avatar={{ src: require('@/assets/images/database.png'), shape: 'square' }}
  >
    <Content
      extraContent={
        <DataStatisticsCard
          totalSize={1102893}
          numOfFiles={6000}
          version={'v20220406'}
        ></DataStatisticsCard>
      }
    >
      {HeaderContent}
    </Content>
  </PageHeader>
);

export default CustomPageHeader;
