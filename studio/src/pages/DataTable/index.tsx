import React, { useState } from 'react';
import { Button, Modal, Typography, Row, Col, message, Tooltip, Spin, Tag, Tabs, Select, Progress, Alert, Switch } from 'antd';
import { useEffect } from 'react';
import { getDatasetData, getDataDictionary, getDatasets, getDatafiles, getDatasetReadme, getDatasetLicense, getDatafileTables, getDatasetDataWithQueryPlan } from '@/services/biominer/datasets';
import { history } from 'umi';
import ColumnSelector, { getDefaultSelectedKeys } from './ColumnSelector';
import { filters2string } from './Filter';
import type { ComposeQueryItem } from './Filter';
import { CloudDownloadOutlined, DownloadOutlined, FileOutlined, FilterOutlined, QuestionCircleOutlined, LoadingOutlined, BarChartOutlined, TableOutlined } from '@ant-design/icons';
import QueryBuilder from './QueryBuilder';
import GroupedVisualPanel from './GroupedVisualPanel';
import VisualPanel from './VisualPanel';
import VirtualTable from './VirtualTable';
import DataInfo from './DataInfo';
import DataDownloader from './DataDownloader';
import Papa from 'papaparse';
import semver from 'semver';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { DEFAULT_ID_COLUMN_NAME, DEFAULT_ID_COLUMN_NAMES } from './ChartCard';
import { mergeData } from './utils';

import './index.less';

