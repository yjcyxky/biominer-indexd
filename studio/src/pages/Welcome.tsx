import React from 'react';
import { PageContainer } from '@ant-design/pro-layout';
import { Card, Alert, Typography } from 'antd';
import { useIntl, FormattedMessage } from 'umi';
import styles from './Welcome.less';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

const CodePreview: React.FC = ({ children }) => (
  <pre className={styles.pre}>
    <code>
      <Typography.Text copyable>{children}</Typography.Text>
    </code>
  </pre>
);

const markdown = `
### Intro
BioMiner Indexd is a hash-based data indexing and tracking service providing globally unique identifiers.

### Features 

- [x] Manage & retrieve files: index each file by UUID (e.g. biominer.fudan-pgx/b14563ac-dbc1-49e8-b484-3dad89de1a54) and record all repository locations, file names, MD5 values, DOI numbers, repository links, version numbers, sizes, etc. of files 

- [x] Track file location: provide a mechanism to register & track file location, for the same file released in multiple repositories (OSS, S3, GSA, NODE, SRA, ENA.) 

- [x] Manage multi-version files: provide Base UUID indexing of different versions of files (i.e., get the Base UUID, you can query all the historical versions of a file in the system) for different versions of Pipeline analysis to generate multiple versions of Level2/3 files. 

- [x] Bulk get download links: query specified files by UUID/MD5 and get download links of specified repositories. It is better to use with [biopoem](https://github.com/yjcyxky/biopoem). 

- [ ] Track file status: whether the file is in the index, or has been deleted, or has been updated, or can be downloaded. 

- [ ] More features... 


If you have any questions or suggestions, please submit a pull request or an issue on [BioMiner Indexd](https://github.com/yjcyxky/biominer-indexd).

### How to download the omics data files?

Download biominer-aget binary from [BioMiner Aget for Linux](https://biominer-indexd.oss-cn-shanghai.aliyuncs.com/biominer-aget/biominer-aget_x86-64_linux) or [BioMiner Aget for Mac](https://biominer-indexd.oss-cn-shanghai.aliyuncs.com/biominer-aget/biominer-aget_x86-64_macosx)

Copy the biominer-aget binary into /usr/bin/biominer-aget or any other directory which in PATH variable.

e.g. you want to download the file with UUID 00006134-c655-4bbe-9144-0ee86da83902 from the repository biominer.fudan-pgx, you can run the following command:

\`\`\`bash
biominer-aget --guid biominer.fudan-pgx/00006134-c655-4bbe-9144-0ee86da83902 --output-dir ~/Downloads/ --repo gsa --chunk_size 1m --concurrency 1000
\`\`\`

> Please note that:
> 1. The \`chunk_size\` and \`concurrency\` parameters are related with the download speed. **There may be tens or hundreds of times the difference, so it's worth taking some time to find the best one.**
> 2. The biominer-aget binary is not available for Windows.

\`\`\`bash
$ biominer-aget --help
Biominer Aget 0.3.7
Jingcheng Yang <yjcyxky@163.com>
An Index Engine for Omics Data Files

USAGE:
    biominer-aget_x86-64_macosx [FLAGS] [OPTIONS] --guid <guid>

FLAGS:
    -D, --debug      Activate debug mode short and long flags (-D, --debug) will be deduced from the field's name
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --api-server <api-server>      Where to find the biominer api server
    -k, --chunk_size <chunk_size>      The number ofinterval length of each concurrent request [default: '50m']
        --concurrency <concurrency>    The number of concurrency request [default: 10]
        --dns-timeout <dns-timeout>    DNS Timeout(seconds) of request [default: 10]
    -g, --guid <guid>                  Which file you want to download
    -o, --output-dir <output-dir>       [default: ./]
    -p, --password <password>          Password for the biominer api server [default: anonymous]
    -r, --repo <repo>                   [default: node]  [possible values: node, gsa, s3, oss, minio]
        --retries <retries>            The maximum times of retring [default: 0]
        --retry-wait <retry-wait>      The seconds between retries [default: 0]
    -t, --timeout <timeout>            Timeout(seconds) of request [default: 60]
    -u, --username <username>          Username for the biominer-indexd api server [default: anonymous]
\`\`\`
`;

const Welcome: React.FC = () => {
  const intl = useIntl();

  return (
    <PageContainer>
      <Card>
        <Alert
          message={intl.formatMessage({
            id: 'pages.welcome.alertMessage',
            defaultMessage: 'Faster download tool have been released.',
          })}
          type="success"
          showIcon
          banner
          style={{
            margin: -12,
            marginBottom: 24,
          }}
        />
        <Typography.Text style={{ fontSize: '16px' }}>
          <ReactMarkdown remarkPlugins={[remarkGfm]} children={markdown}></ReactMarkdown>
        </Typography.Text>
      </Card>
    </PageContainer>
  );
};

export default Welcome;
