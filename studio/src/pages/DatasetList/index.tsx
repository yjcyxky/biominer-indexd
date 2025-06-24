import React, { useState, useEffect } from 'react';
import { Layout, Menu, List, Button, Typography, message, Input, Row, Col, Modal, Tooltip, Tag, Checkbox, MenuProps } from 'antd';
import { getDatasets, getDatasetLicense } from '@/services/biominer/datasets';
import { AimOutlined, BarChartOutlined, ClusterOutlined, FileTextFilled, InfoCircleFilled, PieChartFilled, SortAscendingOutlined, SortDescendingOutlined, UserAddOutlined } from '@ant-design/icons';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useHistory } from 'umi';
import semver from 'semver';

import './index.less';

const { Sider } = Layout;

type MenuItem = Required<MenuProps>['items'][number];

const getItem = (
    label: React.ReactNode,
    key: React.Key,
    icon?: React.ReactNode,
    children?: MenuItem[],
    type?: 'group',
): MenuItem => {
    return {
        key,
        icon,
        children,
        label,
        type,
    } as MenuItem;
}

const DatasetList: React.FC = () => {
    const [datasets, setDatasets] = useState<API.DatasetsResponse>({
        records: [],
        total: 0,
        page: 1,
        page_size: 10,
    });
    const [allTags, setAllTags] = useState<Record<'organization' | 'disease' | 'organ', string[]>>({
        organization: [],
        disease: [],
        organ: [],
    });
    const [selectedTag, setSelectedTag] = useState<string>('All');
    const [tagDatasetsMap, setTagDatasetsMap] = useState<Record<string, API.DatasetMetadata[]>>({});
    const [searchValue, setSearchValue] = useState<string>('');
    const [isModalOpen, setIsModalOpen] = useState<boolean>(false);
    const [markdownTitle, setMarkdownTitle] = useState<React.ReactNode>(null);
    const [markdown, setMarkdown] = useState<string>('');
    const [orderField, setOrderField] = useState<string>('total');
    const [orderType, setOrderType] = useState<{ name: string, total: string }>({ name: 'desc', total: 'desc' });
    const history = useHistory();
    const [loading, setLoading] = useState(false);
    const [selectedDataset, setSelectedDataset] = useState<API.DatasetMetadata | null>(null);

    useEffect(() => {
        setLoading(true);
        // TODO: Add a query param to filter the datasets by the key and version. Like only show the latest version of the dataset.
        getDatasets({ page: 1, page_size: 1000 })
            .then(res => {
                // Group the datasets by the key field and only keep the latest version.
                const groupedDatasets = res.records.reduce((acc: Record<string, API.DatasetMetadata>, ds: API.DatasetMetadata) => {
                    acc[ds.key] = ds;
                    return acc;
                }, {});

                const datasets = Object.values(groupedDatasets).sort((a, b) => semver.gt(a.version, b.version) ? -1 : 1);

                setDatasets({
                    records: datasets,
                    total: datasets.length,
                    page: 1,
                    page_size: 10,
                });
                setLoading(false);
            })
            .catch(err => message.error(err.message))
            .finally(() => {
                setLoading(false);
            });
    }, []);

    useEffect(() => {
        const tags = [...new Set(datasets.records.flatMap(ds => ds.tags))].sort();
        setAllTags({
            organization: tags.filter(tag => tag.startsWith('org:')).concat(tags.filter(tag => !tag.startsWith('org:') && !tag.startsWith('disease:') && !tag.startsWith('organ:'))),
            disease: tags.filter(tag => tag.startsWith('disease:')),
            organ: tags.filter(tag => tag.startsWith('organ:')),
        });

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
            } else if (oField === 'total') {
                return oType === 'asc' ? a.total - b.total : b.total - a.total;
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
                            total: orderType.total,
                        });
                    }}>Sort by Name</Button>
                    <Button type="text" icon={orderType.total === 'asc' ? <SortDescendingOutlined /> : <SortAscendingOutlined />} onClick={() => {
                        setOrderField('total');
                        setOrderType({
                            name: orderType.name,
                            total: orderType.total === 'asc' ? 'desc' : 'asc',
                        });
                    }}>Sort by Count</Button>
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
                <Sider width={280} className="dataset-left-sider">
                    <Menu
                        mode="inline"
                        selectedKeys={[selectedTag]}
                        onClick={e => setSelectedTag(e.key)}
                        defaultOpenKeys={['organization', 'disease', 'organ']}
                    >
                        <Menu.Item key="All">
                            <span className="tag-name">All</span>
                            <Tag className="tag-count">{datasets.records.length}</Tag>
                        </Menu.Item>
                        <Menu.Item key="No tags">
                            <span className="tag-name">No tags</span>
                            <Tag className="tag-count">{tagDatasetsMap['No tags']?.length || 0}</Tag>
                        </Menu.Item>
                        {/* Organization Menu */}
                        <Menu.SubMenu key="organization" title="Organization" icon={<ClusterOutlined />}>
                            {allTags.organization.map(tag => (
                                <Menu.Item key={tag} icon={<ClusterOutlined />}>
                                    <Tooltip title={tag.replace('org:', '')}>
                                        <span className="tag-name">{tag.replace('org:', '')}</span>
                                    </Tooltip>
                                    <Tag className="tag-count">{tagDatasetsMap[tag]?.length || 0}</Tag>
                                </Menu.Item>
                            ))}
                        </Menu.SubMenu>
                        {/* Disease Menu */}
                        <Menu.SubMenu key="disease" title="Disease" icon={<UserAddOutlined />}>
                            {allTags.disease.map(tag => (
                                <Menu.Item key={tag} icon={<UserAddOutlined />}>
                                    <Tooltip title={tag.replace('disease:', '')}>
                                        <span className="tag-name">{tag.replace('disease:', '')}</span>
                                    </Tooltip>
                                    <Tag className="tag-count">{tagDatasetsMap[tag]?.length || 0}</Tag>
                                </Menu.Item>
                            ))}
                        </Menu.SubMenu>
                        {/* Organ Menu */}
                        <Menu.SubMenu key="organ" title="Organ" icon={<AimOutlined />}>
                            {allTags.organ.map(tag => (
                                <Menu.Item key={tag} icon={<AimOutlined />}>
                                    <Tooltip title={tag.replace('organ:', '')}>
                                        <span className="tag-name">{tag.replace('organ:', '')}</span>
                                    </Tooltip>
                                    <Tag className="tag-count">{tagDatasetsMap[tag]?.length || 0}</Tag>
                                </Menu.Item>
                            ))}
                        </Menu.SubMenu>
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
                                                setMarkdownTitle(<span>Dataset Info for <Tag style={{ fontSize: 16 }}>{item.name}</Tag></span>);
                                                setMarkdown(item.description);
                                            }}>
                                            Info
                                        </Button>
                                    </Tooltip>,
                                    <Tooltip title="Coming soon...">
                                        <Button type="link" icon={<FileTextFilled />}
                                            onClick={() => {
                                                // TODO: Show the license of the dataset in another modal.
                                                setIsModalOpen(true);
                                                setMarkdownTitle(<span>Dataset License for <Tag style={{ fontSize: 16 }}>{item.name}</Tag></span>);
                                                getDatasetLicense({ key: item.key, version: item.version })
                                                    .then(res => {
                                                        setMarkdown(res);
                                                    })
                                                    .catch(err => {
                                                        setMarkdown("No license found.\n\nPlease contact the administrator for more information.");
                                                    });
                                            }}>
                                            License
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
                                <div className="sample-file-count">{item.total} {item.is_filebased ? 'files' : 'samples'}</div>
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
                title={markdownTitle}
                open={isModalOpen}
                onCancel={handleCancel}
                footer={null}
            >
                <Typography.Text style={{ fontSize: 16 }}>
                    <ReactMarkdown remarkPlugins={[remarkGfm]} children={markdown}></ReactMarkdown>
                </Typography.Text>
            </Modal>
        </Row>
    );
};

export default DatasetList;