const pickYField = (fields: API.DataDictionaryField[], id_column_name: string) => {
    const filteredFields = fields.filter(field => field.key !== id_column_name);
    if (filteredFields.length === 0) {
        return undefined;
    }

    return filteredFields[0];
}

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
    // Initial dataset metadata which is dependent by other states.
    const [datasetMetadataList, setDatasetMetadataList] = useState<API.DatasetMetadata[]>([]);
    const [datasetMetadata, setDatasetMetadata] = useState<API.DatasetMetadata | null>(null);
    const [dataDictionary, setDataDictionary] = useState<API.DataDictionary>({
        fields: [],
    });
    const [isReady, setIsReady] = useState<boolean>(false);

    // Data
    const [data, setData] = useState<API.DatasetDataResponse>({
        records: [],
        page: 1,
        page_size: 100,
        total: 0,
    });
    const [selectedColumns, setSelectedColumns] = useState<string[]>([]);
    // TODO: Use a correct type for the columns
    const [columns, setColumns] = useState<any[]>([]);
    const [page, setPage] = useState<number>(data.page);
    const [pageSize, setPageSize] = useState<number>(data.page_size);

    const [cachedDatasetKey, setCachedDatasetKey] = useState<string | undefined>(undefined);
    const [cachedDatasetVersion, setCachedDatasetVersion] = useState<string | undefined>(undefined);
    const [filters, setFilters] = useState<ComposeQueryItem | undefined>(undefined);

    // UI states
    const [currentRecord, setCurrentRecord] = useState<Record<string, any> | null>(null);
    const [datasetDownloadModalVisible, setDatasetDownloadModalVisible] = useState<boolean>(false);
    const [filterModalVisible, setFilterModalVisible] = useState<boolean>(false);
    const [activeTab, setActiveTab] = useState<string>('summary');

    // Enhanced loading states
    const [loadingStates, setLoadingStates] = useState<{
        metadata: boolean;
        data: boolean;
        datafileTables: boolean;
        rendering: boolean;
    }>({
        metadata: false,
        data: false,
        datafileTables: false,
        rendering: false,
    });
    const [loadingProgress, setLoadingProgress] = useState<{
        current: number;
        total: number;
        message: string;
    }>({
        current: 0,
        total: 0,
        message: '',
    });

    // README and LICENSE
    const [readme, setReadme] = useState<string | undefined>(undefined);
    const [license, setLicense] = useState<string | undefined>(undefined);

    // Datafile tables
    const [dataFileTables, setDataFileTables] = useState<API.DataFileTable[]>([]);
    const [dataFileTableData, setDataFileTableData] = useState<Record<string, API.DatasetDataResponse['records']>>({});
    const [dataFileTablePage, setDataFileTablePage] = useState<Record<string, number>>({});
    const [dataFileTablePageSize, setDataFileTablePageSize] = useState<Record<string, number>>({});
    const [dataFileTableTotal, setDataFileTableTotal] = useState<Record<string, number>>({});
    const [dataFileTableColumns, setDataFileTableColumns] = useState<Record<string, any[]>>({});
    const [dataFileTableSelectedColumns, setDataFileTableSelectedColumns] = useState<Record<string, string[]>>({});
    const [dataFileTableShowTable, setDataFileTableShowTable] = useState<Record<string, boolean>>({});

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
        setLoadingStates(prev => ({ ...prev, metadata: true }));
        setLoadingProgress({
            current: 0,
            total: 3,
            message: 'Loading dataset metadata...'
        });

        // Fetch the dataset metadata.
        getDatasets({
            page: 1,
            page_size: 1000,
            // TODO: Add a query param to filter the datasets by the key and version.
        }).then(datasets => {
            setLoadingProgress(prev => ({ ...prev, current: 1, message: 'Processing dataset information...' }));

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
                setPageSize(dataset.total);
                setLoadingProgress(prev => ({ ...prev, current: 2, message: 'Dataset metadata loaded' }));
            } else {
                message.error('Failed to fetch the dataset metadata.');
                history.push('/');
            }

            setLoadingStates(prev => ({ ...prev, metadata: false }));
        }).catch(err => {
            message.error('Failed to fetch the dataset metadata.');
            history.push('/');
            setLoadingStates(prev => ({ ...prev, metadata: false }));
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

        setLoadingStates(prev => ({ ...prev, metadata: true }));
        setLoadingProgress({
            current: 0,
            total: 2,
            message: 'Loading data dictionary...'
        });

        // Fetch the dataset data dictionary.
        getDataDictionary({
            key: cachedDatasetKey,
            version: cachedDatasetVersion ?? '',
        }).then(dDictionary => {
            setLoadingProgress(prev => ({ ...prev, current: 1, message: 'Processing data dictionary...' }));
            setDataDictionary(dDictionary);
            setSelectedColumns(getDefaultSelectedKeys(dDictionary.fields));
            setLoadingProgress(prev => ({ ...prev, current: 2, message: 'Data dictionary loaded' }));
            setLoadingStates(prev => ({ ...prev, metadata: false }));
        }).catch(err => {
            message.error('Failed to fetch the dataset data dictionary.');
            setLoadingStates(prev => ({ ...prev, metadata: false }));
        });

        // Fetch the datafiles dictionaries.
        setLoadingStates(prev => ({ ...prev, datafileTables: true }));
        getDatafileTables({
            key: cachedDatasetKey,
            version: cachedDatasetVersion ?? '',
        }).then((dTables: API.DataFileTable[]) => {
            setDataFileTables(dTables);
            setDataFileTableSelectedColumns({
                ...dataFileTableSelectedColumns,
                ...dTables.reduce((acc, table) => {
                    const cols = getDefaultSelectedKeys(table.data_dictionary.fields.filter(field => field.key !== table.id_column_name), 1);
                    acc[table.table_name] = [...cols, table.id_column_name];
                    return acc;
                }, {} as Record<string, string[]>),
            });
            setLoadingStates(prev => ({ ...prev, datafileTables: false }));
        }).catch((err: any) => {
            message.error('Failed to fetch the dataset datafile tables.');
            console.error('Failed to fetch the dataset datafile tables.', err);
            setLoadingStates(prev => ({ ...prev, datafileTables: false }));
        });
    }, [cachedDatasetKey, cachedDatasetVersion])

    useEffect(() => {
        if (dataDictionary.fields.length > 0 && datasetMetadata) {
            setIsReady(true);
        }
    }, [dataDictionary, datasetMetadata])

    useEffect(() => {
        setColumns(dataDictionary.fields.filter(col => selectedColumns.includes(col.key)));
    }, [dataDictionary, selectedColumns]);

    useEffect(() => {
        if (!cachedDatasetKey) return;
        if (!cachedDatasetVersion) return;
        if (!isReady) return;

        setLoadingStates(prev => ({ ...prev, data: true }));
        setLoadingProgress({
            current: 0,
            total: 3,
            message: 'Building query plan...'
        });

        const fetchData = async () => {
            setLoadingProgress(prev => ({ ...prev, current: 1, message: 'Executing data query...' }));

            const queryPlan: any = {
                table: "metadata_table",
                joins: [],
                selects: selectedColumns.map(col => ({
                    type: "field",
                    value: col,
                })),
                filters: filters,
                group_by: [],
                having: undefined,
                order_by: [],
                limit: pageSize,
                offset: (page - 1) * pageSize,
                distinct: false,
            }

            if (filters) {
                queryPlan.query = filters;
            }

            const data = await getDatasetDataWithQueryPlan({
                key: cachedDatasetKey,
                version: cachedDatasetVersion ?? '',
                query_plan: queryPlan,
            });

            setLoadingProgress(prev => ({ ...prev, current: 2, message: 'Processing query result...' }));

            setData(data);

            setLoadingProgress(prev => ({ ...prev, current: 3, message: 'Data loaded, rendering components...' }));

            // 模拟组件渲染时间
            setLoadingStates(prev => ({ ...prev, rendering: true }));
            setTimeout(() => {
                setLoadingStates(prev => ({ ...prev, rendering: false, data: false }));
                setLoadingProgress({
                    current: 0,
                    total: 0,
                    message: ''
                });
            }, 500);
        }
        fetchData();
    }, [page, pageSize, cachedDatasetKey, cachedDatasetVersion, filters, selectedColumns, isReady])

    useEffect(() => {
        if (!cachedDatasetKey) return;
        if (!cachedDatasetVersion) return;
        if (!isReady) return;

        if (!dataFileTables.find(table => table.table_name === activeTab)) {
            setLoadingStates(prev => ({ ...prev, data: false }));
            return;
        }

        console.log("Fetching data file table: ", activeTab, filters, data, isReady, dataFileTableSelectedColumns, dataFileTablePage, dataFileTablePageSize)

        setLoadingStates(prev => ({ ...prev, data: true }));
        setLoadingProgress({
            current: 0,
            total: 2,
            message: `Loading ${activeTab} table data...`
        });

        const dataFileTableFilters = {
            field: dataFileTables.find(table => table.table_name === activeTab)?.id_column_name,
            operator: 'in',
            value: data.records.map((record: any) => record[DEFAULT_ID_COLUMN_NAME]),
        }

        const fetchData = async () => {
            setLoadingProgress(prev => ({ ...prev, current: 1, message: 'Querying data file...' }));

            const queryPlan: any = {
                table: activeTab,
                joins: [],
                selects: (dataFileTableSelectedColumns[activeTab] ?? []).map(col => ({
                    type: "field",
                    value: col,
                })),
                filters: dataFileTableFilters,
                group_by: [],
                having: undefined,
                order_by: [],
                limit: dataFileTablePageSize[activeTab] ?? 100,
                offset: (dataFileTablePage[activeTab] ?? 1 - 1) * (dataFileTablePageSize[activeTab] ?? 100),
                distinct: false,
            }

            const data = await getDatasetDataWithQueryPlan({
                key: cachedDatasetKey,
                version: cachedDatasetVersion ?? '',
                query_plan: queryPlan,
            });

            setLoadingProgress(prev => ({ ...prev, current: 2, message: 'Processing data file result...' }));

            setDataFileTableData({ ...dataFileTableData, [activeTab]: data.records });
            setDataFileTableTotal({ ...dataFileTableTotal, [activeTab]: data.total });
            // TODO: Why they cause the dead loop?
            // setDataFileTablePage({ ...dataFileTablePage, [activeTab]: data.page });
            // setDataFileTablePageSize({ ...dataFileTablePageSize, [activeTab]: data.page_size });

            setLoadingStates(prev => ({ ...prev, data: false }));
            setLoadingProgress({
                current: 0,
                total: 0,
                message: ''
            });
        }
        fetchData();
    }, [activeTab, filters, data, isReady, dataFileTableSelectedColumns, dataFileTablePage, dataFileTablePageSize])

    useEffect(() => {
        if (!cachedDatasetKey) return;
        if (!cachedDatasetVersion) return;
        if (!isReady) return;

        const table = dataFileTables.find(table => table.table_name === activeTab);
        if (!table) return;

        const selectedDataFileTableColumns = dataFileTableSelectedColumns[activeTab] ?? [];
        const columns = table.data_dictionary.fields.filter(col => selectedDataFileTableColumns.includes(col.key));
        setDataFileTableColumns({ ...dataFileTableColumns, [activeTab]: columns });
    }, [activeTab, dataFileTables, dataFileTableSelectedColumns])

    const resetParams = () => {
        setPage(1);
        setPageSize(100);
    }

    const resetAllState = () => {
        setPage(1);
        setPageSize(100);
        setFilters(undefined);
        setCurrentRecord(null);
        setDatasetDownloadModalVisible(false);
        setActiveTab('summary');
        setReadme(undefined);
        setLicense(undefined);

        setDataFileTables([]);
        setDataFileTableData({});
        setDataFileTablePage({});
        setDataFileTablePageSize({});
        setDataFileTableTotal({});
        setDataFileTableColumns({});
        setDataFileTableSelectedColumns({});
    };

    const setDatasetVersion = (version: string) => {
        console.log("setDatasetVersion", version);
        setCachedDatasetVersion(version);
        resetAllState();
        history.replace(`/datatable/${cachedDatasetKey}?version=${version}`);
    }

    // 计算总体loading状态
    const isAnyLoading = loadingStates.metadata || loadingStates.data || loadingStates.datafileTables || loadingStates.rendering;
    const showProgress = loadingProgress.total > 0;

    return (
        <Spin spinning={isAnyLoading} indicator={<LoadingOutlined style={{ fontSize: 24 }} spin />}>
            {/* Loading Progress Indicator */}
            {showProgress && (
                <Alert
                    message={
                        <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
                            <span>{loadingProgress.message}</span>
                            <Progress
                                percent={Math.round((loadingProgress.current / loadingProgress.total) * 100)}
                                size="small"
                                style={{ maxWidth: 'calc(100% - 240px)' }}
                                showInfo={false}
                            />
                        </div>
                    }
                    type="info"
                    showIcon
                    style={{ margin: '0 auto 16px', width: '50%' }}
                />
            )}

            <Row className="datatable-header">
                <Typography.Title level={4} style={{ height: 36 }}>
                    {datasetMetadata?.name}
                    {!isAnyLoading ?
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
            <Tabs defaultActiveKey="summary" activeKey={activeTab} onChange={(key) => { setActiveTab(key) }} destroyOnHidden={true}
                className='datatable-tabs' tabBarExtraContent={
                    <Row className='datatable-tabs-extra-content'>
                        <Col className="datatable-tabs-extra-content-left">
                            {!isAnyLoading ?
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
                            {!isAnyLoading ?
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
                <Tabs.TabPane tab="Summary" key="summary">
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
                {
                    dataFileTables.map(table => (
                        <Tabs.TabPane tab={<Tooltip title={table.description}>{table.title}</Tooltip>} key={table.table_name}>
                            <Row>
                                <Button onClick={() => {
                                    setDataFileTableShowTable({ ...dataFileTableShowTable, [table.table_name]: !dataFileTableShowTable[table.table_name] });
                                }}
                                    className={`switch-${table.table_name}`}
                                    icon={dataFileTableShowTable[table.table_name] ? <BarChartOutlined /> : <TableOutlined />}>
                                    {dataFileTableShowTable[table.table_name] ? 'Show Charts' : 'Show Table'}
                                </Button>
                                <ColumnSelector fields={table.data_dictionary.fields.filter((field: any) => field.key !== table.id_column_name)}
                                    selectedKeys={dataFileTableSelectedColumns[table.table_name] ?? []}
                                    className={`column-selector-${table.table_name}`}
                                    title={`Columns`}
                                    onChange={(keys) => {
                                        // The id column must be selected for the grouped visualization.
                                        setDataFileTableSelectedColumns({ ...dataFileTableSelectedColumns, [table.table_name]: [...keys, table.id_column_name] });
                                    }} mode="single" />
                                {
                                    !dataFileTableShowTable[table.table_name] ? (
                                        <GroupedVisualPanel fields={dataDictionary.fields.filter((field: any) => {
                                            return selectedColumns.find((col: any) => col === field.key) && !DEFAULT_ID_COLUMN_NAMES.includes(field.key);
                                        })} data={
                                            mergeData(dataFileTableData[table.table_name] ?? [], data.records, {
                                                leftOn: table.id_column_name ?? DEFAULT_ID_COLUMN_NAME,
                                                rightOn: DEFAULT_ID_COLUMN_NAME,
                                                how: 'left',
                                            })}
                                            idColumnName={table.id_column_name ?? DEFAULT_ID_COLUMN_NAME}
                                            total={dataFileTableTotal[table.table_name] ?? 0}
                                            selectedColumns={dataFileTableSelectedColumns[table.table_name] ?? []}
                                            yField={pickYField(dataFileTableColumns[table.table_name] ?? [], table.id_column_name ?? DEFAULT_ID_COLUMN_NAME)}
                                            onClose={() => { }} />
                                    ) : (
                                        <VirtualTable
                                            className='datatable-table'
                                            size="small"
                                            dataSource={dataFileTableData[table.table_name] ?? []}
                                            rowKey={(record: any, idx: any) => idx?.toString() ?? ''}
                                            scroll={{ y: window.innerHeight - 276, x: window.innerWidth - 48 }}
                                            pagination={{
                                                position: ['bottomRight'],
                                                pageSize: dataFileTablePageSize[table.table_name] ?? 100,
                                                current: dataFileTablePage[table.table_name] ?? 1,
                                                total: dataFileTableTotal[table.table_name] ?? 0,
                                                onChange: (page: number, pageSize: number) => {
                                                    setDataFileTablePage({ ...dataFileTablePage, [table.table_name]: page });
                                                    setDataFileTablePageSize({ ...dataFileTablePageSize, [table.table_name]: pageSize });
                                                },
                                                showSizeChanger: true,
                                                showQuickJumper: true,
                                                pageSizeOptions: [100, 200, 300, 500, 1000],
                                            }}
                                            dataDictionary={dataFileTableColumns[table.table_name] ?? []}
                                            isFileBased={false}
                                        />
                                    )
                                }
                            </Row>
                        </Tabs.TabPane>
                    ))
                }
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
