import React, { useEffect, useRef, useState } from 'react';
import ResizeObserver from 'rc-resize-observer';
import { Spin, Table, Space, Tooltip, Button, Popover } from 'antd';
import { VariableSizeGrid as Grid } from 'react-window';
import type { TableProps } from 'antd';
import { InfoCircleOutlined, BarChartOutlined } from '@ant-design/icons';
import ChartCard from './ChartCard';

import './VirtualTable.less';

interface VirtualTableProps extends TableProps<any> {
    dataSource: API.DatasetDataResponse['records'];
    onCellClick?: (record: API.DatasetDataResponse['records'][number], row: number, col: API.DataDictionaryField) => void;
    dataDictionary: API.DataDictionaryField[];
    isFileBased: boolean;
}

const VirtualTable: React.FC<VirtualTableProps> = ({
    dataSource,
    scroll,
    size,
    rowKey,
    pagination,
    dataDictionary,
    onCellClick,
    isFileBased,
    ...rest
}) => {
    const totalHeight = (scroll?.y as number) ?? 500;
    const rowHeight = 42;

    const gridRef = useRef<any>();
    const headerRef = useRef<HTMLDivElement>(null);
    const [columns, setColumns] = useState<any[]>([]);

    useEffect(() => {
        const columnDefs = dataDictionary.map((col) => ({
            title: (
                <div style={{ display: 'flex', justifyContent: 'space-between', whiteSpace: 'nowrap', gap: 4, alignItems: 'center' }}>
                    <span>{col.name}</span>
                    <Space>
                        <Tooltip title={col.description}>
                            <Button size="small" icon={<InfoCircleOutlined />} />
                        </Tooltip>
                        <Popover content={<ChartCard className='chart-card-popover'
                            field={col} data={dataSource} isFileBased={isFileBased} total={dataSource.length} />}
                            trigger="click" destroyTooltipOnHide>
                            <Button size="small" icon={<BarChartOutlined />} type="primary" />
                        </Popover>
                    </Space>
                </div>
            ),
            dataIndex: col.key,
            key: col.key,
            render: (value: any, record: any, rowIndex: number) => {
                if (col.key === 'patient_id') {
                    return <a onClick={() => onCellClick?.(record, rowIndex, col)}>{value}</a>;
                }
                return value;
            }
        }));

        setColumns(columnDefs);
    }, [dataDictionary, dataSource]);

    const [connectObject] = useState(() => {
        const obj = {};
        Object.defineProperty(obj, 'scrollLeft', {
            get: () => null,
            set: (scrollLeft: number) => {
                if (gridRef.current) {
                    gridRef.current.scrollTo({ scrollLeft });
                }
                if (headerRef.current) {
                    headerRef.current.scrollLeft = scrollLeft;
                }
            },
        });
        return obj;
    });

    useEffect(() => {
        if (gridRef.current) {
            console.log('🛠 Recalculating columns for Grid');
            gridRef.current.resetAfterColumnIndex(0, true);
        }
    }, [columns.length]);

    // 自动补全列宽
    const widthColumnCount = columns.filter(c => !c.width).length;
    const mergedColumns = columns.map((col) => {
        if (col.width) return col;
        return {
            ...col,
            width: Math.floor(scroll?.x as number / widthColumnCount),
        };
    });

    const renderVirtualList = (rawData: any[], { scrollbarSize, ref, onScroll }: { scrollbarSize: number, ref: any, onScroll: any }) => {
        ref.current = connectObject;

        return (
            <div>
                {/* 表头同步 */}
                <div ref={headerRef} style={{ display: 'flex', width: scroll?.x as number, overflow: 'hidden' }}>
                    {mergedColumns.map((column, index) => (
                        <div
                            key={index}
                            style={{
                                width: column.width,
                                padding: 8,
                                background: '#fafafa',
                                borderBottom: '1px solid #f0f0f0',
                                fontWeight: 500,
                            }}
                        >
                            {column.title}
                        </div>
                    ))}
                </div>

                {/* 虚拟表格 */}
                <Grid
                    className="datatable-table-grid"
                    ref={gridRef}
                    columnCount={mergedColumns.length}
                    columnWidth={(index) => mergedColumns[index].width as number}
                    height={totalHeight}
                    rowCount={rawData.length}
                    rowHeight={() => rowHeight}
                    width={scroll?.x as number}
                    onScroll={({ scrollLeft }) => {
                        onScroll({ scrollLeft });
                        if (headerRef.current) {
                            headerRef.current.scrollLeft = scrollLeft;
                        }
                    }}
                    onItemsRendered={() => {
                        // TODO: add loading state
                    }}
                >
                    {({ columnIndex, rowIndex, style }) => {
                        const column = mergedColumns[columnIndex];
                        const record = rawData[rowIndex];
                        // @ts-ignore
                        const value = record[column.dataIndex as string];
                        return (
                            <div
                                className={rowIndex % 2 === 0 ? 'datatable-table-grid-row-even' : 'datatable-table-grid-row-odd'}
                                style={{
                                    ...style,
                                    padding: '8px',
                                    whiteSpace: 'nowrap',
                                    overflow: 'hidden',
                                    textOverflow: 'ellipsis',
                                    textAlign: `${column.align}` as 'left' | 'center' | 'right' | 'justify' | undefined,
                                }}
                            >
                                {typeof column.render === 'function'
                                    ? column.render(value, record, rowIndex)
                                    : value}
                            </div>
                        );
                    }}
                </Grid>
            </div>
        );
    };

    return (
        <Table
            {...rest}
            size={size}
            bordered={true}
            columns={mergedColumns}
            dataSource={dataSource}
            pagination={pagination}
            rowKey={rowKey}
            scroll={scroll}
            showHeader={false}
            components={{
                // @ts-ignore
                body: renderVirtualList,
            }}
            summary={() => (
                <Table.Summary fixed>
                    <Table.Summary.Row style={{ fontSize: 16, fontWeight: 500 }}>
                        <Table.Summary.Cell index={0} colSpan={columns.length}>
                            {/* @ts-ignore */}
                            🔢 Showing {dataSource.length} of {pagination?.total} samples
                        </Table.Summary.Cell>
                    </Table.Summary.Row>
                </Table.Summary>
            )}
        />
    );
};

export default VirtualTable;
