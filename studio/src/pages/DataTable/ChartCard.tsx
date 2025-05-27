import React from 'react';
import { Card, Button, Empty, Space, Tooltip } from 'antd';
import { CloseCircleFilled, CloseOutlined, InfoCircleFilled, InfoCircleOutlined } from '@ant-design/icons';
import { Pie, Bar } from '@ant-design/plots';

interface ChartCardProps {
    field: API.DataDictionaryField;
    data: API.DatasetDataResponse['records'];
    onClose: () => void;
}

// 构建频率统计数据
const buildPlotData = (records: API.DatasetDataResponse['records'], key: string): { value: string; count: number }[] => {
    const freqMap = new Map<string, number>();
    records.forEach((r) => {
        const val = r[key];
        if (val !== undefined && val !== null) {
            freqMap.set(val, (freqMap.get(val) || 0) + 1);
        }
    });
    return Array.from(freqMap.entries()).map(([value, count]) => ({ value, count }));
};

// 推荐图类型逻辑
const getRecommendedChartType = (field: API.DataDictionaryField): 'pie' | 'bar' | 'unsupported' => {
    if (field.data_type === 'BOOLEAN') return 'pie';
    if (field.data_type === 'STRING') {
        const count = field.allowed_values?.length || 0;
        if (count <= 6) return 'pie';
        // if (count <= 20) return 'bar';
        return 'unsupported';
    }
    if (field.data_type === 'NUMBER') {
        // TODO: Support histogram
        // return 'bar'; // 可扩展 histogram
        return 'unsupported';
    }
    return 'unsupported';
};

const ChartCard: React.FC<ChartCardProps> = ({ field, data, onClose }) => {
    const plotData = buildPlotData(data, field.key);
    const chartType = getRecommendedChartType(field);

    const renderVisualization = () => {
        if (!field.allowed_values && chartType !== 'bar') {
            return <Empty description={`Unsupported ${field.data_type} field`} />;
        }

        if (chartType === 'pie') {
            return (
                <Pie
                    autoFit
                    data={plotData}
                    angleField="count"
                    colorField="value"
                    radius={0.8}
                    label={{ type: 'spider', labelHeight: 28 }}
                />
            );
        }

        if (chartType === 'bar') {
            return (
                <Bar
                    autoFit
                    data={plotData}
                    xField="count"
                    yField="value"
                    seriesField="value"
                    legend={false}
                />
            );
        }

        return <Empty description={`Unsupported ${field.data_type} field`} style={{ margin: '20%' }} />;
    };

    return (
        <Card
            size="small"
            title={<div className="chart-drag-handle">{field.name}</div>}
            extra={
                <Space size={0}>
                    <Tooltip title={field.description}>
                        <Button type="text" size="small" icon={<InfoCircleOutlined />} />
                    </Tooltip>
                    <Button type="text" size="small" icon={<CloseOutlined />} onClick={onClose} />
                </Space>
            }
            style={{ height: '100%', overflow: 'hidden' }}
            bodyStyle={{ height: 'calc(100% - 40px)', overflow: 'auto' }}
        >
            {renderVisualization()}
        </Card>
    );
};

export default ChartCard;
