import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Layout, Menu, List, Button, Typography, message, Input, Row, Col, Modal, Tooltip, Tag, Checkbox, MenuProps } from 'antd';
import { getDatasets, getDatasetLicense } from '@/services/biominer/datasets';
import { AimOutlined, BarChartOutlined, ClusterOutlined, FileTextFilled, InfoCircleFilled, PieChartFilled, SortAscendingOutlined, SortDescendingOutlined, UserAddOutlined } from '@ant-design/icons';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { history } from '@umijs/max';
import semver from 'semver';
// @ts-ignore
import { VariableSizeList as VirtualList } from 'react-window';

import './index.less';

const { Sider } = Layout;

// // 估算的基础高度，实际高度会根据内容动态计算
// const ESTIMATED_ITEM_HEIGHT = 131;
// const MIN_ITEM_HEIGHT = 100;
// const MAX_ITEM_HEIGHT = 300;

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
    const [filteredData, setFilteredData] = useState<API.DatasetMetadata[]>([]);
    const [selectedTag, setSelectedTag] = useState<string>('All');
    const [tagDatasetsMap, setTagDatasetsMap] = useState<Record<string, API.DatasetMetadata[]>>({});
    const [searchValue, setSearchValue] = useState<string>('');
    const [isModalOpen, setIsModalOpen] = useState<boolean>(false);
    const [markdownTitle, setMarkdownTitle] = useState<React.ReactNode>(null);
    const [markdown, setMarkdown] = useState<string>('');
    const [orderField, setOrderField] = useState<string>('total');
    const [orderType, setOrderType] = useState<{ name: string, total: string }>({ name: 'desc', total: 'desc' });
    const [loading, setLoading] = useState(false);
    const [selectedDataset, setSelectedDataset] = useState<API.DatasetMetadata | null>(null);
    const [listHeight, setListHeight] = useState(800);

    const listRef = useRef<HTMLDivElement>(null);
    const virtualListRef = useRef<VirtualList>(null);
    const itemHeightMap = useRef<Record<string, number>>({});
    const itemRefs = useRef<Record<string, HTMLDivElement>>({});

    // 计算列表高度
    useEffect(() => {
        const updateHeight = () => {
            if (listRef.current) {
                const containerHeight = listRef.current.clientHeight - 73; // 减去底部按钮区域高度
                setListHeight(Math.max(400, containerHeight));
            }
        };

        updateHeight();
        window.addEventListener('resize', updateHeight);
        return () => window.removeEventListener('resize', updateHeight);
    }, []);

    // 获取数据集
    useEffect(() => {
        setLoading(true);
        // TODO: Add a query param to filter the datasets by the key and version. Like only show the latest version of the dataset.
        getDatasets({ page: 1, page_size: 100000 })
            .then(res => {
                // Group the datasets by the key field and only keep the latest version.
                const groupedDatasets = res.records.reduce((acc: Record<string, API.DatasetMetadata>, ds: API.DatasetMetadata) => {
                    if (!acc[ds.key] || semver.gt(ds.version, acc[ds.key].version)) {
                        acc[ds.key] = ds;
                    }
                    return acc;
                }, {});

                const datasets = Object.values(groupedDatasets);

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

    // 处理标签分组
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

    // 过滤和排序数据
    const filterDatasets = useCallback((
        tag: string, searchValue: string, oField: string, oType: 'asc' | 'desc'
    ) => {
        if (!tagDatasetsMap[tag]) {
            return [];
        }

        const data = tagDatasetsMap[tag]?.filter(ds =>
            searchValue === '' || ds.name.toLowerCase().includes(searchValue.toLowerCase())
        );

        return data?.sort((a: any, b: any) => {
            if (oField === 'name') {
                return oType === 'asc' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name);
            } else if (oField === 'total') {
                return oType === 'asc' ? a.total - b.total : b.total - a.total;
            }
            return 0;
        }) || [];
    }, [tagDatasetsMap]);

    // 更新过滤后的数据
    useEffect(() => {
        const filtered = filterDatasets(selectedTag, searchValue, orderField, orderType[orderField as keyof typeof orderType] as 'asc' | 'desc');
        setFilteredData(filtered);
        // 重置虚拟列表的滚动位置
        if (virtualListRef.current) {
            virtualListRef.current.scrollToItem(0);
        }
    }, [tagDatasetsMap, selectedTag, searchValue, orderField, orderType, filterDatasets]);

    const onSearch = (value: string) => {
        setSearchValue(value);
    };

    const handleCancel = () => {
        setIsModalOpen(false);
    };

    // // 动态计算每个 item 的高度
    // const getItemSize = useCallback((index: number) => {
    //     const item = filteredData[index];
    //     if (!item) return ESTIMATED_ITEM_HEIGHT;

    //     const cachedHeight = itemHeightMap.current[item.key];
    //     if (cachedHeight) return cachedHeight;

    //     // 根据内容估算高度
    //     const descriptionLength = item.description?.length || 0;
    //     const hasGroups = item.groups.length > 0;
    //     const baseHeight = 80; // 基础高度（标题、按钮等）
    //     const groupsHeight = hasGroups ? 30 : 0;
    //     const descriptionHeight = Math.min(Math.max(descriptionLength / 5, 20), 150); // 根据描述长度估算

    //     const estimatedHeight = baseHeight + groupsHeight + descriptionHeight;
    //     return Math.min(Math.max(estimatedHeight, MIN_ITEM_HEIGHT), MAX_ITEM_HEIGHT);
    // }, [filteredData]);

    // 渲染单个 item
    const renderItem = useCallback(({ index, style }: { index: number; style: React.CSSProperties }) => {
        const item = filteredData[index];
        if (!item) return null;

        return (
            <div
                style={style}
                ref={(el) => {
                    if (el) {
                        itemRefs.current[item.key] = el;
                        // 测量实际高度
                        setTimeout(() => {
                            if (el && el.offsetHeight) {
                                const actualHeight = el.offsetHeight;
                                if (itemHeightMap.current[item.key] !== actualHeight) {
                                    itemHeightMap.current[item.key] = actualHeight;
                                    // 通知虚拟列表重新计算
                                    virtualListRef.current?.resetAfterIndex(index);
                                }
                            }
                        }, 0);
                    }
                }}
            >
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
                                <Tooltip title={<p dangerouslySetInnerHTML={{ __html: item.description }} />}
                                    classNames={{ root: 'dataset-description-tooltip' }}>
                                    <p className='dataset-description-text' dangerouslySetInnerHTML={{ __html: item.description }} />
                                </Tooltip>
                            </div>
                        }
                    />
                    <div className="sample-file-count">{item.total} {item.is_filebased ? 'files' : 'samples'}</div>
                </List.Item>
            </div>
        );
    }, [filteredData, selectedDataset, isModalOpen, markdown, markdownTitle]);

    return (
        <Row className="dataset-container">
            <Col className="dataset-header" span={24}>
                <div className="dataset-header-title">Select Datasets for Visualization & Analysis:</div>
                <Tag className="tag-count" color="blue">
                    {filteredData.length} Datasets Listed
                </Tag>
                <div className="dataset-search">
                    <Button type="text" icon={orderType.name === 'asc' ? <SortAscendingOutlined /> : <SortDescendingOutlined />} onClick={() => {
                        setOrderField('name');
                        setOrderType({
                            name: orderType.name === 'asc' ? 'desc' : 'asc',
                            total: orderType.total,
                        });
                    }}>Sort by Name</Button>
                    <Button type="text" icon={orderType.total === 'asc' ? <SortAscendingOutlined /> : <SortDescendingOutlined />} onClick={() => {
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
                        itemLayout="horizontal"
                        locale={{ emptyText: 'No datasets available' }}
                        ref={listRef}
                        style={{ height: '100%', position: 'relative' }}
                        renderItem={renderItem}
                    >
                        {/* @ts-ignore */}
                        <VirtualList
                            ref={virtualListRef}
                            height={listHeight}
                            itemCount={filteredData.length}
                            itemSize={() => { return 120 }}
                            width="100%"
                            className="dataset-virtual-list"
                            overscanCount={3}
                        >
                            {renderItem}
                        </VirtualList>
                    </List>

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
