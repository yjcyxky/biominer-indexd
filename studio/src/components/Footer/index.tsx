import { useIntl } from 'umi';
import { GithubOutlined } from '@ant-design/icons';
import { DefaultFooter } from '@ant-design/pro-layout';
import "./index.less";

const Footer: React.FC = () => {
  const intl = useIntl();
  const defaultMessage = intl.formatMessage({
    id: 'app.copyright.produced',
    defaultMessage: 'BioMiner ©2022 Created by Jingcheng Yang',
  });

  const currentYear = new Date().getFullYear();

  return (
    <DefaultFooter
      className="biominer-footer"
      style={{ margin: '0px' }}
      copyright={`${currentYear} ${defaultMessage}`}
      links={[
        {
          key: 'quartet-data-portal',
          title: 'Quartet Data Portal',
          href: 'https://chinese-quartet.org',
          blankTarget: true,
        },
        {
          key: 'github',
          title: <GithubOutlined />,
          href: 'https://github.com/biominer-lab',
          blankTarget: true,
        },
        {
          key: 'biominer',
          title: 'BioMiner',
          href: 'https://biominer.3steps.cn',
          blankTarget: true,
        },
      ]}
    />
  );
};

export default Footer;
