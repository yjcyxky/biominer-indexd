import { InfoCircleOutlined, PercentageOutlined, SettingOutlined } from '@ant-design/icons';
import { Dropdown, Button, Input, Checkbox, Tooltip } from 'antd';
import { useState, useMemo } from 'react';

export const getDefaultSelectedKeys = (fields: API.DataDictionaryField[]) => {
    return fields.filter(field => field.order <= 5).map(field => field.key).slice(0, 6);
}

const MAX_SHOW_KEYS = 500;
const MAX_SELECTED_KEYS = 10;

interface ColumnSelectorProps {
    title?: string,
    className?: string,
    fields: API.DataDictionaryField[],
    selectedKeys: string[],
    onChange: (keys: string[]) => void
}

const ColumnSelector: React.FC<ColumnSelectorProps> = ({ className, title, fields, selectedKeys, onChange }) => {
    const [search, setSearch] = useState('');

    const filteredFields = useMemo(() => {
        return fields.filter(field =>
            field.name.toLowerCase().includes(search.toLowerCase())
        );
    }, [search, fields]);

    // Limit the fields to render to prevent performance issues
    const fieldsToRender = useMemo(() => {
        return filteredFields.slice(0, MAX_SHOW_KEYS);
    }, [filteredFields]);

    const onToggle = (key: string, checked: boolean) => {
        onChange(checked ? [...selectedKeys, key] : selectedKeys.filter(k => k !== key));
    };

    const onToggleAll = () => {
        if (selectedKeys.length === fields.length || selectedKeys.length >= MAX_SELECTED_KEYS) {
            // Reset to default selected keys
            onChange(getDefaultSelectedKeys(fields));
        } else {
            // Select all fields up to MAX_SELECTED_KEYS
            const keysToSelect = fields.map(f => f.key).slice(0, MAX_SELECTED_KEYS);
            onChange(keysToSelect);
        }
    };

    const menu = (
        <div style={{ padding: 8, width: 400, backgroundColor: 'white', borderRadius: 8, border: '1px solid #d9d9d9' }}>
            <div style={{ marginBottom: 8, display: 'flex', gap: 8 }}>
                <Tooltip title={`${fields.length} columns in total, Allow up to select ${MAX_SELECTED_KEYS} columns`}>
                    <Button size="small" onClick={onToggleAll}>
                        {selectedKeys.length === fields.length ? 'Reset to default' : `Select all (max ${MAX_SELECTED_KEYS} | ${fields.length} in total)`}
                    </Button>
                </Tooltip>
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
                {fieldsToRender.map(field => (
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
                {filteredFields.length > MAX_SHOW_KEYS && (
                    <div style={{ padding: '8px 0', textAlign: 'center', color: '#888', fontSize: 12 }}>
                        Showing {MAX_SHOW_KEYS} of {filteredFields.length} fields. Use search to find more.
                    </div>
                )}
            </div>
        </div>
    );

    return (
        <Dropdown overlay={menu} trigger={['click']}>
            <Button icon={<SettingOutlined />} className={`${className}`}>{title ?? 'Select Columns'}</Button>
        </Dropdown>
    );
};

export default ColumnSelector;
