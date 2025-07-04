import { DownloadOutlined } from "@ant-design/icons";
import { Button, Typography } from "antd";

import { Modal } from "antd";
import { useEffect } from "react";
import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";

const downloadMarkdownFile = "/assets/data_downloader.md"

const DataDownloader: React.FC<{
    open: boolean;
    onClose: () => void;
    onDownloadMetadataTable: () => void;
    onDownloadDatafiles: () => void;
}> = ({ open, onClose, onDownloadMetadataTable, onDownloadDatafiles }) => {

    const [markdownContent, setMarkdownContent] = useState<string>('');

    useEffect(() => {
        if (downloadMarkdownFile) {
            fetch(downloadMarkdownFile).then(res => res.text()).then(text => {
                setMarkdownContent(text);
            });
        }
    }, [downloadMarkdownFile]);

    return (
        <Modal
            className="datatable-data-info-modal"
            open={open}
            width={800}
            destroyOnHidden={true}
            onCancel={onClose}
            title={<Typography.Title level={4}>Download Dataset</Typography.Title>}
            footer={<div>
                <Button type="primary" onClick={() => {
                    onDownloadMetadataTable();
                    onClose();
                }} icon={<DownloadOutlined />}>
                    Download Metadata Table
                </Button>
                <Button type="primary" onClick={() => {
                    onDownloadDatafiles();
                    onClose();
                }} icon={<DownloadOutlined />}>
                    Download Datafile Table
                </Button>
            </div>}
        >
            <Typography.Text style={{ fontSize: '16px' }}>
                <ReactMarkdown className="modal-markdown-content" remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeRaw]} children={markdownContent} />
            </Typography.Text>
        </Modal>
    )
};

export default DataDownloader;