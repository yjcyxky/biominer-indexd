import React, { useEffect, useRef, useState } from 'react';
import Muuri from 'muuri';
import GroupedChartCard from './GroupedChartCard';
import { getRecommendedGroupedChartType } from './GroupedChartCard';
import './GroupedVisualPanel.less';
// @ts-ignore
import { Resizable } from 're-resizable';
import { Empty, Row } from 'antd';

const GRID_UNIT = 5;

const chartMap: Record<string, { w: number; h: number }> = {
    bar: { w: 128, h: 64 },
    box: { w: 128, h: 64 },
    violin: { w: 128, h: 64 },
    scatter: { w: 128, h: 64 },
    histogram: { w: 128, h: 64 },
    summary: { w: 128, h: 64 },
    kmplot: { w: 128, h: 64 },
    default: { w: 128, h: 64 },
};

interface VisualPanelProps {
    fields: API.DataDictionaryField[];
    data: API.DatasetDataResponse['records'];
    total: number;
    selectedColumns: string[];
    yField: API.DataDictionaryField | undefined;
    idColumnName: string;
    onClose?: (field: API.DataDictionaryField) => void;
}

const GroupedVisualPanel: React.FC<VisualPanelProps> = ({
    fields, data, total, selectedColumns,
    yField, onClose,
    idColumnName
}) => {
    console.log('yField', yField, 'fields', fields, 'selectedColumns', selectedColumns, 'data', data, 'total', total);
    const gridRef = useRef<HTMLDivElement>(null);
    const muuriRef = useRef<Muuri | null>(null);

    const [resizing, setResizing] = useState<boolean>(false);
    const [filteredFields, setFilteredFields] = useState<API.DataDictionaryField[]>(fields);
    const [sizes, setSizes] = useState<Record<string, { width: number; height: number }>>({});

    const getInitialSize = (fieldKey: string, chartType: string) => {
        if (sizes[fieldKey]) return sizes[fieldKey];
        const { w, h } = chartMap[chartType] || chartMap.default;
        return {
            width: w * GRID_UNIT,
            height: h * GRID_UNIT,
        };
    };

    useEffect(() => {
        if (gridRef.current) {
            muuriRef.current = new Muuri(gridRef.current, {
                dragEnabled: true,
                layoutOnInit: true,
                layoutDuration: 300,
                layoutEasing: 'ease',
                dragSort: true,
                dragSortPredicate: {
                    threshold: 50,
                    action: 'move'
                },
                dragStartPredicate: {
                    distance: 10,
                    delay: 100,
                },
                dragHandle: '.chart-drag-handle'
            });
        }

        return () => {
            muuriRef.current?.destroy();
        };
    }, [filteredFields]);

    // useEffect(() => {
    //     setFilteredFields([...fields.filter((f) => selectedColumns.includes(f.key))]);
    // }, [fields, selectedColumns]);

    if (!yField) {
        return <Empty description="Please specify the right yField, it should be one of the selected columns" />
    }

    return (
        <Row className="grouped-visual-panel-container">
            <div className="muuri-grid" ref={gridRef}>
                {filteredFields.map((field) => {
                    const chartType = getRecommendedGroupedChartType(field, yField);
                    const { width, height } = getInitialSize(field.key, chartType);
                    const style = {
                        width: `${width}px`,
                        height: `${height}px`,
                    };
                    const xyFields = [field, yField];
                    return (
                        <div className="item" key={field.key} style={style}>
                            <div className="item-content">
                                <Resizable
                                    defaultSize={{
                                        width: width,
                                        height: height,
                                    }}
                                    minWidth={5 * GRID_UNIT}
                                    minHeight={5 * GRID_UNIT}
                                    onResizeStart={() => setResizing(true)}
                                    onResizeStop={(e: any, direction: any, ref: any) => {
                                        setResizing(false);
                                        const newWidth = ref.offsetWidth;
                                        const newHeight = ref.offsetHeight;

                                        setSizes(prev => ({
                                            ...prev,
                                            [field.key]: {
                                                width: newWidth,
                                                height: newHeight,
                                            },
                                        }));

                                        muuriRef.current?.refreshItems();
                                        muuriRef.current?.layout(true);
                                    }}
                                    handleClasses={{ bottomRight: 'resizable-handle' }}
                                    enable={{
                                        top: false,
                                        right: true,
                                        bottom: true,
                                        left: false,
                                        topRight: false,
                                        bottomRight: true,
                                        topLeft: false,
                                        bottomLeft: false,
                                    }}
                                >
                                    <div className="resizable-container">
                                        {resizing && <div className="resize-indicator" />}
                                        <GroupedChartCard fields={xyFields} allFields={filteredFields} groupByField={field.key}
                                            data={data} total={total} idColumnName={idColumnName}
                                            onClose={() => {
                                                // TODO: remove the chart
                                            }} resize={() => {
                                                muuriRef.current?.refreshItems().layout(true);
                                            }} selectedColumns={selectedColumns}
                                            allowChangeChartType={true}
                                        />
                                    </div>
                                </Resizable>
                            </div>
                        </div>
                    );
                })}
            </div>
        </Row>
    );
};

export default GroupedVisualPanel;
