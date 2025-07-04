import React from 'react';
import {
    // Message bubble
    Bubble,
    // Input box
    Sender,
} from '@ant-design/x';

const messages = [
    {
        content: 'Hello, Ant Design X!',
        role: 'user',
    },
];

const Chatbox = () => (
    <>
        <Bubble.List items={messages} />
        <Sender />
    </>
);

export default Chatbox;