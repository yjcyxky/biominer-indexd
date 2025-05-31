import "@glideapps/glide-data-grid/dist/index.css";

import {
    DataEditor,
    GridCell,
    GridCellKind,
    GridColumn,
    Item,
    GridColumnIcon,
    DrawHeaderCallback,
    HeaderClickedEventArgs,
    Rectangle,
    Theme,
} from "@glideapps/glide-data-grid";
import { Pagination, Row } from "antd";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import ChartCard from "./ChartCard";
import ButtonCellRenderer from "./ButtonCell";
import { findIndex } from "lodash";
import { useLayer } from "react-laag";

import "./VirtualTableFaster.less";


const allCells = [
    ButtonCellRenderer,
]

interface VirtualTableFasterProps {
    dataSource: API.DatasetDataResponse['records'];
    dataDictionary: API.DataDictionary['fields'];
    isFileBased: boolean;
    loading: boolean;
    className?: string;
    scroll?: {
        y: number;
        x: number;
    };
    pagination?: {
        position: ['bottomRight'];
        pageSize: number;
        current: number;
        total: number;
        onChange: (page: number, pageSize: number) => void;
        showSizeChanger: boolean;
        showQuickJumper: boolean;
        pageSizeOptions: number[];
    };
    onCellClick?: (record: API.DatasetDataResponse['records'][number], row: number, col: API.DataDictionaryField) => void;
}

