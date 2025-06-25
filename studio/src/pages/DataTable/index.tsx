import React, { useState } from 'react';
import { Button, Modal, Typography, Row, Col, message, Tooltip, Spin, Tag, Tabs, Select } from 'antd';
import { useEffect } from 'react';
import { getDatasetData, getDataDictionary, getDatasets, getDatafiles, getDatasetReadme, getDatasetLicense } from '@/services/biominer/datasets';
import { history } from 'umi';
import ColumnSelector, { getDefaultSelectedKeys } from './ColumnSelector';
import { filters2string } from './Filter';
import type { ComposeQueryItem } from './Filter';
import { CloudDownloadOutlined, DownloadOutlined, FileOutlined, FilterOutlined, QuestionCircleOutlined } from '@ant-design/icons';
import QueryBuilder from './QueryBuilder';
import VisualPanel from './VisualPanel';
import VirtualTable from './VirtualTable';
import DataInfo from './DataInfo';
import DataDownloader from './DataDownloader';
import Papa from 'papaparse';
import semver from 'semver';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

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
    const [datasetMetadataList, setDatasetMetadataList] = useState<API.DatasetMetadata[]>([]);
    const [datasetMetadata, setDatasetMetadata] = useState<API.DatasetMetadata | null>(null);
    const [dataDictionary, setDataDictionary] = useState<API.DataDictionary>({
        fields: [],
    });
    const [filterModalVisible, setFilterModalVisible] = useState<boolean>(false);
    const [selectedColumns, setSelectedColumns] = useState<string[]>([]);
    // TODO: Use a correct type for the columns
    const [columns, setColumns] = useState<any[]>([]);
    const [page, setPage] = useState<number>(data.page);
    const [pageSize, setPageSize] = useState<number>(data.page_size);
    const [loading, setLoading] = useState<boolean>(false);
    const [cachedDatasetKey, setCachedDatasetKey] = useState<string | undefined>(undefined);
    const [cachedDatasetVersion, setCachedDatasetVersion] = useState<string | undefined>(undefined);
    const [filters, setFilters] = useState<ComposeQueryItem | undefined>(undefined);
    const [currentRecord, setCurrentRecord] = useState<Record<string, any> | null>(null);
    const [datasetDownloadModalVisible, setDatasetDownloadModalVisible] = useState<boolean>(false);
    const [activeTab, setActiveTab] = useState<string>('plots');
    const [readme, setReadme] = useState<string | undefined>(undefined);
    const [license, setLicense] = useState<string | undefined>(undefined);

    const getDatasetKey = (): string => {
        let datasetKey = key ?? '';
        if (!datasetKey) {
            // Get the key from the url, but not the query params.
            const url = window.location.pathname;
            const key = url.split('/').pop();
            if (!key) {
                history.push('/');
                return '';
            }
            datasetKey = key;
        }

        return datasetKey;
    }

    const getDatasetVersion = () => {
        const urlParams = new URLSearchParams(window.location.search);
        const version = urlParams.get('version');
        return version ?? undefined;
    }

    const fetchData = (datasetKey: string, version: string | undefined) => {
        setLoading(true);

        // Fetch the dataset metadata.
        getDatasets({
            page: 1,
            page_size: 1000,
            // TODO: Add a query param to filter the datasets by the key and version.
        }).then(datasets => {
            const filteredDatasets = datasets.records.filter(ds => ds.key === datasetKey);
            if (!filteredDatasets) {
                history.push('/');
                return;
            }

            setDatasetMetadataList(filteredDatasets);

            // 1. 获取最新版本（降序排列）
            const sortedDatasets = filteredDatasets.sort((a, b) =>
                semver.rcompare(a.version, b.version)
            );
            const latestDataset = sortedDatasets[0];

            // 2. 根据 version 查找 dataset，否则 fallback 到 latestDataset
            const selectedVersion = version ?? latestDataset?.version;
            let dataset = filteredDatasets.find(ds => ds.version === selectedVersion);

            // 3. fallback 再次尝试使用 latestDataset
            if (!dataset && latestDataset) {
                dataset = latestDataset;
            }

            // 4. 更新状态
            if (dataset) {
                setCachedDatasetVersion(dataset.version);
                setDatasetMetadata(dataset);
            } else {
                message.error('Failed to fetch the dataset metadata.');
                history.push('/');
            }
        }).catch(err => {
            message.error('Failed to fetch the dataset metadata.');
            history.push('/');
        });
    };

    useEffect(() => {
        const datasetKey = getDatasetKey();
        const version = getDatasetVersion();
        setCachedDatasetKey(datasetKey);
        setCachedDatasetVersion(version);

        console.log("datasetKey", datasetKey, "version", version);
        fetchData(datasetKey, version);
    }, []);

    useEffect(() => {
        if (!cachedDatasetKey) return;
        if (!cachedDatasetVersion) return;

        // Fetch the dataset data dictionary.
        getDataDictionary({
            key: cachedDatasetKey,
            version: cachedDatasetVersion ?? '',
        }).then(dDictionary => {
            setDataDictionary(dDictionary);
            setSelectedColumns(getDefaultSelectedKeys(dDictionary.fields));
        }).catch(err => {
            message.error('Failed to fetch the dataset data dictionary.');
        });
    }, [cachedDatasetKey, cachedDatasetVersion])

    useEffect(() => {
        if (!cachedDatasetKey) return;
        if (!cachedDatasetVersion) return;

        setLoading(true);

        const fetchData = async () => {
            // Fetch the dataset data.
            const queryMap: any = {
                key: cachedDatasetKey,
                version: cachedDatasetVersion ?? '',
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
    }, [page, pageSize, cachedDatasetKey, cachedDatasetVersion, filters])

    useEffect(() => {
        setColumns(dataDictionary.fields.filter(col => selectedColumns.includes(col.key)));
    }, [dataDictionary, selectedColumns]);

    const resetParams = () => {
        setPage(1);
        setPageSize(100);
    }

    const setDatasetVersion = (version: string) => {
        console.log("setDatasetVersion", version);
        setCachedDatasetVersion(version);
        history.replace(`/datatable/${cachedDatasetKey}?version=${version}`);
    }

    return (
        <Spin spinning={loading}>
            <Row className="datatable-header">
                <Typography.Title level={4} style={{ height: 36 }}>
                    {datasetMetadata?.name}
                    {!loading ?
                        <>
                            <Tooltip title="Cite the dataset" placement="top">
                                <Button onClick={() => {
                                    window.open(`https://www.ncbi.nlm.nih.gov/pubmed/${datasetMetadata?.pmid}`, '_blank');
                                }} icon={<FileOutlined />} style={{ marginLeft: 8 }} type="default">
                                    Cite Dataset
                                </Button>
                            </Tooltip>
                            <Button onClick={() => {
                                getDatasetReadme({
                                    key: cachedDatasetKey ?? '',
                                    version: cachedDatasetVersion ?? '',
                                }).then(readme => {
                                    setReadme(readme);
                                }).catch(err => {
                                    message.error('Failed to fetch the dataset readme.');
                                    setReadme(undefined);
                                });
                            }} icon={<FileOutlined />} style={{ marginLeft: 8 }} type="default">
                                README
                            </Button>
                            <Button onClick={() => {
                                getDatasetLicense({
                                    key: cachedDatasetKey ?? '',
                                    version: cachedDatasetVersion ?? '',
                                }).then(license => {
                                    setLicense(license);
                                }).catch(err => {
                                    message.error('Failed to fetch the dataset license.');
                                    setLicense(undefined);
                                });
                            }} icon={<FileOutlined />} style={{ marginLeft: 8 }} type="default">
                                License
                            </Button>
                            <Select
                                defaultValue={cachedDatasetVersion ?? 'Select Version'}
                                style={{ marginLeft: 8, width: 100 }}
                                options={datasetMetadataList.map(ds => ({ label: ds.version, value: ds.version }))}
                                onChange={(value: string) => {
                                    setDatasetVersion(value);
                                }}
                            />
                        </>
                        : null}
                </Typography.Title>
                <p style={{ width: '100%', overflow: 'hidden', margin: 0, textOverflow: 'ellipsis', maxHeight: 45 }}
                    dangerouslySetInnerHTML={{ __html: datasetMetadata?.description ?? '' }} />
            </Row>
            <Tabs defaultActiveKey="plots" activeKey={activeTab} onChange={(key) => { setActiveTab(key) }}
                className='datatable-tabs' tabBarExtraContent={
                    <Row className='datatable-tabs-extra-content'>
                        <Col className="datatable-tabs-extra-content-left">
                            {!loading ?
                                <>
                                    {
                                        filters && <Row className="datatable-filters">
                                            {filters2string(filters, 0, dataDictionary.fields)}
                                        </Row>
                                    }
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
                                </>
                                : null}
                        </Col>
                        <Col className="datatable-tabs-extra-content-right">
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
                                </>
                                : null}
                        </Col>
                    </Row>
                }>
                <Tabs.TabPane tab="Summary" key="plots">
                    <VisualPanel fields={dataDictionary.fields} data={data.records} isFileBased={datasetMetadata?.is_filebased ?? false}
                        total={data.total} selectedColumns={selectedColumns}
                        onClose={(field) => {
                            setSelectedColumns(selectedColumns.filter(col => col !== field.key));
                        }} />
                </Tabs.TabPane>
                <Tabs.TabPane tab="Clinical Data" key="table">
                    {/* // <VirtualTable
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
                        // /> */}
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
                </Tabs.TabPane>
            </Tabs >
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
                        // TODO: The cachedDatasetKey and cachedDatasetVersion must exist.
                        if (!cachedDatasetKey) return;
                        if (!cachedDatasetVersion) return;

                        getDatasetData({
                            key: cachedDatasetKey,
                            version: cachedDatasetVersion ?? '',
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
                    message.info('Downloading the datafiles, it will take a while. Please don\'t close or refresh the page.');
                    if (!cachedDatasetKey) {
                        message.error('Failed to download the datafiles, please try again later.');
                        return;
                    }

                    getDatafiles({
                        key: cachedDatasetKey,
                        version: cachedDatasetVersion ?? '',
                    }).then((d: any) => {
                        downloadTSV(d, 'datafiles.tsv')
                    }).catch((err: any) => {
                        message.error('Failed to download the datafiles, please try again later.');
                    });
                }} />
            {
                readme ?
                    <Modal open={readme !== null} width={800} onCancel={() => setReadme(undefined)} footer={null}
                        title={<Typography.Title level={4}>README</Typography.Title>}>
                        <ReactMarkdown remarkPlugins={[remarkGfm]} children={readme}></ReactMarkdown>
                    </Modal>
                    : null
            }
            {
                license ?
                    <Modal open={license !== null} width={800} onCancel={() => setLicense(undefined)} footer={null}
                        title={<Typography.Title level={4}>License</Typography.Title>}>
                        <ReactMarkdown remarkPlugins={[remarkGfm]} children={license}></ReactMarkdown>
                    </Modal>
                    : null
            }
        </Spin >
    );
};

export default DataTable;
