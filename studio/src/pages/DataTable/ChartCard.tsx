import React, { useEffect, useRef } from 'react';
import { Card, Button, Empty, Space, Tooltip, Row } from 'antd';
import { CloseOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { Pie, Bar } from '@ant-design/plots';

import './ChartCard.less';

interface ChartCardProps {
    field: API.DataDictionaryField;
    data: API.DatasetDataResponse['records'];
    onClose?: () => void;
    className?: string;
    resize?: () => void;
}

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

export const getRecommendedChartType = (field: API.DataDictionaryField): 'pie' | 'bar' | 'unsupported' => {
    if (field.data_type === 'BOOLEAN') return 'pie';
    if (field.data_type === 'STRING') {
        const count = field.allowed_values?.length || 0;
        if (count <= 6) return 'pie';
        return 'unsupported';
    }
    if (field.data_type === 'NUMBER') {
        return 'unsupported';
    }
    return 'unsupported';
};

const ChartCard: React.FC<ChartCardProps> = ({ field, data, onClose, className, resize }) => {
    const plotData = buildPlotData(data, field.key);
    const chartType = getRecommendedChartType(field);

    const chartRef = useRef<any>(null);        // 获取图表组件实例
    const containerRef = useRef<HTMLDivElement>(null); // 监听容器尺寸

    // 监听自身尺寸变化以触发图表重绘
    useEffect(() => {
        const observer = new ResizeObserver(() => {
            if (chartRef.current?.chart?.forceFit) {
                chartRef.current.chart.forceFit(); // 对于 Ant Design Plots 图表实例
            } else {
                window.dispatchEvent(new Event('resize')); // fallback
            }

            resize?.();
        });

        if (containerRef.current) {
            observer.observe(containerRef.current);
        }

        return () => {
            observer.disconnect();
        };
    }, []);

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
                    ref={chartRef}
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
                    ref={chartRef}
                />
            );
        }

        return <Empty description={`Unsupported ${field.data_type} field`} className="chart-empty" />;
    };

    return (
        <Row className="chart-card">
            <Card
                size="small"
                title={<div className="chart-drag-handle">{field.name}</div>}
                extra={
                    <Space size={0}>
                        <Tooltip title={field.description}>
                            <Button type="text" size="small" icon={<InfoCircleOutlined />} />
                        </Tooltip>
                        {onClose && <Button type="text" size="small" icon={<CloseOutlined />} onClick={onClose} />}
                    </Space>
                }
                className={className}
                ref={containerRef}
            >
                {renderVisualization()}
            </Card>
        </Row>
    );
};

export default ChartCard;
