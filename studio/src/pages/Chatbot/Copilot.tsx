import {
    AppstoreAddOutlined,
    CloseOutlined,
    CloudUploadOutlined,
    CommentOutlined,
    CopyOutlined,
    DislikeOutlined,
    LikeOutlined,
    OpenAIFilled,
    PaperClipOutlined,
    PlusOutlined,
    ProductOutlined,
    ReloadOutlined,
    ScheduleOutlined,
    RobotOutlined,
    UserOutlined,
    DownloadOutlined,
    AppstoreOutlined,
} from '@ant-design/icons';
import {
    Attachments,
    type AttachmentsProps,
    Bubble,
    BubbleProps,
    Conversations,
    Prompts,
    Sender,
    Suggestion,
    Welcome,
    useXAgent,
    useXChat,
} from '@ant-design/x';
import type { Conversation } from '@ant-design/x/es/conversations';
import { Button, Card, Image, Popover, Space, Spin, Typography, message } from 'antd';
import dayjs from 'dayjs';
import React, { useEffect, useRef, useState } from 'react';
import MarkdownIt from 'markdown-it';

const md = new MarkdownIt({ html: true, breaks: true });

import './index.less';

type BubbleDataType = {
    role: string;
    content: string;
};

const MOCK_SESSION_LIST = [
    {
        key: '5',
        label: 'New session',
        group: 'Today',
    },
    {
        key: '4',
        label: 'What is BioMiner Indexed Data?',
        group: 'Today',
    },
    {
        key: '3',
        label: 'How to use BioMiner Indexed Data?',
        group: 'Today',
    },
    {
        key: '2',
        label: 'Any dataset contains RNA-seq, Mutation, and Clinical data for Breast Cancer?',
        group: 'Yesterday',
    },
    {
        key: '1',
        label: 'What is the relationship between TP53 variants and their clinical outcomes in Breast Cancer?',
        group: 'Yesterday',
    },
];

const MOCK_SUGGESTIONS = [
    { label: 'What is BioMiner Indexed Data?', value: 'report' },
    { label: 'How to use BioMiner Indexed Data?', value: 'draw' },
    {
        label: 'Any dataset contains RNA-seq, Mutation, and Clinical data for Breast Cancer?',
        value: 'knowledge',
        icon: <OpenAIFilled />,
        children: [
            { label: 'Any dataset contains RNA-seq, Mutation, and Clinical data for Breast Cancer?', value: 'react' },
            { label: 'Any dataset contains RNA-seq, Mutation, and Clinical data for Breast Cancer?', value: 'antd' },
        ],
    },
];

const MOCK_QUESTIONS = [
    'What is BioMiner Indexed?',
    'How to download Dataset from BioMiner Indexed?',
    'Any dataset contains RNA-seq, Mutation, and Clinical data for Breast Cancer?',
];

const AGENT_PLACEHOLDER = 'Generating content, please wait...';

type AttachmentsRef = {
    upload: (file: File) => void;
};

const renderMarkdown: BubbleProps['messageRender'] = (content) => {
    return (
        <Typography>
            {/* biome-ignore lint/security/noDangerouslySetInnerHtml: used in demo */}
            <div dangerouslySetInnerHTML={{ __html: md.render(content) }} />
        </Typography>
    );
};

interface CopilotProps {
    copilotOpen: boolean;
    setCopilotOpen: (open: boolean) => void;
    style?: React.CSSProperties;
}

