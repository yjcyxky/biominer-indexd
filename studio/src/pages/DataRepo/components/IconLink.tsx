import { ReactNode } from 'react';
import { Avatar } from 'antd';

export const IconLink = ({
  href,
  avatarSrc,
  text,
}: {
  href: string;
  avatarSrc: ReactNode;
  text: string;
}) => (
  <a
    className="quick-link"
    href={href}
    target="_blank"
    style={{ marginRight: '5px', display: 'inline-flex', alignItems: 'center' }}
  >
    <Avatar
      className="quick-link-icon"
      style={{ color: '#000000', backgroundColor: '#ffffff' }}
      icon={avatarSrc}
      alt={text}
    />
    {text}
  </a>
);
