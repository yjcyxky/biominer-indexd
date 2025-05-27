import React, { useState, useEffect } from 'react';
import { Layout, Menu, List, Button, Typography, message, Input, Row, Col, Modal } from 'antd';
import { getDatasets } from '@/services/biominer/Datasets';
import './index.less';
import { DownloadOutlined, FileTextFilled, InfoCircleFilled, PieChartFilled } from '@ant-design/icons';
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
    const history = useHistory();

    useEffect(() => {
        getDatasets({ page: 1, page_size: 1000 })
            .then(res => setDatasets(res))
            .catch(err => message.error(err.message));
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

    const filteredDatasets = (tag: string, searchValue: string) => {
        if (searchValue === '') {
            return tagDatasetsMap[tag];
        }
        return tagDatasetsMap[tag].filter(ds => ds.name.includes(searchValue));
    };

    return (
        <Row className="dataset-container">
            <Col className="dataset-header" span={24}>
                <div className="dataset-header-title">Select Datasets for Visualization & Analysis:</div>
                <div className="dataset-search">
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
                            All <span className="count">({datasets.records.length})</span>
                        </Menu.Item>
                        <Menu.Item key="No tags">
                            No tags <span className="count">({tagDatasetsMap['No tags']?.length || 0})</span>
                        </Menu.Item>
                        {allTags.map(tag => (
                            <Menu.Item key={tag}>
                                {tag} <span className="count">({tagDatasetsMap[tag]?.length || 0})</span>
                            </Menu.Item>
                        ))}
                    </Menu>
                </Sider>

                <List
                    className="dataset-right-sider"
                    itemLayout="horizontal"
                    dataSource={filteredDatasets(selectedTag, searchValue) || []}
                    locale={{ emptyText: 'No datasets available' }}
                    renderItem={item => (
                        <List.Item
                            className="dataset-item"
                            actions={[
                                <Button type="link" icon={<FileTextFilled />}
                                    onClick={() => window.open(`https://pubmed.ncbi.nlm.nih.gov/${item.pmid}`, '_blank')}>
                                    Cite
                                </Button>,
                                <Button type="link" icon={<InfoCircleFilled />}
                                    onClick={() => {
                                        // TODO: Show the markdown content of the dataset in another modal.
                                        setIsModalOpen(true);
                                        setMarkdown(item.description);
                                    }} disabled>
                                    Info
                                </Button>,
                                <Button type="link" icon={<DownloadOutlined />}
                                    onClick={() => {
                                        // TODO: Redirect to the data-repo page.
                                    }} disabled>
                                    Download
                                </Button>,
                                <Button type="link" icon={<PieChartFilled />} onClick={() => {
                                    history.push(`/datatable/${item.key}`);
                                }}>
                                    View
                                </Button>,
                            ]}
                        >
                            <List.Item.Meta
                                title={<Typography.Text strong>{item.name}</Typography.Text>}
                                description={<p dangerouslySetInnerHTML={{ __html: item.description }} />}
                            />
                            <div className="sample-count">{item.num_of_samples} samples</div>
                        </List.Item>
                    )}
                />
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
