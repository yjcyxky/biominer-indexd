import React, { useEffect } from 'react';
import { Scene, PointLayer, Popup } from '@antv/l7';
import { GaodeMap, Mapbox } from '@antv/l7-maps';

export interface SiteInfo {
    lng: number;
    lat: number;
    name: string;
    site_name: string;
    site_description: string;
    site_link: string;
}

interface SiteMapProps {
    data?: Array<SiteInfo>;
    style?: React.CSSProperties;
}

const WorldMap: React.FC<SiteMapProps> = ({ data, style }) => {
    let popup: Popup | null = null;

    useEffect(() => {
        if (!data) {
            return;
        }

        const scene = new Scene({
            id: 'world-map',
            map: new GaodeMap({
                style: 'light',
                center: [0, 20],
                zoom: 1,
                token: '9f46ca85547dff760bbcb76a894c6cc6',
            }),
            logoVisible: false,
        });

        scene.on('loaded', () => {
            const pointLayer = new PointLayer({})
                .source(data, {
                    parser: {
                        type: 'json',
                        x: 'lng',
                        y: 'lat',
                    },
                })
                .shape('circle')
                .size(10)
                .color('#1890FF')
                .style({
                    opacity: 1,
                    strokeWidth: 1,
                    stroke: '#fff',
                })
                .active(true);

            scene.addLayer(pointLayer);

            pointLayer.on('mousemove', (e) => {
                if (e.feature) {
                    const item = e.feature;
                    popup = new Popup({
                        lngLat: e.lngLat,
                        title: item.name,
                        autoClose: true,
                        autoPan: true,
                        closeOnClick: true,
                        closeOnEsc: true,
                        closeButton: true,
                        html: `
                                <div>
                                    <span class="popup-tag">Site Name</span> ${item.site_name}<br/>
                                    <span class="popup-tag">Site Desc</span> ${item.site_description}<br/>
                                    <a href="${item.site_link}" target="_blank">Access Site</a>
                                </div>
                            `
                    });
                    scene.addPopup(popup);
                }
            });

            scene.on('resize', () => {
                const mapInstance = scene.getMapService().map as any;
                mapInstance.resize();
            });
        });

        return () => {
            scene.destroy();
        };
    }, [data]);

    return <div id="world-map" style={style}/>;
};

export default WorldMap;
