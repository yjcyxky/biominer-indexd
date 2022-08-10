import { IconLink } from './IconLink';
import { Typography } from 'antd';
import { useIntl } from 'umi';

import { GithubOutlined, InfoCircleOutlined, QuestionCircleOutlined } from '@ant-design/icons';

const { Paragraph } = Typography;

const HeaderContent: React.FC = () => {
  const intl = useIntl();

  return (
    <>
      <Paragraph>
        {intl.formatMessage({
          id: 'data-repo.header-content.description',
          defaultMessage:
            'BioMiner Indexd is a hash-based data indexing and tracking service providing globally unique identifiers.',
        })}
      </Paragraph>
      {/* <Paragraph>Please add a description...</Paragraph> */}
      <div>
        <IconLink
          href="/about"
          avatarSrc={<InfoCircleOutlined />}
          text={intl.formatMessage({
            id: 'data-repo.header-content.productDoc',
            defaultMessage: 'Docs',
          })}
        />
        <IconLink
          // href="https://github.com/yjcyxky/biominer-indexd/issues"
          href="https://github.com/chinese-quartet/docs.chinese-quartet.org/issues"
          avatarSrc={<QuestionCircleOutlined />}
          text={intl.formatMessage({
            id: 'data-repo.header-content.issues',
            defaultMessage: 'Issues',
          })}
        />
        <IconLink
          // href="https://github.com/yjcyxky/biominer-indexd"
          href="https://github.com/chinese-quartet/"
          avatarSrc={<GithubOutlined />}
          text={intl.formatMessage({
            id: 'data-repo.header-content.githubRepo',
            defaultMessage: 'GitHub Repo',
          })}
        />
      </div>
    </>
  );
};

export default HeaderContent;
