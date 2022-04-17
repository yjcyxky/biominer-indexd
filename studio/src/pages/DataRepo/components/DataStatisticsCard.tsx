import { Space } from 'antd';
import { StatisticCard } from '@ant-design/pro-card';
const { Statistic } = StatisticCard;

export type DataStatisticsCardProps = {
  totalSize: number;
  version: string;
  numOfFiles: number;
};

const DataStatisticsCard: React.FC<DataStatisticsCardProps> = ({
  totalSize,
  version,
  numOfFiles,
}) => (
  <StatisticCard
    statistic={{
      value: totalSize,
      suffix: 'GB',
      description: (
        <Space>
          <Statistic title="Version" value={version} />
          <Statistic title="Files" value={numOfFiles} />
        </Space>
      ),
    }}
    style={{ width: 268, padding: '0px', textAlign: 'right' }}
  />
);

export default DataStatisticsCard;
