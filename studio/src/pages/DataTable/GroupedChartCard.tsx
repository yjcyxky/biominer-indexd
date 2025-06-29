import React, { FC, useEffect, useRef, useState, useMemo } from 'react';
import { Card, Button, Empty, Space, Tooltip, Row, Col, Statistic, Table, Popover, Select, Tag } from 'antd';
import { CloseOutlined, EyeFilled, InfoCircleOutlined, SettingOutlined } from '@ant-design/icons';
import { Pie, Bar, Histogram, Line, Box, Violin, Column, Scatter } from '@ant-design/plots';
import { groupBy, sumBy, meanBy, minBy, maxBy } from 'lodash';
// @ts-ignore
import Plotly from 'plotly.js/dist/plotly';
// @ts-ignore
import createPlotlyComponent from 'react-plotly.js/factory';

const Plot = createPlotlyComponent(Plotly);

import './GroupedChartCard.less';

export const DEFAULT_ID_COLUMN_NAME = 'sample_id';

const fontSize = 20;

type GroupedChartType = 'bar' | 'box' | 'violin' | 'scatter' | 'histogram' | 'summary' | 'kmplot' | 'unsupported';

const isAllNull = (data: any[]) => {
    return data.every(d => d === null || d === undefined || d === '');
}

const groupedChartTypeOptions = [
    { label: 'Bar Chart', value: 'bar' },
    { label: 'Box Plot', value: 'box' },
    { label: 'Violin Plot', value: 'violin' },
    { label: 'Scatter Plot', value: 'scatter' },
    { label: 'Histogram', value: 'histogram' },
    { label: 'KM Plot', value: 'kmplot' },
    { label: 'Summary', value: 'summary' },
    { label: 'Unsupported', value: 'unsupported' },
];

interface GroupedChartCardProps {
    fields: API.DataDictionaryField[];
    allFields: API.DataDictionaryField[];
    data: API.DatasetDataResponse['records'];
    selectedColumns?: string[];
    groupByField: string;
    onClose?: () => void;
    className?: string;
    resize?: () => void;
    total: number;
    allowChangeChartType?: boolean;
    idColumnName: string;
}

// 数据预处理函数
const preprocessGroupedData = (
    data: API.DatasetDataResponse['records'],
    xField: API.DataDictionaryField,
    yField: API.DataDictionaryField
) => {
    return data
        .filter(record => {
            const xValue = record[xField.key];
            const yValue = record[yField.key];
            return xValue !== null && xValue !== undefined && xValue !== '' &&
                yValue !== null && yValue !== undefined && yValue !== '';
        })
        .map(record => ({
            [xField.key]: record[xField.key],
            [yField.key]: record[yField.key],
            // 保留其他字段用于特殊分析
            ...record
        }));
};

// 构建分组统计数据
const buildGroupedStats = (
    data: API.DatasetDataResponse['records'],
    xField: API.DataDictionaryField,
    yField: API.DataDictionaryField
) => {
    const grouped = groupBy(data, xField.key);

    return Object.entries(grouped).map(([groupValue, records]) => {
        const numericValues = records
            .map(r => r[yField.key])
            .filter(v => typeof v === 'number' && !isNaN(v))
            .map(v => Number(v));

        if (numericValues.length === 0) {
            return {
                group: groupValue,
                count: records.length,
                mean: 0,
                median: 0,
                min: 0,
                max: 0,
                std: 0,
                values: []
            };
        }

        const mean = meanBy(numericValues);
        const sorted = numericValues.sort((a, b) => a - b);
        const median = sorted.length % 2 === 0
            ? (sorted[sorted.length / 2 - 1] + sorted[sorted.length / 2]) / 2
            : sorted[Math.floor(sorted.length / 2)];
        const min = Math.min(...numericValues);
        const max = Math.max(...numericValues);
        const variance = numericValues.reduce((acc, val) => acc + Math.pow(val - mean, 2), 0) / numericValues.length;
        const std = Math.sqrt(variance);

        return {
            group: groupValue,
            count: records.length,
            mean,
            median,
            min,
            max,
            std,
            values: numericValues
        };
    });
};

// 注：箱线图、小提琴图和柱状图数据现在直接在各自的渲染函数中构建

