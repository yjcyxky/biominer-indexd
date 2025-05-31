import React, { useEffect, useRef, useState } from 'react';
import Muuri from 'muuri';
import ChartCard from './ChartCard';
import { getRecommendedChartType } from './ChartCard';
import './VisualPanel.less';
// @ts-ignore
import { Resizable } from 're-resizable';
import { Row } from 'antd';

const GRID_UNIT = 20;

const chartMap: Record<string, { w: number; h: number }> = {
    id: { w: 16, h: 16 },
    table: { w: 32, h: 16 },
    bar: { w: 32, h: 16 },
    pie: { w: 16, h: 16 },
    histogram: { w: 32, h: 16 },
    dotplot: { w: 16, h: 16 },
    kaplan_meier: { w: 32, h: 16 },
    default: { w: 16, h: 16 },
};

interface VisualPanelProps {
    fields: API.DataDictionaryField[];
    data: API.DatasetDataResponse['records'];
    isFileBased: boolean;
    total: number;
    selectedColumns: string[];
    onClose?: (field: API.DataDictionaryField) => void;
}

const VisualPanel: React.FC<VisualPanelProps> = ({ fields, data, isFileBased, total, selectedColumns, onClose }) => {
    const gridRef = useRef<HTMLDivElement>(null);
    const muuriRef = useRef<Muuri | null>(null);

    const [resizing, setResizing] = useState<boolean>(false);
    const [filteredFields, setFilteredFields] = useState<API.DataDictionaryField[]>([]);
    const [sizes, setSizes] = useState<Record<string, { width: number; height: number }>>({});

    const extraField: API.DataDictionaryField = {
        key: '__summary',
        name: 'Summary',
        data_type: 'NUMBER',
        allowed_values: [],
        notes: '',
        order: 0,
        description: `Total number of ${isFileBased ? 'files' : 'samples'} in the dataset and loaded ${isFileBased ? 'files' : 'samples'}`,
    }

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

    useEffect(() => {
        setFilteredFields([extraField, ...fields.filter((f) => selectedColumns.includes(f.key))]);
    }, [fields, selectedColumns]);

    return (
        <Row className="visual-panel-container">
            <div className="muuri-grid" ref={gridRef}>
                {filteredFields.map((field) => {
                    const chartType = getRecommendedChartType(field, data.length, total, selectedColumns);
                    const { width, height } = getInitialSize(field.key, chartType);
                    const style = {
                        width: `${width}px`,
                        height: `${height}px`,
                    };
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
                                        <ChartCard field={field} data={data} isFileBased={isFileBased} total={total}
                                            onClose={() => {
                                                onClose?.(field);
                                            }} resize={() => {
                                                muuriRef.current?.refreshItems().layout(true);
                                            }} selectedColumns={selectedColumns} />
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

export default VisualPanel;
