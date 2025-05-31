import React, { useState } from 'react';
import { Button, Modal, Typography, Row, Col, message, Tooltip, Spin, Tag } from 'antd';
import { useEffect } from 'react';
import { getDatasetData, getDataDictionary, getDatasets } from '@/services/biominer/datasets';
import { history } from 'umi';
import ColumnSelector, { getDefaultSelectedKeys } from './ColumnSelector';
import { filters2string } from './Filter';
import type { ComposeQueryItem } from './Filter';
import { BarChartOutlined, CloudDownloadOutlined, DownloadOutlined, FileOutlined, FilterOutlined, QuestionCircleOutlined } from '@ant-design/icons';
import QueryBuilder from './QueryBuilder';
import VisualPanel from './VisualPanel';
import VirtualTable from './VirtualTable';
import DataInfo from './DataInfo';
import DataDownloader from './DataDownloader';
import Papa from 'papaparse';

import './index.less';

export const downloadTSV = (data: Record<string, any>[], filename = 'metadata.tsv') => {
    const tsv = Papa.unparse(data, {
        delimiter: '\t',
        quotes: false,
    });

    const blob = new Blob(['\uFEFF' + tsv], { type: 'text/tab-separated-values;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
};

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
    const [visualPanelVisible, setVisualPanelVisible] = useState<boolean>(true);
    const [currentRecord, setCurrentRecord] = useState<Record<string, any> | null>(null);
    const [datasetDownloadModalVisible, setDatasetDownloadModalVisible] = useState<boolean>(false);

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

        const fetchData = () => {
            setLoading(true);

            // Fetch the dataset metadata.
            getDatasets({
                page: 1,
                page_size: 1000,
            }).then(datasets => {
                const dataset = datasets.records.find(ds => ds.key === datasetKey);
                if (!dataset) {
                    history.push('/');
                    return;
                }
                setDatasetMetadata(dataset);
            }).catch(err => {
                message.error('Failed to fetch the dataset metadata.');
                history.push('/');
            });

            // Fetch the dataset data.
            const queryMap: any = {
                key: datasetKey,
                page: page,
                page_size: pageSize,
            };

            if (filters) {
                queryMap.query = filters;
            }

            getDatasetData(queryMap).then(d => {
                setData(d);
                setLoading(false);
            }).catch(err => {
                message.error('Failed to fetch the dataset data.');
                setLoading(false);
            });

            // Fetch the dataset data dictionary.
            getDataDictionary({
                key: datasetKey,
            }).then(dDictionary => {
                setDataDictionary(dDictionary);
                setSelectedColumns(getDefaultSelectedKeys(dDictionary.fields));
            }).catch(err => {
                message.error('Failed to fetch the dataset data dictionary.');
            });
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
        setColumns(dataDictionary.fields.filter(col => selectedColumns.includes(col.key)));
    }, [dataDictionary, selectedColumns]);

    const resetParams = () => {
        setPage(1);
        setPageSize(100);
    }

    return (
        <Spin spinning={loading}>
            <Row className="datatable-header">
                <Col span={24} className="datatable-header-upper">
                    <Typography.Title level={4} style={{ height: 28 }}>
                        {datasetMetadata?.name}
                        {!loading ?
                            <Tooltip title="Cite the dataset">
                                <Button onClick={() => {
                                    window.open(`https://www.ncbi.nlm.nih.gov/pubmed/${datasetMetadata?.pmid}`, '_blank');
                                }} icon={<FileOutlined />} style={{ marginLeft: 8 }} type="default" size="small">
                                    Cite Dataset
                                </Button>
                            </Tooltip>
                            : null}
                    </Typography.Title>
                    <p style={{ width: '100%', overflow: 'hidden', margin: 0, textOverflow: 'ellipsis', maxHeight: 45 }}
                        dangerouslySetInnerHTML={{ __html: datasetMetadata?.description ?? '' }} />
                </Col>
                <Col span={24} className="datatable-header-lower">
                    <Col span={14} className="datatable-header-lower-left">
                        {!loading ?
                            <>
                                <Typography.Text style={{ fontSize: 16 }} className="sample-file-count">
                                    ⚠️ Loaded {data.records.length} {datasetMetadata?.is_filebased ? 'files' : 'samples'}, {data.total} {datasetMetadata?.is_filebased ? 'files' : 'samples'} in total.
                                    <Tooltip title={
                                        <span>
                                            If you want to load all the data, please click the button <Tag color="gray">Load All ({data.total})</Tag>. But it will take a while to load.
                                        </span>}
                                    >
                                        <QuestionCircleOutlined style={{ marginLeft: 8 }} />
                                    </Tooltip>
                                    <Button
                                        disabled={data.records.length === data.total}
                                        onClick={
                                            () => {
                                                setPage(1);
                                                if (data.total > 1000) {
                                                    message.warning('The dataset is too large, it will take a while to load.');
                                                }
                                                setPageSize(data.total);
                                            }
                                        }
                                        icon={<CloudDownloadOutlined />} type="primary" size="small" style={{ marginLeft: 8 }}>
                                        Load All
                                    </Button>
                                </Typography.Text>
                                {
                                    filters && <Row className="datatable-filters">
                                        {filters2string(filters)}
                                    </Row>
                                }
                            </>
                            : null}
                    </Col>
                    <Col span={10} className="datatable-header-lower-right">
                        {!loading ?
                            <>
                                <Tooltip title="Download the dataset (each dataset might have at least two files: metadata table and datafile table which contain information about the data)">
                                    <Button
                                        onClick={() => {
                                            setDatasetDownloadModalVisible(true);
                                        }} icon={<DownloadOutlined />} type="default">
                                        Download Dataset
                                    </Button>
                                </Tooltip>
                                <Button
                                    disabled={data.records.length === data.total}
                                    onClick={
                                        () => {
                                            setPage(1);
                                            if (data.total > 1000) {
                                                message.warning('The dataset is too large, it will take a while to load.');
                                            }
                                            setPageSize(data.total);
                                        }
                                    }
                                    icon={<CloudDownloadOutlined />} type="default">
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
                            </>
                            : null}
                    </Col>
                </Col>
            </Row>
            {
                visualPanelVisible ?
                    <VisualPanel fields={dataDictionary.fields} data={data.records} isFileBased={datasetMetadata?.is_filebased ?? false}
                        total={data.total} selectedColumns={selectedColumns}
                        onClose={(field) => {
                            setSelectedColumns(selectedColumns.filter(col => col !== field.key));
                        }} /> :
                    // <VirtualTable
                    //     className='datatable-table'
                    //     dataSource={data.records}
                    //     dataDictionary={columns}
                    //     loading={loading}
                    //     scroll={{ y: window.innerHeight - 270, x: tableWidth }}
                    //     pagination={{
                    //         position: ['bottomRight'],
                    //         pageSize: pageSize,
                    //         current: page,
                    //         total: data.total,
                    //         onChange: (page: number, pageSize: number) => {
                    //             setPage(page);
                    //             setPageSize(pageSize);
                    //         },
                    //         showSizeChanger: true,
                    //         showQuickJumper: true,
                    //         pageSizeOptions: [100, 200, 300, 500, 1000],
                    //     }}
                    //     onCellClick={(record, row, col) => {
                    //         setCurrentRecord(record);
                    //     }}
                    //    isFileBased={datasetMetadata?.is_filebased ?? false}
                    // />
                    <VirtualTable
                        className='datatable-table'
                        size="small"
                        dataSource={data.records}
                        rowKey={(record: any, idx: any) => idx?.toString() ?? ''}
                        scroll={{ y: window.innerHeight - 276, x: window.innerWidth - 48 }}
                        pagination={{
                            position: ['bottomRight'],
                            pageSize: pageSize,
                            current: page,
                            total: data.total,
                            onChange: (page: number, pageSize: number) => {
                                setPage(page);
                                setPageSize(pageSize);
                            },
                            showSizeChanger: true,
                            showQuickJumper: true,
                            pageSizeOptions: [100, 200, 300, 500, 1000],
                        }}
                        onCellClick={(record, row, col) => {
                            setCurrentRecord(record);
                        }}
                        dataDictionary={columns}
                        isFileBased={datasetMetadata?.is_filebased ?? false}
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
            {
                currentRecord ?
                    <Modal
                        className="datatable-data-info-modal"
                        open={currentRecord !== null}
                        width={800}
                        onCancel={() => setCurrentRecord(null)}
                        footer={null}
                    >
                        <DataInfo data={currentRecord} dataDictionary={dataDictionary} title={`Patient ${currentRecord.patient_id} - Details`} />
                    </Modal>
                    : null
            }
            <DataDownloader open={datasetDownloadModalVisible} onClose={() => setDatasetDownloadModalVisible(false)}
                onDownloadMetadataTable={() => {
                    // TODO: Download the metadata table
                    message.info('Downloading the metadata table, it will take a while. Please don\'t close or refresh the page.');

                    if (data.records.length === data.total) {
                        downloadTSV(data.records, 'metadata.tsv')
                    } else {
                        // TODO: The cachedDatasetKey must exist?
                        if (!cachedDatasetKey) return;

                        getDatasetData({
                            key: cachedDatasetKey,
                            page: 1,
                            page_size: data.total,
                        }).then(d => {
                            downloadTSV(d.records, 'metadata.tsv')
                        }).catch(err => {
                            message.error('Failed to download the metadata table, please try again later.');
                        });
                    }
                }}
                onDownloadDatafiles={() => {
                    // TODO: Download the datafiles
                    message.info('Data files are not available yet. But it will come soon.');
                }} />
        </Spin >
    );
};

export default DataTable;