// 统计表格组件
const StatsTableComponent: FC<{
    statsData: any[];
    xField: API.DataDictionaryField;
    yField: API.DataDictionaryField;
    total: number;
}> = ({ statsData, xField, yField, total }) => {
    const columns = [
        {
            title: xField.name,
            dataIndex: 'group',
            width: '20%',
            ellipsis: true,
        },
        {
            title: 'Count',
            dataIndex: 'count',
            width: '10%',
        },
        {
            title: 'Mean',
            dataIndex: 'mean',
            width: '12%',
            render: (text: number) => text.toFixed(2)
        },
        {
            title: 'Median',
            dataIndex: 'median',
            width: '12%',
            render: (text: number) => text.toFixed(2)
        },
        {
            title: 'Min',
            dataIndex: 'min',
            width: '10%',
            render: (text: number) => text.toFixed(2)
        },
        {
            title: 'Max',
            dataIndex: 'max',
            width: '10%',
            render: (text: number) => text.toFixed(2)
        },
        {
            title: 'Std',
            dataIndex: 'std',
            width: '12%',
            render: (text: number) => text.toFixed(2)
        },
        {
            title: 'Freq (%)',
            dataIndex: 'count',
            width: '14%',
            render: (text: number) => (
                <span style={{ color: '#1890ff' }}>{(text / total * 100).toFixed(2)}%</span>
            )
        }
    ];

    return (
        <Table
            dataSource={statsData}
            columns={columns}
            rowKey={(record, index) => `${record.group}-${index}`}
            showHeader={true}
            size="small"
            pagination={false}
            sticky={true}
            scroll={{ x: 600 }}
        />
    );
};

// 获取推荐的图表类型
export const getRecommendedGroupedChartType = (
    xField: API.DataDictionaryField,
    yField: API.DataDictionaryField
): GroupedChartType => {
    // 如果X轴是分类变量，Y轴是数值变量
    if (xField.data_type === 'STRING' && yField.data_type === 'NUMBER') {
        return 'box';
    }

    // 如果都是数值变量
    if (xField.data_type === 'NUMBER' && yField.data_type === 'NUMBER') {
        return 'scatter';
    }

    // 如果都是分类变量
    if (xField.data_type === 'STRING' && yField.data_type === 'STRING') {
        return 'summary';
    }

    return 'unsupported';
};

