import { Space } from 'antd';
import { StatisticCard } from '@ant-design/pro-card';
const { Statistic } = StatisticCard;
import { useIntl } from 'umi';

export type DataStatisticsCardProps = {
  totalSize: number;
  version: string;
  numOfFiles: number;
};

const DataStatisticsCard: React.FC<DataStatisticsCardProps> = ({
  totalSize,
  version,
  numOfFiles,
}) => {
  const intl = useIntl();
  return (
    <StatisticCard
      statistic={{
        // Bytes --> GB
        value: (totalSize / 1024 / 1024 / 1024).toFixed(3),
        suffix: 'GB',
        description: (
          <Space>
            <Statistic
              title={intl.formatMessage({
                id: 'data-repo.data-statistics-card.version',
                defaultMessage: 'Version',
              })}
              value={version}
            />
            <Statistic
              title={intl.formatMessage({
                id: 'data-repo.data-statistics-card.files',
                defaultMessage: 'Files',
              })}
              value={numOfFiles}
            />
          </Space>
        ),
      }}
      style={{ width: 268, padding: '0px', textAlign: 'right' }}
    />
  );
};

export default DataStatisticsCard;
