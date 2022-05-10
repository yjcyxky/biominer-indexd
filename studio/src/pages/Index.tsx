import React, { useState, useRef, useEffect } from 'react';
import { history } from 'umi';
import { Input, Card, Col, Typography, Row, message } from 'antd';
import { StatisticCard } from '@ant-design/pro-card';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import biominerAPI from '@/services/biominer';

import './index.less';

const { Search } = Input;

const isValidGuid = (guid: string) => {
  if (guid.length <= 36) {
    return false;
  }
  const regex =
    /^biominer.fudan-pgx\/[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;
  return regex.test(guid);
};

const markdown = `
BioMiner Indexd is a hash-based data indexing and tracking service providing globally unique identifiers.

If you have any questions or suggestions, please submit a pull request or an issue on [BioMiner Indexd](https://github.com/yjcyxky/biominer-indexd).

Examples: [Quartet Data Portal](https://chinese-quartet.org)
`;

const Index: React.FC = () => {
  const [fileStat, setFileStat] = useState<API.FileStatResponse>({
    total_size: -1,
    version: '',
    num_of_files: -1,
    num_of_baseid: -1,
    registry_id: '',
  });

  const checkGuid = (guid: string) => {
    if (!isValidGuid(guid)) {
      message.warn('Invalid Indexd GUID');
      return false;
    } else {
      return true;
    }
  };

  const onSearch = (value: string) => {
    if (checkGuid(value)) {
      history.push({
        pathname: '/data-repo',
        query: {
          guid: value,
        },
      });
    }
  };

  useEffect(() => {
    // Avoid request frequently, only request when the data is empty
    if (fileStat.total_size === -1 || fileStat.num_of_files === -1) {
      biominerAPI.Files.getFileStat()
        .then((res) => {
          setFileStat(res);
        })
        .catch((err) => {
          console.log(err);
        });
    }
  });

  return (
    <Row className="home-page">
      <Card className="search-container">
        <Col className="search-input">
          <span style={{ margin: '10px 0px', fontWeight: 'bold' }}>Resolve a Indexd GUID</span>
          <Search
            placeholder="Search Indexd ..."
            allowClear
            enterButton
            size="large"
            onSearch={onSearch}
          />
          <span style={{ margin: '10px 0px', textAlign: 'justify' }}>
            Type or paste a Indexd GUID, e.g.,
            <b> biominer.fudan-pgx/00006134-c655-4bbe-9144-0ee86da83902</b>, into the text box
            below. (Be sure to enter all of the characters before and after the slash. Do not
            include extra characters, or sentence punctuation marks). Clicking on a Indexd link
            takes you to one or more current URLs or other services related to a single resource. If
            the URLs or services change over time, e.g., the resource moves, this same DOI will
            continue to resolve to the correct resources or services at their new locations. (try
            this one:
            <a
              href="https://www.indexd.org/biominer.fudan-pgx/00006134-c655-4bbe-9144-0ee86da83902"
              target="_blank"
            >
              https://www.indexd.org/biominer.fudan-pgx/00006134-c655-4bbe-9144-0ee86da83902
            </a>
            )
          </span>
        </Col>
        <Row className="statistics" gutter={16}>
          <Col span={8}>
            <span>Volume</span>
            <StatisticCard
              statistic={{
                // Bytes --> GB
                value: (fileStat.total_size / 1024 / 1024 / 1024).toFixed(3),
                suffix: 'GB',
                description: null,
              }}
              style={{ padding: '0px', textAlign: 'center' }}
            />
          </Col>
          <Col span={8}>
            <span>Version</span>
            <StatisticCard
              statistic={{
                // Version
                value: fileStat.version,
                description: null,
              }}
              style={{ padding: '0px', textAlign: 'center' }}
            />
          </Col>
          <Col span={8}>
            <span>Num of Files</span>
            <StatisticCard
              statistic={{
                // Version
                value: fileStat.num_of_files,
                description: null,
              }}
              style={{ padding: '0px', textAlign: 'center' }}
            />
          </Col>
        </Row>
        <Typography.Text className="notation" style={{ fontSize: '16px' }}>
          <ReactMarkdown remarkPlugins={[remarkGfm]} children={markdown}></ReactMarkdown>
        </Typography.Text>
      </Card>
    </Row>
  );
};

export default Index;