const GroupedChartCard: React.FC<GroupedChartCardProps> = ({
    fields,
    allFields,
    data,
    selectedColumns,
    groupByField,
    onClose,
    className,
    resize,
    total,
    allowChangeChartType = false,
    idColumnName
}) => {
    const xField = fields.find(field => field.key === groupByField);
    const yField = fields.find(field => field.key !== groupByField);

    const [chartType, setChartType] = useState<GroupedChartType>(
        xField && yField ? getRecommendedGroupedChartType(xField, yField) : 'unsupported'
    );
    const headerOffset = 64;

    const [height, setHeight] = useState<number>(300);
    const [width, setWidth] = useState<number>(300);
    const [chartTitle, setChartTitle] = useState<string>('Grouped Analysis');

    const chartRef = useRef<any>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    // 预处理数据
    const processedData = useMemo(() => {
        if (!xField || !yField) return [];
        return preprocessGroupedData(data, xField, yField);
    }, [data, xField, yField]);

    // 构建统计数据
    const statsData = useMemo(() => {
        if (!xField || !yField) return [];
        return buildGroupedStats(processedData, xField, yField);
    }, [processedData, xField, yField]);

    // 注：图表数据现在直接在各自的渲染函数中构建，以提高性能和减少内存使用

    useEffect(() => {
        if (xField && yField) {
            setChartTitle(`${yField.name} by ${xField.name}`);
        }
    }, [xField, yField]);

    // 监听自身尺寸变化以触发图表重绘
    useEffect(() => {
        if (!resize) return;

        const observer = new ResizeObserver(() => {
            if (chartRef.current?.chart?.forceFit) {
                chartRef.current.chart.forceFit();
            } else {
                window.dispatchEvent(new Event('resize'));
            }

            setHeight((containerRef.current?.clientHeight || 332) - headerOffset);
            setWidth((containerRef.current?.clientWidth || 300));
            resize?.();
        });

        if (containerRef.current) {
            observer.observe(containerRef.current);
            setHeight((containerRef.current?.clientHeight || 332) - headerOffset);
        }

        return () => {
            observer.disconnect();
        };
    }, []);

    const renderVisualization = () => {
        if (!groupByField) {
            return <Empty description="Please select a group by field" className="chart-empty" />;
        }

        if (fields.length < 2) {
            return <Empty description="Please select at least two fields" className="chart-empty" />;
        }

        if (!fields.some(field => field.key === groupByField)) {
            return <Empty description="The group by field is not in the fields" className="chart-empty" />;
        }

        if (!xField || !yField) {
            return <Empty description="Please select two fields for grouped analysis" className="chart-empty" />;
        }

        if (isAllNull(data.map(d => d[xField.key]))) {
            return <Empty description={<span>All values in <Tag color="warning">{xField.name}</Tag> are missing</span>} className="chart-empty" />;
        }
        if (isAllNull(data.map(d => d[yField.key]))) {
            return <Empty description={<span>All values in <Tag color="warning">{yField.name}</Tag> are missing</span>} className="chart-empty" />;
        }

        if (processedData.length === 0) {
            return <Empty description="No valid data for grouped analysis" className="chart-empty" />;
        }

        if (chartType === 'summary') {
            return <StatsTableComponent
                statsData={statsData}
                xField={xField!}
                yField={yField!}
                total={total}
            />
        }

        if (chartType === 'bar') {
            if (xField.data_type === 'STRING' && yField.data_type === 'NUMBER') {
                // 使用 Plotly 实现带误差条的柱状图
                const grouped = groupBy(processedData, xField.key);
                const colors = ['#1890ff', '#52c41a', '#faad14', '#f5222d', '#722ed1', '#13c2c2', '#fa541c', '#2f54eb'];

                const xLabels = Object.keys(grouped);
                const yValues = xLabels.map(label => {
                    const records = grouped[label];
                    const numericValues = records
                        .map(r => r[yField.key])
                        .filter(v => typeof v === 'number' && !isNaN(v))
                        .map(v => Number(v));
                    return numericValues.length > 0 ? meanBy(numericValues) : 0;
                });

                const errors = xLabels.map(label => {
                    const records = grouped[label];
                    const numericValues = records
                        .map(r => r[yField.key])
                        .filter(v => typeof v === 'number' && !isNaN(v))
                        .map(v => Number(v));

                    if (numericValues.length > 1) {
                        const mean = meanBy(numericValues);
                        const variance = sumBy(numericValues, v => Math.pow(v - mean, 2)) / (numericValues.length - 1);
                        const stdDev = Math.sqrt(variance);
                        return stdDev / Math.sqrt(numericValues.length);
                    }
                    return 0;
                });

                return (
                    <Plot
                        data={[{
                            x: xLabels,
                            y: yValues,
                            error_y: {
                                type: 'data',
                                array: errors,
                                visible: true,
                                color: '#666',
                                thickness: 1.5,
                                width: 4
                            },
                            type: 'bar',
                            marker: {
                                color: xLabels.map((_, i) => colors[i % colors.length])
                            },
                            text: xLabels.map((label, i) => `n=${grouped[label].length}`),
                            textposition: 'outside',
                            textfont: {
                                size: fontSize,
                                color: '#666'
                            }
                        }]}
                        layout={{
                            width: width,
                            height: height,
                            margin: { l: 60, r: 20, t: 40, b: 80 },
                            xaxis: {
                                title: {
                                    text: xField.name,
                                    font: {
                                        size: fontSize,
                                        color: '#000'
                                    }
                                },
                                showgrid: false,
                                showline: true,
                                linecolor: '#000',
                                linewidth: 1,
                                tickfont: {
                                    size: fontSize - 2,
                                    color: '#000'
                                },
                                automargin: true
                            },
                            yaxis: {
                                title: {
                                    text: `Mean ${yField.name}`,
                                    font: {
                                        size: fontSize,
                                        color: '#000'
                                    }
                                },
                                showgrid: false,
                                showline: true,
                                linecolor: '#000',
                                linewidth: 1,
                                zeroline: false,
                                tickfont: {
                                    size: fontSize - 2,
                                    color: '#000'
                                }
                            },
                            plot_bgcolor: '#fff',
                            paper_bgcolor: '#fff',
                            showlegend: false,
                        }}
                    />
                );
            } else {
                // 对于分类变量 vs 分类变量的情况
                const grouped = groupBy(processedData, xField.key);
                const colors = ['#1890ff', '#52c41a', '#faad14', '#f5222d', '#722ed1', '#13c2c2', '#fa541c', '#2f54eb'];
                const barData = Object.entries(grouped).map(([groupValue, records], index) => ({
                    type: groupValue,
                    value: records.length,
                    count: records.length
                }));

                return (
                    <Column
                        autoFit
                        data={barData}
                        xField="type"
                        yField="value"
                        seriesField="type"
                        ref={chartRef}
                        xAxis={{
                            title: { text: xField.name },
                            label: { autoRotate: true }
                        }}
                        yAxis={{
                            title: { text: 'Count' }
                        }}
                        columnStyle={{
                            radius: [2, 2, 0, 0],
                        }}
                        color={colors}
                    />
                );
            }
        }

        if (chartType === 'box') {
            // 使用 Plotly 实现 Box Plot，因为 AntD 的 Box 组件可能有兼容性问题
            const grouped = groupBy(processedData, xField.key);
            const traces = Object.entries(grouped).map(([groupValue, records], index) => {
                const numericValues = records
                    .map(r => r[yField.key])
                    .filter(v => typeof v === 'number' && !isNaN(v))
                    .map(v => Number(v));

                return {
                    y: numericValues,
                    x: groupValue,
                    type: 'box',
                    name: groupValue.length > 10
                        ? groupValue.slice(0, 10) + '…'
                        : groupValue,
                    boxpoints: 'outliers',
                    marker: {
                        color: ['#1890ff', '#52c41a', '#faad14', '#f5222d', '#722ed1', '#13c2c2', '#fa541c', '#2f54eb'][index % 8],
                    },
                    line: {
                        color: ['#1890ff', '#52c41a', '#faad14', '#f5222d', '#722ed1', '#13c2c2', '#fa541c', '#2f54eb'][index % 8],
                    }
                };
            });

            return (
                <Plot
                    data={traces}
                    layout={{
                        width: width,
                        height: height,
                        margin: { l: 60, r: 20, t: 40, b: 80 },
                        xaxis: {
                            title: {
                                text: xField.name,
                                font: {
                                    size: fontSize,
                                    color: '#000'
                                },
                                type: 'category',
                            },
                            showgrid: false,
                            showline: true,
                            linecolor: '#000',
                            linewidth: 1,
                            tickfont: {
                                size: fontSize - 2,
                                color: '#000'
                            },
                            automargin: true
                        },
                        yaxis: {
                            title: {
                                text: yField.name,
                                font: {
                                    size: fontSize,
                                    color: '#000'
                                }
                            },
                            showgrid: false,
                            showline: true,
                            linecolor: '#000',
                            linewidth: 1,
                            zeroline: false,
                            tickfont: {
                                size: fontSize - 2,
                                color: '#000'
                            }
                        },
                        // boxmode: 'group',
                        showlegend: true,
                        plot_bgcolor: '#fff',
                        paper_bgcolor: '#fff',
                    }}
                />
            );
        }

        if (chartType === 'violin') {
            // 使用 Plotly 实现 Violin Plot
            const grouped = groupBy(processedData, xField.key);
            const traces = Object.entries(grouped).map(([groupValue, records], index) => {
                const numericValues = records
                    .map(r => r[yField.key])
                    .filter(v => typeof v === 'number' && !isNaN(v))
                    .map(v => Number(v));

                return {
                    y: numericValues,
                    type: 'violin',
                    name: groupValue.length > 10
                        ? groupValue.slice(0, 10) + '…'
                        : groupValue,
                    box: {
                        visible: true
                    },
                    meanline: {
                        visible: true
                    },
                    fillcolor: ['#1890ff', '#52c41a', '#faad14', '#f5222d', '#722ed1', '#13c2c2', '#fa541c', '#2f54eb'][index % 8],
                    opacity: 0.6,
                    line: {
                        color: ['#1890ff', '#52c41a', '#faad14', '#f5222d', '#722ed1', '#13c2c2', '#fa541c', '#2f54eb'][index % 8],
                    }
                };
            });

            return (
                <Plot
                    data={traces}
                    layout={{
                        width: width,
                        height: height,
                        margin: { l: 60, r: 20, t: 40, b: 80 },
                        xaxis: {
                            title: {
                                text: xField.name,
                                font: {
                                    size: fontSize,
                                    color: '#000'
                                }
                            },
                            showgrid: false,
                            showline: true,
                            linecolor: '#000',
                            linewidth: 1,
                            tickfont: {
                                size: fontSize - 2,
                                color: '#000'
                            },
                            automargin: true
                        },
                        yaxis: {
                            title: {
                                text: yField.name,
                                font: {
                                    size: fontSize,
                                    color: '#000'
                                }
                            },
                            showgrid: false,
                            showline: true,
                            linecolor: '#000',
                            linewidth: 1,
                            zeroline: false,
                            tickfont: {
                                size: fontSize - 2,
                                color: '#000'
                            }
                        },
                        violinmode: 'group',
                        showlegend: true,
                        plot_bgcolor: '#fff',
                        paper_bgcolor: '#fff',
                    }}
                />
            );
        }

        if (chartType === 'scatter') {
            if (xField.data_type === 'NUMBER' && yField.data_type === 'NUMBER') {
                // 可以通过第三个变量分组（如果有的话）
                const groupField = fields.find(f =>
                    f.key !== xField.key &&
                    f.key !== yField.key &&
                    f.data_type === 'STRING'
                );

                const colors = ['#1890ff', '#52c41a', '#faad14', '#f5222d', '#722ed1', '#13c2c2', '#fa541c', '#2f54eb'];

                let traces;
                if (groupField && selectedColumns?.includes(groupField.key)) {
                    // 有分组变量，按组显示
                    const grouped = groupBy(processedData, groupField.key);
                    traces = Object.entries(grouped).map(([groupValue, records], index) => ({
                        x: records.map(d => d[xField.key]),
                        y: records.map(d => d[yField.key]),
                        type: 'scatter',
                        mode: 'markers',
                        name: groupValue.length > 10
                            ? groupValue.slice(0, 10) + '…'
                            : groupValue,
                        marker: {
                            size: 8,
                            color: colors[index % colors.length],
                            opacity: 0.7
                        },
                    }));
                } else {
                    // 没有分组，单一散点图
                    traces = [{
                        x: processedData.map(d => d[xField.key]),
                        y: processedData.map(d => d[yField.key]),
                        type: 'scatter',
                        mode: 'markers',
                        marker: {
                            size: 8,
                            color: '#1890ff',
                            opacity: 0.7
                        },
                        name: `${yField.name} vs ${xField.name}`
                    }];
                }

                return (
                    <Plot
                        data={traces}
                        layout={{
                            width: width,
                            height: height,
                            margin: { l: 60, r: 20, t: 40, b: 60 },
                            xaxis: {
                                title: {
                                    text: xField.name,
                                    font: {
                                        size: fontSize,
                                        color: '#000'
                                    }
                                },
                                showgrid: false,
                                showline: true,
                                linecolor: '#000',
                                linewidth: 1,
                                zeroline: false,
                                tickfont: {
                                    size: fontSize - 2,
                                    color: '#000'
                                }
                            },
                            yaxis: {
                                title: {
                                    text: yField.name,
                                    font: {
                                        size: fontSize,
                                        color: '#000'
                                    }
                                },
                                showgrid: false,
                                showline: true,
                                linecolor: '#000',
                                linewidth: 1,
                                zeroline: false,
                                tickfont: {
                                    size: fontSize - 2,
                                    color: '#000'
                                }
                            },
                            plot_bgcolor: '#fff',
                            paper_bgcolor: '#fff',
                            showlegend: traces.length > 1,
                        }}
                    />
                );
            } else {
                return <Empty description="Scatter plot requires both fields to be numeric" className="chart-empty" />;
            }
        }

        if (chartType === 'histogram') {
            if (xField.data_type === 'NUMBER' && yField.data_type === 'NUMBER') {
                // 两个数值变量：可以分组展示直方图
                const grouped = groupBy(processedData, xField.key);
                const traces = Object.entries(grouped).map(([groupValue, records]) => ({
                    x: records.map(r => r[yField.key]),
                    type: 'histogram',
                    name: `${xField.name}: ${groupValue}`,
                    opacity: 0.7,
                }));

                return (
                    <Plot
                        data={traces}
                        layout={{
                            width: width,
                            height: height,
                            margin: { l: 60, r: 20, t: 40, b: 60 },
                            xaxis: {
                                title: {
                                    text: yField.name,
                                    font: {
                                        size: fontSize,
                                        color: '#000'
                                    }
                                },
                                showgrid: false,
                                showline: true,
                                linecolor: '#000',
                                linewidth: 1,
                                tickfont: {
                                    size: fontSize - 2,
                                    color: '#000'
                                }
                            },
                            yaxis: {
                                title: {
                                    text: 'Frequency',
                                    font: {
                                        size: fontSize,
                                        color: '#000'
                                    }
                                },
                                showgrid: false,
                                showline: true,
                                linecolor: '#000',
                                linewidth: 1,
                                zeroline: false,
                                tickfont: {
                                    size: fontSize - 2,
                                    color: '#000'
                                }
                            },
                            barmode: 'overlay',
                            plot_bgcolor: '#fff',
                            paper_bgcolor: '#fff',
                        }}
                    />
                );
            } else if (yField.data_type === 'NUMBER') {
                // 只有Y轴是数值：展示简单直方图
                const numericValues = processedData
                    .map(record => record[yField.key])
                    .filter(v => typeof v === 'number' && !isNaN(v));

                return (
                    <Histogram
                        autoFit
                        data={numericValues.map(v => ({ value: v }))}
                        binField="value"
                        binWidth={10}
                        ref={chartRef}
                        xAxis={{ title: { text: yField.name } }}
                        yAxis={{ title: { text: 'Frequency' } }}
                    />
                );
            } else {
                return <Empty description="Histogram requires numeric fields" className="chart-empty" />;
            }
        }

        if (chartType === 'kmplot') {
            // KM Plot实现
            if (xField.data_type === 'STRING') {
                // 确定 timeField 和 eventField
                let timeKey = '', eventKey = '';
                if (xField.key.includes('os')) {
                    timeKey = 'os_months';
                    eventKey = 'os_status';
                } else if (xField.key.includes('dfs')) {
                    timeKey = 'dfs_months';
                    eventKey = 'dfs_status';
                } else if (xField.key.includes('rfs')) {
                    timeKey = 'rfs_months';
                    eventKey = 'rfs_status';
                } else if (xField.key.includes('dmfs')) {
                    timeKey = 'dmfs_months';
                    eventKey = 'dmfs_status';
                }

                const timeField = fields.find(f => f.key === timeKey) || allFields.find(f => f.key === timeKey);
                const eventField = fields.find(f => f.key === eventKey) || allFields.find(f => f.key === eventKey);

                console.log("KM Plot - timeField", timeField, ' eventField', eventField, 'fields', fields);

                if (!timeField || !eventField) {
                    return <Empty description="KM Plot requires valid time/event fields, like os_months, os_status, dfs_months, dfs_status, rfs_months, rfs_status, dmfs_months, dmfs_status." className="chart-empty" />;
                }

                const formatEvent = (event: string | number | boolean) => {
                    if (event === '1' || event === 1 || event === true || event === 1.0) return 1;
                    if (event === '0' || event === 0 || event === false || event === 0.0) return 0;

                    if (typeof event === 'string' && event.startsWith('1')) return 1;
                    if (typeof event === 'string' && event.startsWith('0')) return 0;

                    return 0;
                }

                const formattedData = processedData
                    .filter(d => d[timeField.key] !== null && d[eventField.key] !== null)
                    .map(d => ({
                        ...d,
                        [timeField.key]: Number(d[timeField.key]),
                        [eventField.key]: formatEvent(d[eventField.key]), // 1=事件, 0=截尾
                    }))
                    .sort((a, b) => a.time - b.time);

                // 构建分组
                let grouped: Record<string, typeof processedData> = {};
                if (['STRING', 'BOOLEAN'].includes(yField.data_type)) {
                    grouped = groupBy(formattedData, yField.key);
                } else if (yField.data_type === 'NUMBER') {
                    const values = formattedData.map(r => Number(r[yField.key])).filter(v => !isNaN(v));
                    const median = values.sort((a, b) => a - b)[Math.floor(values.length / 2)];
                    grouped = {
                        [`<= median (${median.toFixed(2)})`]: formattedData.filter(r => Number(r[yField.key]) <= median),
                        [`> median (${median.toFixed(2)})`]: formattedData.filter(r => Number(r[yField.key]) > median)
                    };
                }

                console.log("KM Plot - grouped", grouped, 'formattedData', formattedData);

                // 绘制 KM 曲线
                const colors = ['#1890ff', '#52c41a', '#faad14', '#f5222d', '#722ed1', '#13c2c2', '#fa541c', '#2f54eb'];
                const kmTraces = Object.entries(grouped).map(([group, records], idx) => {
                    const validData = records
                        .filter(r => typeof r[timeField.key] === 'number' && (r[eventField.key] === 1 || r[eventField.key] === 0))
                        .sort((a, b) => a[timeField.key] - b[timeField.key]);

                    let atRisk = validData.length;
                    let survival = 1;
                    const times = [0];
                    const survivals = [1];
                    const texts: string[] = [];

                    validData.forEach((rec) => {
                        const time = rec[timeField.key];
                        const event = rec[eventField.key];
                        if (event === 1) {
                            survival = survival * (atRisk - 1) / atRisk;
                        }
                        atRisk -= 1;
                        times.push(time);
                        survivals.push(survival);
                        texts.push(
                            `Sample ID: ${rec[idColumnName] || 'N/A'}<br>` +
                            `% event free: ${(survival * 100).toFixed(2)}%<br>` +
                            `Time of event: ${time.toFixed(2)} months<br>` +
                            `Number of patients at risk: ${atRisk}`
                        );
                    });

                    return {
                        x: times,
                        y: survivals,
                        text: texts,
                        type: 'scatter',
                        mode: 'lines+markers',
                        name: `${group} (n=${validData.length})`,
                        line: { shape: 'hv', color: colors[idx % colors.length] },
                        hovertemplate: '%{text}<extra></extra>'
                    };
                });

                // 渲染
                return (
                    <Plot
                        data={kmTraces}
                        layout={{
                            width,
                            height,
                            margin: { l: 80, r: 20, t: 40, b: 80 }, // 增加 b 以给 legend 留空间
                            xaxis: {
                                title: timeField.name,
                                showline: true,
                                linewidth: 1,
                                linecolor: '#000'
                            },
                            yaxis: {
                                title: 'Survival Probability',
                                range: [0, 1.05],
                                tickformat: '.0%',
                                showline: true,
                                linewidth: 1,
                                linecolor: '#000'
                            },
                            hovermode: 'x unified',
                            legend: {
                                orientation: 'h',
                                x: 0,
                                y: -0.2,
                                bgcolor: 'rgba(255,255,255,0.9)',
                                bordercolor: '#000',
                                borderwidth: 1
                            },
                            plot_bgcolor: '#fff',
                            paper_bgcolor: '#fff',
                            hoverlabel: {
                                bgcolor: 'rgba(255,255,255,0.9)',  // ✅ 白色背景（带透明度）
                                font: {
                                    color: '#000',
                                    size: 12,
                                    family: 'Arial',
                                },
                                align: 'left',                     // ✅ 左对齐
                                bordercolor: '#ccc',               // ✅ 灰色边框
                                namelength: 0                      // 不截断
                            }
                        }}
                    />
                );
            } else {
                return <Empty description="KM Plot requires categorical grouping variable (X-axis), like os_status, dfs_status, rfs_status, dmfs_status." className="chart-empty" />;
            }
        }

        return <Empty description={`Unsupported chart type: ${chartType}`} className="chart-empty" />;
    };

    return (
        <Row className="grouped-chart-card">
            <Card
                size="small"
                title={<div className="chart-drag-handle">{chartTitle}</div>}
                extra={
                    <Space size={0}>
                        <Popover
                            placement="topRight"
                            content={
                                <StatsTableComponent
                                    statsData={statsData}
                                    xField={xField!}
                                    yField={yField!}
                                    total={total}
                                />
                            }
                            title={null}
                            style={{ width: '800px' }}
                            prefixCls='grouped-chart-card-table-popover'
                            trigger="click"
                            destroyTooltipOnHide
                        >
                            <Button type="text" size="small" icon={<EyeFilled />} />
                        </Popover>

                        <Tooltip title={`${xField?.name} vs ${yField?.name} Grouped Analysis`}>
                            <Button type="text" size="small" icon={<InfoCircleOutlined />} />
                        </Tooltip>

                        {allowChangeChartType && (
                            <Tooltip title="Switch Chart Type">
                                <Select
                                    style={{ width: 150 }}
                                    options={groupedChartTypeOptions}
                                    value={chartType}
                                    onChange={(value) => {
                                        setChartType(value as GroupedChartType);
                                    }}
                                />
                            </Tooltip>
                        )}

                        {onClose && (
                            <Button type="text" size="small" icon={<CloseOutlined />} onClick={onClose} />
                        )}
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

export default GroupedChartCard;
