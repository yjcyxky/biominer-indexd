import React from 'react';
import SiteMap, { SiteInfo } from './SiteMap';
import { Button, Card, List, Row } from 'antd';

import './index.less';

const defaultData: Array<SiteInfo> = [
    {
        // 42°22′24″N 71°07′09″W
        lng: -71.1233,
        lat: 42.3601,
        name: 'Harvard University',
        site_name: 'Surgery Department | Harvard University',
        site_description: 'Harvard University is a private research university in Cambridge, Massachusetts, United States. Founded in 1636, it is the oldest institution of higher education in the United States and one of the most prestigious universities in the world.',
        site_link: 'https://www.harvard.edu',
    },
    {
        // 31.2974° N, 121.5036° E
        lng: 121.5036,
        lat: 31.2974,
        name: 'Fudan University',
        site_name: 'PGx | Fudan University',
        site_description: 'PGx is a research group at Fudan University, Shanghai, China. It is dedicated to the research of personalized medicine.',
        site_link: 'https://www.fudan.edu.cn/pgx',
    },
    {
        // 49.4107° N, 8.7066° E
        lng: 8.7066,
        lat: 49.4107,
        name: 'Heidelberg University',
        site_name: 'Institute of Neuroscience | Heidelberg University',
        site_description: "Heidelberg University is a public research university in Heidelberg, Baden-Württemberg, Germany. Founded in 1386 on instruction of Pope Urban VI, Heidelberg is Germany's oldest university and one of the world's oldest surviving universities; it was the third university established in the Holy Roman Empire after Prague (1347) and Vienna (1365). Since 1899, it has been a coeducational institution.",
        site_link: 'https://www.heidelberg.edu',
    }
];

const SiteMapContainer: React.FC = () => {
    return (
        <Row className="site-map-container">
            <Card
                title="Why BioMiner Indexd and how to install your own instance?"
                style={{ flex: '1 1 500px', minWidth: '500px' }}
            >
                <img src={require('@/assets/images/framework.png')} alt="System Framework" style={{ width: '100%', display: 'block' }} />
                <p style={{ fontSize: '14px', color: '#999', marginTop: '16px' }}>
                    This system (BioMiner Indexd) standardizes and indexes clinical and multi-omics data through SOP-based processing, including collection, quality control, and cleaning. Omics files and metadata are converted into efficient, searchable datasets with versioning and unified indexing, supporting data integration and reuse.
                </p>
                <List
                    size="large"
                    dataSource={[
                        <span>
                            ✅ Step 1: Download the installation package <a href="https://github.com/yjcyxky/biominer-indexd" target="_blank">here.</a>
                            [Only one binary file is needed, no need to install any other software and dependencies]
                        </span>,
                        <span>✅ Step 2: Configure environment variables</span>,
                        <span>✅ Step 3: Start the service</span>,
                        <span>✅ Step 4: Prepare your datasets and visualize them on the system, learn more about the datasets <a href="https://github.com/yjcyxky/biominer-indexd/blob/main/docs/build_dataset_en.md" target="_blank">here</a></span>,
                        <span>✅ Step 5: Register your site on the platform [Optional]</span>,
                    ]}
                    renderItem={(item) => <List.Item>{item}</List.Item>}
                />
                <Button type="primary" style={{ marginTop: '16px' }} onClick={() => window.open('https://github.com/yjcyxky/biominer-indexd', '_blank')}>
                    View Full Documentation
                </Button>
            </Card>

            <Card
                title={`Site Map - Local Installations (${defaultData.length} Sites)`}
                style={{ flex: '2 1 500px', minWidth: '500px' }}
                bodyStyle={{ padding: 0, height: 'calc(100% - 58px)' }}
            >
                <SiteMap style={{ width: '100%', height: '100%', position: 'relative' }} data={defaultData} />
            </Card>
        </Row>
    );
};

export default SiteMapContainer;
