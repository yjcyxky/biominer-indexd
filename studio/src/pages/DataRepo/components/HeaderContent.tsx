import { IconLink } from './IconLink';
import { Typography } from 'antd';

import { GithubOutlined, InfoCircleOutlined, QuestionCircleOutlined } from '@ant-design/icons';

const { Paragraph } = Typography;

const content = (
  <>
    <Paragraph>
      BioMiner is dedicated to building a data mining platform that integrates high-quality
      multi-omics data management, distribution and exploratory analysis.
    </Paragraph>
    {/* <Paragraph>Please add a description...</Paragraph> */}
    <div>
      <IconLink
        href="https://docs.3steps.cn"
        avatarSrc={<InfoCircleOutlined />}
        text="Product Doc"
      />
      <IconLink
        href="https://github.com/biominer-lab/docs.3steps.cn/issues"
        avatarSrc={<QuestionCircleOutlined />}
        text="Issues"
      />
      <IconLink
        href="https://github.com/biominer-lab"
        avatarSrc={<GithubOutlined />}
        text=" GitHub Repo"
      />
    </div>
  </>
);

export default content;