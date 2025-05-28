import React from 'react';
import { Descriptions, Row, Tooltip, Typography } from 'antd';
import { InfoCircleOutlined } from '@ant-design/icons';

const { Title } = Typography;

interface DataInfoProps {
    data: Record<string, any>;
    dataDictionary: API.DataDictionary;
    title?: string;
}

const DataInfo: React.FC<DataInfoProps> = ({ data, dataDictionary, title = 'Patient Details' }) => {
    if (!data) return null;

    return (
        <div style={{ background: '#fff', padding: 24, borderRadius: 8 }}>
            <Title level={4} style={{ marginBottom: 16 }}>{title}</Title>
            <Row className="data-info-modal-content">
                <Descriptions
                    column={1}
                    bordered
                    size="small"
                    labelStyle={{ width: '40%', fontWeight: 500 }}
                >
                    {Object.entries(data).map(([key, value]) => (
                        <Descriptions.Item key={key} label={
                            <>
                                <span style={{ fontWeight: 500, marginRight: 4 }}>{dataDictionary.fields.find(f => f.key === key)?.name || key}</span>
                                <Tooltip title={dataDictionary.fields.find(f => f.key === key)?.description || ''}>
                                    <InfoCircleOutlined />
                                </Tooltip>
                            </>
                        }>
                            {value !== null && value !== undefined ? String(value) : '—'}
                        </Descriptions.Item>
                    ))}
                </Descriptions>
            </Row>
        </div>
    );
};

export default DataInfo;
