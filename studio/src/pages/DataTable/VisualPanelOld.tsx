import React, { useEffect, useState, useCallback, useRef } from "react";
import { WidthProvider, Responsive } from "react-grid-layout";
import "./VisualPanel.less";
// @ts-ignore
import ChartCard from "./ChartCard";
import { getRecommendedChartType } from "./ChartCard";

const ResponsiveReactGridLayout = WidthProvider(Responsive);

// 单位像素值，宽高都基于此
const GRID_UNIT = 20;

const chartMap: Record<string, { w: number; h: number }> = {
    bar: { w: 15, h: 15 },
    pie: { w: 15, h: 15 },
    histogram: { w: 15, h: 15 },
    dotplot: { w: 15, h: 15 },
    default: { w: 15, h: 15 },
};

interface VisualPanelProps {
    isDraggable?: boolean;
    isResizable?: boolean;
    fields: API.DataDictionaryField[];
    data: API.DatasetDataResponse["records"];
    selectedColumns: string[];
}

const VisualPanel = (props: VisualPanelProps) => {
    const {
        isDraggable = true,
        isResizable = true,
        fields = [],
        data = [],
        selectedColumns = [],
    } = props;

    const containerRef = useRef<HTMLDivElement>(null);
    const [containerWidth, setContainerWidth] = useState(1200);
    const [layouts, setLayouts] = useState({});

    const filteredFields = fields.filter((f: API.DataDictionaryField) =>
        selectedColumns.includes(f.key)
    );

    // 保证每单位宽度 = GRID_UNIT 像素
    const dynamicCols = Math.floor(containerWidth / GRID_UNIT);
    const rowHeight = GRID_UNIT;

    const cols = {
        lg: dynamicCols,
        md: dynamicCols,
        sm: dynamicCols,
        xs: dynamicCols,
        xxs: dynamicCols,
    };

    const generateLayouts = useCallback(() => {
        const result = Object.keys(cols).reduce((memo: any, breakpoint) => {
            const col = cols[breakpoint];
            let layouts: any[] = [];

            let x = 0;
            let y = 0;
            let maxRowHeight = 0;

            filteredFields.forEach((field) => {
                const chartType = getRecommendedChartType(field);
                const { w, h } = chartMap[chartType] || chartMap.default;

                if (x + w > col) {
                    x = 0;
                    y += maxRowHeight;
                    maxRowHeight = 0;
                }

                layouts.push({
                    x,
                    y,
                    w,
                    h,
                    i: field.key,
                });

                x += w;
                maxRowHeight = Math.max(maxRowHeight, h);
            });

            memo[breakpoint] = layouts;
            return memo;
        }, {});

        return result;
    }, [filteredFields, cols]);

    useEffect(() => {
        const observer = new ResizeObserver((entries) => {
            for (let entry of entries) {
                if (entry.contentRect) {
                    setContainerWidth(entry.contentRect.width);
                }
            }
        });

        if (containerRef.current) {
            observer.observe(containerRef.current);
        }

        return () => observer.disconnect();
    }, []);

    useEffect(() => {
        setLayouts(generateLayouts());
    }, [generateLayouts, containerWidth]);

    const generateCharts = () =>
        filteredFields.map((field: API.DataDictionaryField) => (
            <div key={field.key}>
                <ChartCard field={field} data={data} />
            </div>
        ));

    const onDragStop = (layout: any, oldItem: any, newItem: any) => {
        // 可选手动更新位置或触发 compact
        setLayouts((prev) => {
            const updated = { ...prev, lg: layout };
            return updated;
        });
    }

    return (
        <div ref={containerRef} style={{ width: "100%" }}>
            <ResponsiveReactGridLayout
                className="layout"
                onLayoutChange={(newLayout: any) => setLayouts(newLayout)}
                isDraggable={isDraggable}
                isResizable={isResizable}
                rowHeight={rowHeight}
                cols={cols}
                layouts={layouts}
                margin={[0, 0]} // 必须设置为 0，避免破坏单位对等
                preventCollision={false}
                draggableHandle=".chart-drag-handle"
                compactType="vertical"
                onDragStop={onDragStop}
            >
                {generateCharts()}
            </ResponsiveReactGridLayout>
        </div>
    );
};

export default VisualPanel;
