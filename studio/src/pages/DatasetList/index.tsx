import React, { useState, useEffect, useCallback } from 'react';
import { Layout, Menu, List, Button, Typography, message, Input, Row, Col, Modal, Tooltip, Tag, Checkbox } from 'antd';
import { getDatasets } from '@/services/biominer/datasets';
import './index.less';
import { BarChartOutlined, DownloadOutlined, FileTextFilled, InfoCircleFilled, PieChartFilled, SortAscendingOutlined, SortDescendingOutlined } from '@ant-design/icons';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useHistory } from 'umi';

const { Sider } = Layout;

const DatasetList: React.FC = () => {
    const [datasets, setDatasets] = useState<API.DatasetsResponse>({
        records: [],
        total: 0,
        page: 1,
        page_size: 10,
    });
    const [allTags, setAllTags] = useState<string[]>([]);
    const [selectedTag, setSelectedTag] = useState<string>('All');
    const [tagDatasetsMap, setTagDatasetsMap] = useState<Record<string, API.DatasetMetadata[]>>({});
    const [searchValue, setSearchValue] = useState<string>('');
    const [isModalOpen, setIsModalOpen] = useState<boolean>(false);
    const [markdown, setMarkdown] = useState<string>('');
    const [orderField, setOrderField] = useState<string>('num_of_samples');
    const [orderType, setOrderType] = useState<{ name: string, num_of_samples: string }>({ name: 'desc', num_of_samples: 'desc' });
    const history = useHistory();
    const [loading, setLoading] = useState(false);
    const [selectedDataset, setSelectedDataset] = useState<API.DatasetMetadata | null>(null);

    useEffect(() => {
        setLoading(true);
        getDatasets({ page: 1, page_size: 1000 })
            .then(res => {
                setDatasets(res);
                setLoading(false);
            })
            .catch(err => message.error(err.message))
            .finally(() => {
                setLoading(false);
            });
    }, []);

    useEffect(() => {
        const tags = [...new Set(datasets.records.flatMap(ds => ds.tags))].sort();
        setAllTags(tags);

        const tagMap: Record<string, API.DatasetMetadata[]> = {};
        datasets.records.forEach(ds => {
            if (ds.tags.length > 0) {
                ds.tags.forEach(tag => {
                    tagMap[tag] = [...(tagMap[tag] || []), ds];
                });
            } else {
                tagMap['No tags'] = [...(tagMap['No tags'] || []), ds];
            }
        });

        tagMap['All'] = datasets.records;
        setTagDatasetsMap(tagMap);
    }, [datasets]);

    const onSearch = (value: string) => {
        setSearchValue(value);
    };

    const handleCancel = () => {
        setIsModalOpen(false);
    };

    const filterDatasets = (
        tag: string, searchValue: string, oField: string, oType: 'asc' | 'desc'
    ) => {
        console.log(tagDatasetsMap[tag], searchValue, oField, oType);
        const data = tagDatasetsMap[tag]?.filter(ds => searchValue === '' || ds.name.includes(searchValue));
        return data && data.sort((a: any, b: any) => {
            if (oField === 'name') {
                return oType === 'asc' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name);
            } else if (oField === 'num_of_samples') {
                return oType === 'asc' ? a.num_of_samples - b.num_of_samples : b.num_of_samples - a.num_of_samples;
            }
        });
    };

    return (
        <Row className="dataset-container">
            <Col className="dataset-header" span={24}>
                <div className="dataset-header-title">Select Datasets for Visualization & Analysis:</div>
                <Tag className="tag-count" color="blue">
                    {tagDatasetsMap[selectedTag]?.length || 0} Datasets Listed
                </Tag>
                <div className="dataset-search">
                    <Button type="text" icon={orderType.name === 'asc' ? <SortDescendingOutlined /> : <SortAscendingOutlined />} onClick={() => {
                        setOrderField('name');
                        setOrderType({
                            name: orderType.name === 'asc' ? 'desc' : 'asc',
                            num_of_samples: orderType.num_of_samples,
                        });
                    }}>Sort by Name</Button>
                    <Button type="text" icon={orderType.num_of_samples === 'asc' ? <SortDescendingOutlined /> : <SortAscendingOutlined />} onClick={() => {
                        setOrderField('num_of_samples');
                        setOrderType({
                            name: orderType.name,
                            num_of_samples: orderType.num_of_samples === 'asc' ? 'desc' : 'asc',
                        });
                    }}>Sort by Sample Count</Button>
                    <Input.Search
                        placeholder="Search by dataset name"
                        enterButton="Search"
                        allowClear
                        size="middle"
                        onSearch={onSearch}
                    />
                </div>
            </Col>
            <Col className="dataset-content" span={24}>
                <Sider width={220} className="dataset-left-sider">
                    <Menu
                        mode="inline"
                        selectedKeys={[selectedTag]}
                        onClick={e => setSelectedTag(e.key)}
                    >
                        <Menu.Item key="All">
                            <span className="tag-name">All</span>
                            <Tag className="tag-count">{datasets.records.length}</Tag>
                        </Menu.Item>
                        <Menu.Item key="No tags">
                            <span className="tag-name">No tags</span>
                            <Tag className="tag-count">{tagDatasetsMap['No tags']?.length || 0}</Tag>
                        </Menu.Item>
                        {allTags.map(tag => (
                            <Menu.Item key={tag}>
                                <span className="tag-name">{tag}</span>
                                <Tag className="tag-count">{tagDatasetsMap[tag]?.length || 0}</Tag>
                            </Menu.Item>
                        ))}
                    </Menu>
                </Sider>

                <div className="dataset-right-sider-container">
                    <List
                        loading={loading}
                        className="dataset-right-sider"
                        itemLayout="horizontal"
                        dataSource={
                            filterDatasets(selectedTag, searchValue, orderField,
                                orderType[orderField as keyof typeof orderType] as 'asc' | 'desc'
                            ) || []
                        }

                        locale={{ emptyText: 'No datasets available' }}
                        renderItem={item => (
                            <List.Item
                                className="dataset-item"
                                actions={[
                                    <Button type="link" icon={<FileTextFilled />}
                                        onClick={() => window.open(`https://pubmed.ncbi.nlm.nih.gov/${item.pmid}`, '_blank')}>
                                        Cite
                                    </Button>,
                                    <Tooltip title="Coming soon...">
                                        <Button type="link" icon={<InfoCircleFilled />} disabled
                                            onClick={() => {
                                                // TODO: Show the markdown content of the dataset in another modal.
                                                setIsModalOpen(true);
                                                setMarkdown(item.description);
                                            }}>
                                            Info
                                        </Button>
                                    </Tooltip>,
                                    <Tooltip title="Coming soon...">
                                        <Button type="link" icon={<DownloadOutlined />} disabled
                                            onClick={() => {
                                                // TODO: Redirect to the data-repo page.
                                            }}>
                                            Download
                                        </Button>
                                    </Tooltip>,
                                    <Button type="link" icon={<PieChartFilled />} onClick={() => {
                                        history.push(`/datatable/${item.key}`);
                                    }}>
                                        Visualize
                                    </Button>,
                                ]}
                            >
                                <List.Item.Meta
                                    title={
                                        <>
                                            <Checkbox
                                                checked={selectedDataset?.key === item.key}
                                                onChange={(e) => {
                                                    if (e.target.checked) {
                                                        setSelectedDataset(item);
                                                    } else {
                                                        setSelectedDataset(null);
                                                    }
                                                }}
                                                style={{ marginRight: 8 }}
                                            />
                                            <Typography.Text>{item.name}</Typography.Text>
                                        </>
                                    }
                                    description={
                                        <div className="dataset-description">
                                            {
                                                item.groups.length > 0 && <span>
                                                    {item.groups.map(group => <Tag key={group} style={{ fontSize: 8 }}>{group}</Tag>)}
                                                </span>
                                            }
                                            <p dangerouslySetInnerHTML={{ __html: item.description }} />
                                        </div>
                                    }
                                />
                                <div className="sample-count">{item.num_of_samples} samples</div>
                            </List.Item>
                        )}
                    />

                    <Row className="dataset-explore-button">
                        <Button type="primary" size="large" onClick={() => {
                            history.push(`/datatable/${selectedDataset?.key}`);
                        }} disabled={selectedDataset === null} icon={<BarChartOutlined />}>
                            Explore Selected Dataset
                        </Button>
                    </Row>
                </div>
            </Col>
            <Modal
                width="50%"
                className='dataset-info-modal'
                title="Dataset Info"
                open={isModalOpen}
                onCancel={handleCancel}
                footer={null}
            >
                <ReactMarkdown remarkPlugins={[remarkGfm]} children={markdown}></ReactMarkdown>
            </Modal>
        </Row>
    );
};

export default DatasetList;
