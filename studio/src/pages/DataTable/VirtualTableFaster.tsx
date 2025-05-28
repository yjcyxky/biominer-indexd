import "@glideapps/glide-data-grid/dist/index.css";

import {
    DataEditor,
    GridCell,
    GridCellKind,
    GridColumn,
    Item,
    GridColumnIcon,
} from "@glideapps/glide-data-grid";
import { Divider, Pagination, Popover, Row, Tooltip } from "antd";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useLayer } from "react-laag";
import { BarChartOutlined, InfoCircleOutlined } from "@ant-design/icons";
import ChartCard from "./ChartCard";
import ButtonCellRenderer from "./ButtonCell";

const allCells = [
    ButtonCellRenderer,
]

interface VirtualTableFasterProps {
    dataSource: API.DatasetDataResponse['records'];
    dataDictionary: API.DataDictionary['fields'];
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
    dataSource, dataDictionary, loading, className, scroll, pagination, onCellClick
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

        console.log('Get data', columns[col].id, value);
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
                hasMenu: true,
            }
        }));
    }, [dataDictionary]);

    const [menu, setMenu] = useState<{ col: number; bounds: { x: number; y: number; width: number; height: number } }>();
    const isOpen = !!menu;
    const { layerProps, renderLayer } = useLayer({
        isOpen,
        auto: true,
        placement: "bottom-end",
        triggerOffset: 2,
        onOutsideClick: () => setMenu(undefined),
        trigger: {
            getBounds: () => ({
                left: menu?.bounds.x ?? 0,
                top: menu?.bounds.y ?? 0,
                width: menu?.bounds.width ?? 0,
                height: menu?.bounds.height ?? 0,
                right: (menu?.bounds.x ?? 0) + (menu?.bounds.width ?? 0),
                bottom: (menu?.bounds.y ?? 0) + (menu?.bounds.height ?? 0),
            })
        }
    });

    const onHeaderMenuClick = useCallback((col: number, bounds: { x: number; y: number; width: number; height: number }) => {
        setMenu({ col, bounds });
    }, []);

    return (
        <Row className={`datatable-table-grid ${className}`}>
            <DataEditor columns={columns} getCellContent={getData} width={scroll?.x} height={scroll?.y}
                rows={dataSource.length} keybindings={{ search: true }} getCellsForSelection={true}
                rowMarkers="number" onHeaderMenuClick={onHeaderMenuClick} customRenderers={allCells}
            />
            {isOpen &&
                renderLayer(
                    <div {...layerProps} className="gdg-column-menu"
                        style={{
                            background: 'white', boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
                            borderRadius: 4, padding: 8,
                            position: 'absolute',
                            top: menu?.bounds.y + menu?.bounds.height + 10,
                            left: menu?.bounds.x,
                            zIndex: 1000
                        }}>
                        <div onClick={() => setMenu(undefined)}>
                            <Tooltip title={dataDictionary[menu?.col ?? 0].description}>
                                <InfoCircleOutlined />
                                <span>Column Description</span>
                            </Tooltip>
                        </div>
                        <Divider style={{ margin: '0' }} />
                        <div onClick={() => setMenu(undefined)}>
                            <Popover content={<ChartCard className='chart-card-popover' field={dataDictionary[menu?.col ?? 0]} data={dataSource} total={dataSource.length} />}
                                trigger="hover" destroyTooltipOnHide>
                                <BarChartOutlined />
                                <span>Column Chart</span>
                            </Popover>
                        </div>
                    </div>
                )}
            <Row className="datatable-table-grid-footer">
                <span className="datatable-table-grid-footer-text">🔢 Showing {dataSource.length} of {pagination?.total} samples</span>
                <Pagination size="small" {...pagination} />
            </Row>
        </Row>
    );
}

export default VirtualTableFaster;