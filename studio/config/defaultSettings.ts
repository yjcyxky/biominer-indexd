import { Settings as LayoutSettings } from '@ant-design/pro-layout';


const Settings: LayoutSettings & {
  pwa?: boolean;
  logo?: string;
} = {
  navTheme: 'light',
  // 拂晓蓝
  layout: 'top',
  contentWidth: 'Fluid',
  fixedHeader: true,
  fixSiderbar: false,
  colorWeak: false,
  title: 'BioMiner Indexd',
  pwa: false,
  logo: '/logo.png',
  iconfontUrl: '',
};

export default Settings;