const VirtualTableFaster: React.FC<VirtualTableFasterProps> = ({
    dataSource, dataDictionary, isFileBased, loading, className, scroll, pagination, onCellClick
}) => {
    const [columns, setColumns] = useState<GridColumn[]>([]);
    const visibleRows = useMemo(() => {
        return dataSource.slice(0, pagination?.pageSize ?? 100);
    }, [dataSource, pagination?.pageSize]);

    // If fetching data is slow you can use the DataEditor ref to send updates for cells
    // once data is loaded.
    const getData = useCallback(([col, row]: Item): GridCell => {
        const record = visibleRows[row];
        let value = record[columns[col].id as keyof typeof record];

        if (value === null || value === undefined) {
            value = 'NA';
        } else if (typeof value === 'number') {
            value = value.toString();
        } else if (typeof value === 'boolean') {
            value = value.toString();
        } else if (typeof value === 'object') {
            value = JSON.stringify(value);
        }

        if (columns[col].id === 'patient_id') {
            return {
                kind: GridCellKind.Custom,
                allowOverlay: true,
                readonly: true,
                data: {
                    kind: 'button-cell',
                    backgroundColor: ["transparent", "#6572ffee"],
                    color: ["accentColor", "accentFg"],
                    title: value,
                    onClick: () => onCellClick?.(record, row, dataDictionary[col])
                },
                copyData: value
            }
        }

        return {
            kind: GridCellKind.Text,
            data: value,
            allowOverlay: false,
            displayData: value
        };
    }, [columns, visibleRows]);

    useEffect(() => {
        setColumns(dataDictionary.map((col, index) => {
            return {
                id: col.key,
                key: col.key,
                title: col.name as string,
                width: 200,
                icon: col.data_type === 'STRING' ? GridColumnIcon.HeaderString : (col.data_type === 'NUMBER' ? GridColumnIcon.HeaderNumber : GridColumnIcon.HeaderBoolean),
                themeOverride: {
                    baseFontStyle: "600 16px"
                }
            }
        }));
    }, [dataDictionary]);

    const onColumnResize = useCallback((column, newSize) => {
        setColumns(prevCols => {
            const index = findIndex(prevCols, { id: column.id });
            const newCols = [...prevCols];
            newCols.splice(index, 1, {
                ...prevCols[index],
                width: newSize,
            });
            return newCols;
        });
    }, []);

    // Configure icons for each column
    const iconPositionsRef = useRef<Record<number, { x: number; y: number; size: number }[]>>({});

    const drawHeader: DrawHeaderCallback = args => {
        const { ctx, theme, rect, column, isHovered, columnIndex } = args;

        if (columnIndex === -1) {
            return;
        }

        ctx.save();

        // Draw background
        ctx.fillStyle = isHovered ? theme.bgHeaderHovered : theme.bgHeader;
        ctx.fillRect(rect.x, rect.y, rect.width, rect.height);

        // Draw title
        ctx.fillStyle = theme.textHeader;
        ctx.font = theme.headerFontStyle;
        ctx.textBaseline = "middle";
        ctx.fillText(column.title?.toString() ?? "", rect.x + 8, rect.y + rect.height / 2);

        // Draw icons to the right
        const iconSize = 16;
        const gap = 4;
        const iconY = rect.y + (rect.height - iconSize) / 2;

        const icons = ["ℹ️", "📊"]; // replace with emojis, or draw actual icons
        const iconPositions: { x: number; y: number; size: number }[] = [];
        let iconX = rect.width - (icons.length * (iconSize + gap)) - 8;

        for (const [index, icon] of icons.entries()) {
            ctx.fillText(icon, rect.x + iconX, rect.y + iconY + iconSize / 2);
            iconPositions.push({ x: iconX, y: iconY, size: iconSize });
            iconX += iconSize + gap;
        }

        iconPositionsRef.current[columnIndex] = iconPositions;

        ctx.restore();
    };

    const detectIconClickFromEvent = (
        localX: number,
        localY: number,
        icons: { x: number; y: number; size: number }[]
    ): number => {
        for (let i = 0; i < icons.length; i++) {
            const { x, y, size } = icons[i];
            if (
                localX >= x &&
                localX <= x + size &&
                localY >= y &&
                localY <= y + size
            ) {
                return i;
            }
        }
        return -1;
    }

    const [popover, setPopover] = useState<{
        col: number;
        iconIndex: number;
        bounds: Rectangle;
    } | null>(null);

    const { layerProps, renderLayer } = useLayer({
        isOpen: !!popover,
        placement: "bottom-start",
        triggerOffset: 4,
        onOutsideClick: () => {
            console.log('onOutsideClick');
            setPopover(null);
        },
        trigger: {
            getBounds: () => ({
                left: popover?.bounds.x ?? 0,
                top: popover?.bounds.y ?? 0,
                width: popover?.bounds.width ?? 0,
                height: popover?.bounds.height ?? 0,
                right: (popover?.bounds.x ?? 0) + (popover?.bounds.width ?? 0),
                bottom: (popover?.bounds.y ?? 0) + (popover?.bounds.height ?? 0),
            })
        }
    });

    const onHeaderClicked = (col: number, e: HeaderClickedEventArgs) => {
        console.log('onHeaderClicked', col, e);
        const iconRects = iconPositionsRef.current[col];
        if (!iconRects) return;

        const iconIndex = detectIconClickFromEvent(e.localEventX, e.localEventY, iconRects);
        if (iconIndex === -1) return;

        const icon = iconRects[iconIndex];
        const absoluteX = e.bounds.x + icon.x;
        const absoluteY = e.bounds.y + icon.y + icon.size;

        // 延迟激活 Popover，避免被 onOutsideClick 立即关闭
        requestAnimationFrame(() => {
            setPopover({
                col,
                iconIndex,
                bounds: {
                    x: absoluteX,
                    y: absoluteY,
                    width: icon.size,
                    height: icon.size,
                }
            });
        });
    };

    return (
        <Row className={`datatable-table-grid ${className}`}>
            <DataEditor columns={columns} getCellContent={getData} width={scroll?.x} height={scroll?.y}
                rows={dataSource.length} keybindings={{ search: true }} getCellsForSelection={true}
                customRenderers={allCells} onHeaderClicked={onHeaderClicked} rowMarkers="number"
                onColumnResize={onColumnResize} drawHeader={drawHeader} verticalBorder={false}
            />
            {popover &&
                renderLayer(
                    <div
                        {...layerProps}
                        style={{
                            position: 'absolute',
                            top: popover.bounds.y + 10,
                            left: popover.bounds.x,
                            background: "#fff",
                            boxShadow: "0 2px 8px rgba(0,0,0,0.15)",
                            borderRadius: 4,
                            padding: 8,
                            minWidth: 160,
                            zIndex: 999,
                        }}
                    >
                        {
                            popover.iconIndex === 0 && (
                                <span>{dataDictionary.find(col => col.key === columns[popover.col].id)?.description}</span>
                            )
                        }

                        {
                            popover.iconIndex === 1 && (
                                <ChartCard className='chart-card-popover'
                                    field={dataDictionary.find(col => col.key === columns[popover.col].id) ?? dataDictionary[0]} data={dataSource}
                                    isFileBased={isFileBased} total={dataSource.length} />
                            )
                        }

                    </div>
                )
            }
            <Row className="datatable-table-grid-footer">
                <span className="datatable-table-grid-footer-text">🔢 Showing {dataSource.length} of {pagination?.total} samples</span>
                <Pagination size="small" {...pagination} />
            </Row>
        </Row>
    );
}

export default VirtualTableFaster;