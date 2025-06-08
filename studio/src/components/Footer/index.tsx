import { useIntl } from 'umi';
import { GithubOutlined } from '@ant-design/icons';
import { DefaultFooter } from '@ant-design/pro-layout';
import "./index.less";
import { useEffect } from 'react';

const Footer: React.FC = () => {
  const intl = useIntl();
  const defaultMessage = intl.formatMessage({
    id: 'app.copyright.produced',
    defaultMessage: 'BioMiner ©2022 Created by Jingcheng Yang',
  });

  const currentYear = new Date().getFullYear();

  useEffect(() => {
    const script = document.createElement('script');
    script.id = 'clustrmaps';
    script.type = 'text/javascript';
    script.src = '//cdn.clustrmaps.com/map_v2.js?cl=0090b0&w=136&t=n&d=kogEDBjnZuX1HQbHeF9VZhCT0lQnkdEJztAViYoLjOA&co=f0f2f5';
    script.async = true;

    const footer = document.getElementsByClassName('ant-pro-global-footer');
    if (footer) {
      // We ensure that the clustrmaps script is only added once
      Array.from(footer).forEach((item) => {
        item.appendChild(script);
      });
    } else {
      console.log('footer not found');
    }

    // 可选：组件卸载时移除 script
    return () => {
      const clustrmaps = document.getElementById('clustrmaps');
      if (clustrmaps) {
        clustrmaps.remove();
      }
    };
  }, []);

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
          href: 'https://github.com/open-prophetdb',
          blankTarget: true,
        },
        {
          key: 'prophetdb',
          title: 'OpenProphetDB',
          href: 'https://www.prophetdb.org',
          blankTarget: true,
        },
      ]}
    />
  );
};

export default Footer;
