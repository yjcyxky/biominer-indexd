import React, { useEffect, useRef, useState } from 'react';
import ResizeObserver from 'rc-resize-observer';
import { Spin, Table } from 'antd';
import { VariableSizeGrid as Grid } from 'react-window';
import type { TableProps } from 'antd';

interface VirtualTableProps extends TableProps<any> { }

const VirtualTable: React.FC<VirtualTableProps> = ({
    columns = [],
    dataSource = [],
    scroll,
    rowKey,
    loading,
    pagination,
    ...rest
}) => {
    const totalHeight = (scroll?.y as number) ?? 500;
    const rowHeight = 42;

    const [tableWidth, setTableWidth] = useState(0);
    const gridRef = useRef<any>();
    const headerRef = useRef<HTMLDivElement>(null);
    const [internalLoading, setInternalLoading] = useState(false);

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
    }, [columns.length, tableWidth]);

    useEffect(() => {
        if (dataSource && dataSource.length > 0) {
            setInternalLoading(true);
        }
    }, [dataSource]);

    // 自动补全列宽
    const widthColumnCount = columns.filter(c => !c.width).length;
    const mergedColumns = columns.map((col) => {
        if (col.width) return col;
        return {
            ...col,
            width: Math.floor(tableWidth / widthColumnCount),
        };
    });

    const renderVirtualList = (rawData: any[], { scrollbarSize, ref, onScroll }: { scrollbarSize: number, ref: any, onScroll: any }) => {
        ref.current = connectObject;

        return (
            <div>
                {/* 表头同步 */}
                <div ref={headerRef} style={{ display: 'flex', width: tableWidth, overflow: 'hidden' }}>
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
                    width={tableWidth}
                    onScroll={({ scrollLeft }) => {
                        onScroll({ scrollLeft });
                        if (headerRef.current) {
                            headerRef.current.scrollLeft = scrollLeft;
                        }
                    }}
                    onItemsRendered={() => {
                        if (internalLoading) {
                            setTimeout(() => {
                                setInternalLoading(false);
                            }, 30);
                        }
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
        <ResizeObserver onResize={({ width }) => setTableWidth(width - 32)}>
            <Spin spinning={internalLoading}>
                <Table
                    {...rest}
                    bordered={true}
                    columns={mergedColumns}
                    dataSource={dataSource}
                    pagination={pagination}
                    loading={loading}
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
            </Spin>
        </ResizeObserver>
    );
};

export default VirtualTable;
