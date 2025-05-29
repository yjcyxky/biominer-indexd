import React, { useEffect, useRef, useState } from 'react';
import { Card, Button, Empty, Space, Tooltip, Row, Col, Statistic, Table } from 'antd';
import { CloseOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { Pie, Bar, Histogram } from '@ant-design/plots';
import { groupBy, sumBy } from 'lodash';
// @ts-ignore
import Plotly from 'plotly.js/dist/plotly';
// @ts-ignore
import createPlotlyComponent from 'react-plotly.js/factory';

const Plot = createPlotlyComponent(Plotly);


import './ChartCard.less';

type ChartType = 'id' | 'table' | 'pie' | 'bar' | 'histogram' | 'summary' | 'unsupported' | 'kaplan_meier';

type KMPoint = {
    time: number;
    survival: number;
    event: number;
    at_risk: number;
    label: string;
}

const computeKaplanMeier = (data: API.DatasetDataResponse['records'], timeKey: string, eventKey: string): KMPoint[] => {
    const formatEvent = (event: string | number) => {
        if (event === '1' || event === 1) return 1;
        if (event === '0' || event === 0) return 0;

        if (typeof event === 'string' && event.startsWith('1')) return 1;
        if (typeof event === 'string' && event.startsWith('0')) return 0;

        return 0;
    }

    const raw = data
        .filter(d => d[timeKey] !== null && d[eventKey] !== null)
        .map(d => ({
            time: Number(d[timeKey]),
            event: formatEvent(d[eventKey]), // 1=事件, 0=截尾
            label: d["patient_id"] || "NA"
        }))
        .sort((a, b) => a.time - b.time);

    const result = [{ time: 0, survival: 1, event: 0, at_risk: raw.length, label: '0' }];
    let atRisk = raw.length;
    let survival = 1;

    let lastTime = 0;
    for (const { time, event, label } of raw) {
        if (time !== lastTime) {
            lastTime = time;
        }

        if (event === 1) {
            survival *= (atRisk - 1) / atRisk;
        }
        atRisk -= 1;
        result.push({ time, survival, event, at_risk: atRisk, label });
    }

    return result;
}

interface ChartCardProps {
    field: API.DataDictionaryField;
    data: API.DatasetDataResponse['records'];
    selectedColumns?: string[];
    onClose?: () => void;
    className?: string;
    resize?: () => void;
    total: number;
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

const isKaplanMeier = (field: API.DataDictionaryField, selectedColumns: string[]): boolean => {
    return (selectedColumns?.includes('os_months') && selectedColumns?.includes('os_status') &&
        field.key === 'os_months') ||
        (selectedColumns?.includes('dfs_months') && selectedColumns?.includes('dfs_status') &&
            field.key === 'dfs_months');
}

export const getRecommendedChartType = (field: API.DataDictionaryField, length: number, total: number, selectedColumns?: string[]): ChartType => {
    const allowedValuesLength = field.allowed_values?.length || 0;
    const ratio = allowedValuesLength / total;
    const threshold = 0.8;
    // console.log("GetRecommendedChartType: ", field, ratio, threshold, allowedValuesLength);

    if (isKaplanMeier(field, selectedColumns || [])) {
        return 'kaplan_meier';
    }

    if (field.key === '__summary') return 'summary';
    if (field.data_type === 'BOOLEAN') return 'pie';
    if (field.data_type === 'STRING') {
        if (allowedValuesLength <= 6) return 'pie';
        if (allowedValuesLength > 6 && allowedValuesLength <= 25) return 'bar';
        if (ratio <= threshold && allowedValuesLength <= 500 && allowedValuesLength > 25) return 'table';
        if (ratio > threshold && allowedValuesLength > 100) return 'id';
    }

    if (field.data_type === 'NUMBER') {
        if (allowedValuesLength <= 6) return 'pie';
        if (allowedValuesLength > 6 && allowedValuesLength <= 25) return 'bar';
        if (ratio <= threshold && allowedValuesLength <= 500 && allowedValuesLength > 25) return 'table';
        if (ratio <= threshold && allowedValuesLength > 500) return 'histogram';
        if (ratio > threshold) return 'histogram';
    }
    return 'unsupported';
};

const ChartCard: React.FC<ChartCardProps> = ({ field, data, total, onClose, className, resize, selectedColumns }) => {
    const chartType = getRecommendedChartType(field, data.length, total, selectedColumns);
    const headerOffset = 64; // Card header height

    const [height, setHeight] = useState<number>(300);
    const [width, setWidth] = useState<number>(300);
    const [fieldName, setFieldName] = useState<string>(field.name);

    const chartRef = useRef<any>(null);        // 获取图表组件实例
    const containerRef = useRef<HTMLDivElement>(null); // 监听容器尺寸

    // 监听自身尺寸变化以触发图表重绘
    useEffect(() => {
        if (!resize) return;

        const observer = new ResizeObserver(() => {
            if (chartRef.current?.chart?.forceFit) {
                chartRef.current.chart.forceFit(); // 对于 Ant Design Plots 图表实例
            } else {
                window.dispatchEvent(new Event('resize')); // fallback
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

    useEffect(() => {
        if (isKaplanMeier(field, selectedColumns || [])) {
            setFieldName(field.key === 'os_months' || field.key === 'os_status' ? "Overall Survival Curve" : "Disease-Free Survival Curve");
        } else {
            setFieldName(field.name);
        };
    }, [selectedColumns]);

    const renderVisualization = () => {
        if (chartType === 'kaplan_meier') {
            let eventKey = '';
            let timeKey = '';
            if (field.key === 'os_months' || field.key === 'os_status') {
                eventKey = 'os_status';
                timeKey = 'os_months';
            } else if (field.key === 'dfs_months' || field.key === 'dfs_status') {
                eventKey = 'dfs_status';
                timeKey = 'dfs_months';
            }

            const curve = computeKaplanMeier(data, timeKey, eventKey);

            const text = curve.map(d =>
                `Patient ID: ${d.label || 'N/A'}<br>` +
                `% event free: ${(d.survival * 100).toFixed(2)}%<br>` +
                `Time of event: ${d.time.toFixed(2)} months<br>` +
                `Number of patients at risk: ${d.at_risk}`
            );

            return (
                <Plot
                    data={[
                        {
                            x: curve.map(d => d.time),
                            y: curve.map(d => d.survival),
                            type: 'scatter',
                            mode: 'lines+markers',
                            name: 'Survival Curve',
                            line: { shape: 'hv', color: 'red' }, // step line
                            text: text,
                            hovertemplate: '%{text}<extra></extra>'
                        }
                    ]}
                    layout={{
                        title: 'Survival Curve',
                        width: width,
                        height: height,
                        margin: {
                            l: 50,   // ✅ 控制左边距，避免过大
                            r: 20,
                            t: 20,
                            b: 20,
                        },
                        xaxis: {
                            title: 'Time (Months)',
                            range: [0, null],      // ✅ 强制 X 轴从 0 开始
                            showgrid: false,
                            zeroline: true,
                            linecolor: '#000',
                            linewidth: 1,
                            mirror: true,
                            ticks: 'outside',
                            tickwidth: 1,
                            tickcolor: '#000',
                        },
                        yaxis: {
                            title: 'Survival Probability',
                            range: [0, 1.05],      // ✅ 上限略大于 1，避免遮住顶部点
                            showgrid: false,
                            zeroline: true,
                            linecolor: '#000',
                            linewidth: 1,
                            mirror: true,
                            ticks: 'outside',
                            tickwidth: 1,
                            tickcolor: '#000',
                        },
                        plot_bgcolor: '#fff',
                        paper_bgcolor: '#fff',
                        legend: { x: 0, y: 1 },
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
        }

        if (chartType === 'id') {
            const uniqueValues = new Set(data.map((r) => r[field.key]));
            return <span style={{ color: '#1890ff', fontSize: '24px' }}>{uniqueValues.size}</span>
        }

        if (chartType === 'table') {
            const tableData = Object.entries(groupBy(data, field.key)).map(([value, records]) => ({
                [field.key]: value,
                count: records.length,
            })).sort((a, b) => b.count - a.count);

            const columns = [
                {
                    title: field.name,
                    dataIndex: field.key,
                    width: 'calc(100% - 160px)',
                    ellipsis: true,
                },
                {
                    title: '#',
                    dataIndex: 'count',
                    width: 80,
                },
                {
                    title: 'Freq (%)',
                    dataIndex: 'count',
                    width: 80,
                    render: (text: string, record: any) => (
                        <span style={{ color: '#1890ff' }}>{Math.round(record.count / data.length * 100)}%</span>
                    )
                }
            ];

            return (
                <Table
                    dataSource={tableData}
                    columns={columns}
                    rowKey={(record, index) => `${record[field.key]}-${index}`}
                    rowSelection={{
                        // TODO: Allow to use selection to filter data
                        type: 'checkbox',
                        selectedRowKeys: data.map((r) => r[field.key]),
                        onChange: (selectedRowKeys) => {
                            console.log(selectedRowKeys);
                        }
                    }}
                    showHeader={true}
                    size="small"
                    pagination={false}
                    sticky={true}
                    scroll={{ y: height }}
                />
            )
        }

        if (chartType === 'summary') {
            const loadedCount = data.length;
            const percentage = loadedCount / total * 100;
            const percentageStr = percentage.toFixed(2);

            return (
                <Statistic title="Loaded Samples [Total]" valueRender={() => (
                    <span style={{ color: '#1890ff' }}>{loadedCount} / {total} ({percentageStr}%)</span>
                )} />
            );
        }

        if (chartType === 'pie') {
            const plotData = buildPlotData(data, field.key);

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
            const formatedData = data.map((r) => ({
                [field.key]: r[field.key] === null || r[field.key] === undefined || r[field.key] === '' ? 'NA' : r[field.key],
            }));
            const tableData = Object.entries(groupBy(formatedData, field.key)).map(([value, records]) => ({
                [field.key]: value,
                count: records.length,
            })).sort((a, b) => {
                const va = a[field.key]?.toString().toLowerCase();
                const vb = b[field.key]?.toString().toLowerCase();
                return va.localeCompare(vb); // ✅ 按照字母顺序排序
            });

            return (
                <Bar
                    autoFit
                    data={tableData}
                    xField={field.key}      // ✅ 类别作为 x 轴
                    yField="count"          // ✅ 数值作为 y 轴
                    legend={false}
                    ref={chartRef}
                    xAxis={{
                        title: {
                            text: field.name,
                            style: { fontSize: 12 },
                        },
                        label: {
                            autoHide: false,
                            autoRotate: true,     // ✅ 避免标签重叠
                        },
                    }}
                    yAxis={{
                        title: {
                            text: 'Frequency',
                            style: { fontSize: 12 },
                        },
                    }}
                    columnStyle={{
                        radius: [2, 2, 0, 0],
                    }}
                />
            );
        }

        if (chartType === 'histogram') {
            const numericValues = data.map((r) => r[field.key]).filter((v) => typeof v === 'number');
            return (
                <Histogram
                    data={numericValues.map((v) => ({ value: v }))}
                    binField="value"
                    binWidth={10} // 可调整
                    xAxis={{ title: { text: field.name } }}
                    yAxis={{ title: { text: 'Frequency' } }}
                    autoFit
                />
            )
        }

        return <Empty description={`Unsupported ${field.data_type} field`} className="chart-empty" />;
    };

    return (
        <Row className="chart-card">
            <Card
                size="small"
                title={<div className="chart-drag-handle">{fieldName}</div>}
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