const Copilot = (props: CopilotProps) => {
    const { copilotOpen, setCopilotOpen, style } = props;
    const attachmentsRef = useRef<AttachmentsRef | null>(null);
    const abortController = useRef<AbortController | null>(null);

    // ==================== State ====================

    const [messageHistory, setMessageHistory] = useState<Record<string, any>>({});

    const [sessionList, setSessionList] = useState<Conversation[]>(MOCK_SESSION_LIST);
    const [curSession, setCurSession] = useState(sessionList[0].key);

    const [attachmentsOpen, setAttachmentsOpen] = useState(false);
    const [files, setFiles] = useState<any[]>([]);

    const [inputValue, setInputValue] = useState('');

    /**
     * 🔔 Please replace the BASE_URL, PATH, MODEL, API_KEY with your own values.
     */

    // ==================== Runtime ====================

    const [agent] = useXAgent<BubbleDataType>({
        baseURL: 'https://api.x.ant.design/api/llm_siliconflow_deepSeek-r1-distill-1wen-7b',
        model: 'DeepSeek-R1-Distill-Qwen-7B',
        dangerouslyApiKey: 'Bearer sk-xxxxxxxxxxxxxxxxxxxx',
    });

    const loading = agent.isRequesting();

    const { messages, onRequest, setMessages } = useXChat({
        agent,
        requestFallback: (_, { error }) => {
            if (error.name === 'AbortError') {
                return {
                    content: 'Request is aborted',
                    role: 'assistant',
                };
            }
            return {
                content: 'Request failed, please try again!',
                role: 'assistant',
            };
        },
        transformMessage: (info) => {
            const { originMessage, chunk } = info || {};
            let currentContent = '';
            let currentThink = '';
            try {
                if (chunk?.data && !chunk?.data.includes('DONE')) {
                    const message = JSON.parse(chunk?.data);
                    currentThink = message?.choices?.[0]?.delta?.reasoning_content || '';
                    currentContent = message?.choices?.[0]?.delta?.content || '';
                }
            } catch (error) {
                console.error(error);
            }

            let content = '';

            if (!originMessage?.content && currentThink) {
                content = `<think>${currentThink}`;
            } else if (
                originMessage?.content?.includes('<think>') &&
                !originMessage?.content.includes('</think>') &&
                currentContent
            ) {
                content = `${originMessage?.content}</think>${currentContent}`;
            } else {
                content = `${originMessage?.content || ''}${currentThink}${currentContent}`;
            }

            return {
                content: content,
                role: 'assistant',
            };
        },
        resolveAbortController: (controller) => {
            abortController.current = controller;
        },
    });

    // ==================== Event ====================
    const handleUserSubmit = (val: string) => {
        onRequest({
            stream: true,
            message: { content: val, role: 'user' },
        });

        // session title mock
        if (sessionList.find((i) => i.key === curSession)?.label === 'New session') {
            setSessionList(
                sessionList.map((i) => (i.key !== curSession ? i : { ...i, label: val?.slice(0, 20) })),
            );
        }
    };

    const onPasteFile = (_: File, files: FileList) => {
        Array.from(files).forEach((file) => {
            attachmentsRef.current?.upload(file);
        });
        setAttachmentsOpen(true);
    };

    // ==================== Nodes ====================
    const chatHeader = (
        <div className="chat-header">
            <div className="header-title">✨ AI Copilot</div>
            <Space size={0}>
                <Button
                    type="text"
                    icon={<PlusOutlined />}
                    onClick={() => {
                        if (agent.isRequesting()) {
                            message.error(
                                'Message is Requesting, you can create a new conversation after request done or abort it right now...',
                            );
                            return;
                        }

                        if (messages?.length) {
                            const timeNow = dayjs().valueOf().toString();
                            abortController.current?.abort();
                            // The abort execution will trigger an asynchronous requestFallback, which may lead to timing issues.
                            // In future versions, the sessionId capability will be added to resolve this problem.
                            setTimeout(() => {
                                setSessionList([
                                    { key: timeNow, label: 'New session', group: 'Today' },
                                    ...sessionList,
                                ]);
                                setCurSession(timeNow);
                                setMessages([]);
                            }, 100);
                        } else {
                            message.error('It is now a new conversation.');
                        }
                    }}
                    className="header-button"
                />
                <Popover
                    placement="bottom"
                    style={{ maxHeight: 600 }}
                    content={
                        <Conversations
                            items={sessionList?.map((i) =>
                                i.key === curSession ? { ...i, label: `[current] ${i.label}` } : i,
                            )}
                            activeKey={curSession}
                            groupable
                            onActiveChange={async (val) => {
                                abortController.current?.abort();
                                // The abort execution will trigger an asynchronous requestFallback, which may lead to timing issues.
                                // In future versions, the sessionId capability will be added to resolve this problem.
                                setTimeout(() => {
                                    setCurSession(val);
                                    setMessages(messageHistory?.[val] || []);
                                }, 100);
                            }}
                            styles={{ item: { padding: '0 8px' } }}
                            className="conversations"
                        />
                    }
                >
                    <Button type="text" icon={<CommentOutlined />} className="header-button" />
                </Popover>
                {/* <Button
                    type="text"
                    icon={<CloseOutlined />}
                    onClick={() => setCopilotOpen(false)}
                    className="header-button"
                /> */}
            </Space>
        </div>
    );

    const chatList = (
        <div className="chat-list">
            {messages?.length ? (
                /** 消息列表 */
                <Bubble.List
                    style={{ height: '100%', paddingInline: 16 }}
                    items={messages?.map((i) => ({
                        ...i.message,
                        classNames: {
                            content: i.status === 'loading' ? 'loading-message' : '',
                        },
                        messageRender: renderMarkdown,
                        avatar: { icon: i.message.role === 'user' ? <UserOutlined /> : <RobotOutlined /> },
                        typing: i.status === 'loading' ? { step: 5, interval: 20, suffix: <>💗</> } : false,
                    }))}
                    roles={{
                        assistant: {
                            placement: 'start',
                            footer: (
                                <div style={{ display: 'flex' }}>
                                    <Button type="text" size="small" icon={<ReloadOutlined />} />
                                    <Button type="text" size="small" icon={<CopyOutlined />} />
                                    <Button type="text" size="small" icon={<LikeOutlined />} />
                                    <Button type="text" size="small" icon={<DislikeOutlined />} />
                                </div>
                            ),
                            loadingRender: () => (
                                <Space>
                                    <Spin size="small" />
                                    {AGENT_PLACEHOLDER}
                                </Space>
                            ),
                        },
                        user: { placement: 'end' },
                    }}
                />
            ) : (
                /** 没有消息时的 welcome */
                <>
                    <Welcome
                        variant="borderless"
                        title="👋 Hello, I'm BioMiner Indexed Agent"
                        description="I'm a AI agent that can help you to find the most relevant data from BioMiner Indexed Data and answer your questions."
                        className="chat-welcome"
                    />

                    <Prompts
                        vertical
                        title="I can help："
                        items={MOCK_QUESTIONS.map((i) => ({ key: i, description: i }))}
                        onItemClick={(info) => handleUserSubmit(info?.data?.description as string)}
                        style={{
                            marginInline: 16,
                        }}
                        styles={{
                            title: { fontSize: 14 },
                        }}
                    />
                </>
            )}
        </div>
    );

    const sendHeader = (
        <Sender.Header
            title="Upload File"
            styles={{ content: { padding: 0 } }}
            open={attachmentsOpen}
            onOpenChange={setAttachmentsOpen}
            forceRender
        >
            <Attachments
                ref={attachmentsRef as any}
                beforeUpload={() => false}
                items={files}
                onChange={({ fileList }: { fileList: any[] }) => setFiles(fileList)}
                placeholder={(type: string) =>
                    type === 'drop'
                        ? { title: 'Drop file here' }
                        : {
                            icon: <CloudUploadOutlined />,
                            title: 'Upload files',
                            description: 'Click or drag files to this area to upload',
                        }
                }
            />
        </Sender.Header>
    );
    const chatSender = (
        <div className="chat-send">
            <div className="send-action">
                <Button
                    icon={<AppstoreOutlined />}
                    onClick={() => handleUserSubmit('What has Ant Design X upgraded?')}
                >
                    Select Dataset
                </Button>
                <Button
                    icon={<DownloadOutlined />}
                    onClick={() => handleUserSubmit('What component assets are available in Ant Design X?')}
                >
                    Download History
                </Button>
                {/* <Button icon={<AppstoreAddOutlined />}>More</Button> */}
            </div>

            {/** 输入框 */}
            <Suggestion items={MOCK_SUGGESTIONS} onSelect={(itemVal) => setInputValue(`[${itemVal}]:`)}>
                {({ onTrigger, onKeyDown }: { onTrigger: (v: string) => void, onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void }) => (
                    <Sender
                        loading={loading}
                        value={inputValue}
                        onChange={(v: string) => {
                            onTrigger(v === '/' ? '/' : '');
                            setInputValue(v);
                        }}
                        onSubmit={() => {
                            handleUserSubmit(inputValue);
                            setInputValue('');
                        }}
                        onCancel={() => {
                            abortController.current?.abort();
                        }}
                        allowSpeech
                        placeholder="Ask or input / use skills"
                        onKeyDown={onKeyDown}
                        onFocus={() => { }}
                        onBlur={() => { }}
                        onKeyPress={() => { }}
                        header={sendHeader}
                        prefix={
                            <Button
                                type="text"
                                icon={<PaperClipOutlined style={{ fontSize: 18 }} />}
                                onClick={() => setAttachmentsOpen(!attachmentsOpen)}
                            />
                        }
                        onPasteFile={onPasteFile}
                        actions={(_, info) => {
                            const { SendButton, LoadingButton, SpeechButton } = info.components;
                            return (
                                <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                                    <SpeechButton className="speech-button" />
                                    {loading ? <LoadingButton type="default" /> : <SendButton type="primary" />}
                                </div>
                            );
                        }}
                    />
                )}
            </Suggestion>
        </div>
    );

    useEffect(() => {
        // history mock
        if (messages?.length) {
            setMessageHistory((prev) => ({
                ...prev,
                [curSession]: messages,
            }));
        }
    }, [messages]);

    return (
        <Card className="copilot-chat" style={style} title={chatHeader}>
            {/** 对话区 - 消息列表 */}
            {chatList}

            {/** 对话区 - 输入框 */}
            {chatSender}
        </Card>
    );
};

export default Copilot;