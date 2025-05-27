import React, { useState } from 'react';
import { Table, Button, Modal, Typography, Row, Col, Space, message, Tooltip, Popover } from 'antd';
import { useEffect } from 'react';
import { getDatasetData, getDataDictionary, getDatasets } from '@/services/biominer/Datasets';
import { history } from 'umi';
import ColumnSelector from './ColumnSelector';
import { filters2string } from './Filter';
import type { ComposeQueryItem } from './Filter';
import { BarChartOutlined, DownloadOutlined, FileOutlined, FilterOutlined, InfoCircleOutlined, MoreOutlined } from '@ant-design/icons';
import QueryBuilder from './QueryBuilder';
import ChartCard from './ChartCard';
import VisualPanel from './VisualPanel';

import './index.less';

const DataTable: React.FC<{ key: string | undefined }> = ({ key }) => {
    const [data, setData] = useState<API.DatasetDataResponse>({
        records: [],
        page: 1,
        page_size: 100,
        total: 0,
    });
    const [dataDictionary, setDataDictionary] = useState<API.DataDictionary>({
        fields: [],
    });
    const [filterModalVisible, setFilterModalVisible] = useState<boolean>(false);
    const [datasetMetadata, setDatasetMetadata] = useState<API.DatasetMetadata | null>(null);
    const [selectedColumns, setSelectedColumns] = useState<string[]>([]);
    // TODO: Use a correct type for the columns
    const [columns, setColumns] = useState<any[]>([]);
    const [page, setPage] = useState<number>(data.page);
    const [pageSize, setPageSize] = useState<number>(data.page_size);
    const [loading, setLoading] = useState<boolean>(false);
    const [cachedDatasetKey, setCachedDatasetKey] = useState<string | undefined>(undefined);
    const [filters, setFilters] = useState<ComposeQueryItem | undefined>(undefined);
    const [visualPanelVisible, setVisualPanelVisible] = useState<boolean>(false);

    useEffect(() => {
        let datasetKey = key ?? '';
        if (!datasetKey) {
            // Get the key from the url, but not the query params.
            const url = window.location.pathname;
            const key = url.split('/').pop();
            if (!key) {
                history.push('/');
                return;
            }

            datasetKey = key;
            setCachedDatasetKey(datasetKey);
        }

        const fetchData = async () => {
            setLoading(true);

            const queryMap: any = {
                key: datasetKey,
                page: page,
                page_size: pageSize,
            };

            if (filters) {
                queryMap.query = filters;
            }

            const data = await getDatasetData(queryMap);
            setData(data);

            const dataDictionary = await getDataDictionary({
                key: datasetKey,
            });
            setDataDictionary(dataDictionary);
            setSelectedColumns(dataDictionary.fields.filter(col => col.order <= 5).map(col => col.key).slice(0, 5));

            const datasets = await getDatasets({
                page: 1,
                page_size: 1000,
            });
            const dataset = datasets.records.find(ds => ds.key === datasetKey);
            if (!dataset) {
                history.push('/');
                return;
            }
            setDatasetMetadata(dataset);
            setLoading(false);
        };

        fetchData();
    }, [key]);

    useEffect(() => {
        if (!cachedDatasetKey) return;
        setLoading(true);

        const fetchData = async () => {
            const queryMap: any = {
                key: cachedDatasetKey,
                page: page,
                page_size: pageSize,
            };

            if (filters) {
                queryMap.query = filters;
            }

            const data = await getDatasetData(queryMap);

            setData(data);
            setLoading(false);
        }
        fetchData();
    }, [page, pageSize, cachedDatasetKey, filters])

    useEffect(() => {
        const columns = dataDictionary.fields.map((col) => ({
            title: (
                <div style={{ display: 'flex', justifyContent: 'space-between', whiteSpace: 'nowrap', gap: 4, alignItems: 'center' }}>
                    <span>{col.name}</span>
                    <Space>
                        <Tooltip title={col.description}>
                            <Button size="small" icon={<InfoCircleOutlined />} />
                        </Tooltip>
                        <Popover content={<ChartCard className='chart-card-popover' field={col} data={data.records} />}
                            trigger="click" destroyTooltipOnHide>
                            <Button size="small" icon={<BarChartOutlined />} type="primary" />
                        </Popover>
                    </Space>
                </div>
            ),
            dataIndex: col.key,
            key: col.key,
        }));

        setColumns(columns.filter(col => selectedColumns.includes(col.key)));
    }, [dataDictionary, selectedColumns]);

    const resetParams = () => {
        setPage(1);
        setPageSize(100);
    }

    return (
        <>
            <Row className="datatable-header">
                <Col span={8} className="datatable-header-left">
                    <Typography.Title level={5}>{datasetMetadata?.name}</Typography.Title>
                    <Tooltip title={datasetMetadata?.description}>
                        <p style={{ width: '100%', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', margin: 0 }}>{datasetMetadata?.description}</p>
                    </Tooltip>
                </Col>
                <Col span={16} className="datatable-header-right">
                    <Space>
                        <Typography.Text style={{ fontSize: 16 }} className="sample-count">
                            ⚠️ Loaded {data.records.length} samples, {data.total} samples in total.
                        </Typography.Text>
                        <Tooltip title="Cite the dataset">
                            <Button onClick={() => {
                                window.open(`https://www.ncbi.nlm.nih.gov/pubmed/${datasetMetadata?.pmid}`, '_blank');
                            }} icon={<FileOutlined />} type="default" />
                        </Tooltip>
                        <Tooltip title="Download the dataset">
                            <Button onClick={() => {
                                // TODO: Download the dataset
                            }} icon={<DownloadOutlined />} disabled type="default" />
                        </Tooltip>
                        <Button onClick={() => {
                            setPage(1);
                            if (data.total > 1000) {
                                message.warning('The dataset is too large, it will take a while to load.');
                            }
                            setPageSize(data.total);
                        }} icon={<MoreOutlined />} type="default">
                            Load All ({data.total})
                        </Button>
                        <ColumnSelector fields={dataDictionary.fields} selectedKeys={selectedColumns} onChange={setSelectedColumns} />
                        <Button type="primary" onClick={() => {
                            setFilterModalVisible(true);
                        }} icon={<FilterOutlined />}>
                            Filter
                        </Button>
                        <Button type="primary" onClick={() => {
                            setVisualPanelVisible(!visualPanelVisible);
                        }} icon={<BarChartOutlined />}>
                            {visualPanelVisible ? 'Show Table' : 'Show Plots'}
                        </Button>
                    </Space>
                </Col>
            </Row>
            {
                filters && <Row className="datatable-filters">
                    {filters2string(filters)}
                </Row>
            }
            {visualPanelVisible ?
                <VisualPanel fields={dataDictionary.fields} data={data.records} /> :
                <Table
                    className={filters ? 'datatable-table-with-filters' : 'datatable-table'}
                    loading={loading}
                    size="small"
                    dataSource={data.records}
                    columns={columns}
                    rowKey={(record, idx) => idx?.toString() ?? ''}
                    scroll={{ y: filters ? 'calc(100vh - 210px)' : 'calc(100vh - 160px)' }}
                    pagination={{
                        position: ['topCenter'],
                        pageSize: pageSize,
                        current: page,
                        total: data.total,
                        onChange: (page, pageSize) => {
                            setPage(page);
                            setPageSize(pageSize);
                        },
                        showSizeChanger: true,
                        showQuickJumper: true,
                        pageSizeOptions: [100, 200, 300, 500, 1000],
                    }}
                />
            }
            <QueryBuilder
                visible={filterModalVisible}
                onCancel={() => setFilterModalVisible(false)}
                onConfirm={(query) => {
                    setFilterModalVisible(false);
                    console.log("QueryBuilder", query);
                    setFilters(query);

                    if (!query) {
                        // Reset the page and page size to the first page and the default page size.
                        resetParams();
                    }
                }}
                dataDictionary={dataDictionary}
            />
        </>
    );
};

export default DataTable;
