import React, { useEffect, useState } from 'react';
import { WidthProvider, Responsive, Layouts } from 'react-grid-layout';
// @ts-ignore
import ChartCard from './ChartCard';

const ResponsiveGridLayout = WidthProvider(Responsive);

const VisualPanel: React.FC<{
    fields: API.DataDictionaryField[];
    data: API.DatasetDataResponse['records'];
}> = ({ fields, data }) => {
    const [visibleCharts, setVisibleCharts] = useState(fields);

    useEffect(() => {
        // 强制让图表库重新计算尺寸
        setTimeout(() => {
            window.dispatchEvent(new Event('resize'));
        }, 100);
    }, []);

    const breakpoints = {
        xxxl: 1920,   // 超宽屏 / 4K 全屏窗口
        xxl: 1600,    // MacBook Pro 16" 全屏或高分外接屏
        xl: 1400,     // 台式机大屏
        lg: 1200,     // 普通笔记本
        md: 996,
        sm: 768,
        xs: 480,
        xxs: 0,
    };

    const cols = {
        xxxl: 10,
        xxl: 8,
        xl: 6,
        lg: 4,
        md: 3,
        sm: 2,
        xs: 1,
        xxs: 1,
    };
    const generateLayout = (colsCount: number) =>
        visibleCharts.map((field, index) => ({
            i: field.key,
            x: index % colsCount,
            y: Math.floor(index / colsCount),
            w: 1,
            h: 2,
        }));

    const layouts: Layouts = {
        xxxl: generateLayout(cols.xxxl),
        xxl: generateLayout(cols.xxl),
        xl: generateLayout(cols.xl),
        lg: generateLayout(cols.lg),
        md: generateLayout(cols.md),
        sm: generateLayout(cols.sm),
        xs: generateLayout(cols.xs),
        xxs: generateLayout(cols.xxs),
    };

    const handleClose = (fieldKey: string) => {
        setVisibleCharts(prev => prev.filter(f => f.key !== fieldKey));
    };

    return (
        <ResponsiveGridLayout
            className="layout"
            layouts={layouts}
            breakpoints={breakpoints}
            cols={cols}
            rowHeight={160}
            isResizable
            isDraggable
            draggableHandle=".chart-drag-handle"
            useCSSTransforms
        >
            {visibleCharts.map(field => (
                <div key={field.key}>
                    <ChartCard field={field} data={data} onClose={() => handleClose(field.key)} />
                </div>
            ))}
        </ResponsiveGridLayout>
    );
};

export default VisualPanel;