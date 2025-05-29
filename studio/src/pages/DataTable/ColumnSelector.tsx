import { InfoCircleOutlined, PercentageOutlined } from '@ant-design/icons';
import { Dropdown, Button, Input, Checkbox, Tooltip } from 'antd';
import { useState, useMemo } from 'react';

export const getDefaultSelectedKeys = (fields: API.DataDictionaryField[]) => {
    return fields.filter(field => field.order <= 5).map(field => field.key);
}

const ColumnSelector = ({ fields, selectedKeys, onChange }: { fields: API.DataDictionaryField[], selectedKeys: string[], onChange: (keys: string[]) => void }) => {
    const [search, setSearch] = useState('');

    const filteredFields = useMemo(() => {
        return fields.filter(field =>
            field.name.toLowerCase().includes(search.toLowerCase())
        );
    }, [search, fields]);

    const onToggle = (key: string, checked: boolean) => {
        onChange(checked ? [...selectedKeys, key] : selectedKeys.filter(k => k !== key));
    };

    const onToggleAll = () => {
        if (selectedKeys.length === fields.length) {
            // Reset to default selected keys
            onChange(getDefaultSelectedKeys(fields));
        } else {
            // Select all fields
            onChange(fields.map(f => f.key));
        }
    };

    const menu = (
        <div style={{ padding: 8, width: 300, backgroundColor: 'white', borderRadius: 8, border: '1px solid #d9d9d9' }}>
            <div style={{ marginBottom: 8, display: 'flex', gap: 8 }}>
                <Button size="small" onClick={onToggleAll}>
                    {selectedKeys.length === fields.length ? 'Reset to default' : `Select all (${fields.length})`}
                </Button>
                <Input.Search
                    placeholder="Search..."
                    size="small"
                    allowClear
                    onChange={e => setSearch(e.target.value)}
                    style={{ flex: 1 }}
                />
            </div>
            <div className="column-selector-header" style={{ fontWeight: 'bold', display: 'flex', padding: '4px 0' }}>
                <div style={{ flex: 1 }}>Name</div>
                <div style={{ width: 50, textAlign: 'right' }}>Freq</div>
            </div>
            <div style={{ maxHeight: 240, overflowY: 'auto' }}>
                {filteredFields.map(field => (
                    <div key={field.key} style={{ display: 'flex', alignItems: 'center', padding: '2px 0' }}>
                        <Checkbox
                            checked={selectedKeys.includes(field.key)}
                            onChange={e => onToggle(field.key, e.target.checked)}
                            style={{ flex: 1 }}
                        >
                            {field.name}
                            <Tooltip title={field.description || field.name}>
                                <InfoCircleOutlined style={{ marginLeft: 4 }} />
                            </Tooltip>
                        </Checkbox>
                        <div style={{ width: 50, textAlign: 'right', fontSize: 12, color: '#888' }}>
                            <PercentageOutlined />
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );

    return (
        <Dropdown overlay={menu} trigger={['click']}>
            <Button>Select Columns</Button>
        </Dropdown>
    );
};

export default ColumnSelector;
