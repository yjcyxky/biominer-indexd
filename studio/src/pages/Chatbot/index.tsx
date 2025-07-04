import React from 'react';
import Copilot from './Copilot';
import { Button, Card, List, Row, Empty } from 'antd';

import './index.less';

const ChatbotContainer: React.FC = () => {
    return (
        <Row className="chatbot-container">
            <Copilot copilotOpen={true} setCopilotOpen={() => { }} style={{ flex: '1 1 500px', minWidth: '500px' }} />

            <Card
                title="Code & Tool Executor"
                style={{ flex: '2 1 500px', minWidth: '500px' }}
                bodyStyle={{ padding: 0, height: 'calc(100% - 58px)' }}
            >
                <Empty description="Please ask me anything and wait for the answer." style={{ height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column' }} />
            </Card>
        </Row>
    );
};

export default ChatbotContainer;
