import { Tag, Space } from 'antd';
import React from 'react';

export type QueryItem = {
    operator: string;
    field: string;
    value: string | number | boolean | string[] | number[] | boolean[];
};

export type ComposeQueryItem = {
    operator: string; // 'and' | 'or'
    items: (QueryItem | ComposeQueryItem)[];
};

// 用于不同层级颜色标识，可扩展
const colorPalette = ['purple', 'green', 'geekblue', 'volcano', 'cyan', 'magenta'];

const isQueryItem = (item: any): item is QueryItem =>
    typeof item === 'object' && 'field' in item && 'value' in item;

// 将一个 QueryItem 转成标签组件
const renderQueryItem = (item: QueryItem, color: string, key: number) => {
    let valueStr = '';
    if (Array.isArray(item.value)) {
        valueStr = item.value.join(', ');
    } else {
        valueStr = `${item.value}`;
    }
    return (
        <Tag key={key} color={color} style={{ marginBottom: 0, marginRight: 0, borderRadius: 5 }}>
            <strong>{item.field}</strong> {item.operator} <strong>{valueStr}</strong>
        </Tag>
    );
};

// 递归渲染 ComposeQueryItem
export const filters2string = (filter: ComposeQueryItem, level = 0): React.ReactNode => {
    const color = colorPalette[level % colorPalette.length];

    return (
        <Space
            size={[8, 4]}
            style={{
                marginBottom: 0,
                height: '100%',
                overflowX: 'scroll'
            }}
        >
            <span style={{ marginRight: 4, color: '#999' }}>(</span>
            {filter.items && filter.items.length > 0 && filter.items.map((item, idx) => (
                <React.Fragment key={idx}>
                    {isQueryItem(item)
                        ? renderQueryItem(item, color, idx)
                        : filters2string(item as ComposeQueryItem, level + 1)}
                    {idx !== filter.items.length - 1 && (
                        <span
                            style={{
                                margin: '0 6px',
                                color: '#666',
                                fontWeight: 500,
                                fontSize: 13,
                            }}
                        >
                            {filter.operator.toUpperCase()}
                        </span>
                    )}
                </React.Fragment>
            ))}
            <span style={{ marginLeft: 4, color: '#999' }}>)</span>
        </Space>
    );
};