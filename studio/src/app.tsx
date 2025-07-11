import type { Settings as LayoutSettings } from '@ant-design/pro-layout';
import type { RunTimeLayoutConfig } from '@umijs/max';
import { history, Link, RequestConfig } from '@umijs/max';
import RightContent from '@/components/RightContent';
import Footer from '@/components/Footer';
// import { currentUser as queryCurrentUser } from './services/biominer-indexd-studio/api';
import {
  BookOutlined, DatabaseOutlined, FileOutlined, GlobalOutlined,
  HomeOutlined, LinkOutlined, MessageOutlined, QuestionCircleOutlined, TableOutlined
} from '@ant-design/icons';
import defaultSettings from '../config/defaultSettings';
import routes from '../config/routes';

const iconMap: Record<string, React.ReactNode> = {
  home: <HomeOutlined />,
  database: <DatabaseOutlined />,
  file: <FileOutlined />,
  global: <GlobalOutlined />,
  'question-circle': <QuestionCircleOutlined />,
  message: <MessageOutlined />,
  table: <TableOutlined />,
  // You must add the icon which you want to use in the menu here.
};

function mapIcon(name: string) {
  return iconMap[name] || null;
}

const isDev = process.env.NODE_ENV === 'development';
let prefix = ''
if (window.location.pathname == '/index.html') {
  prefix = ''
} else {
  prefix = window.location.pathname.replace('/index.html', '')
}
const apiPrefix = process.env.UMI_APP_API_PREFIX ? process.env.UMI_APP_API_PREFIX : prefix;
const loginPath = '/user/login';

console.log("apiPrefix", process.env, apiPrefix);

export const request: RequestConfig = {
  timeout: 30000,
  baseURL: apiPrefix,
  errorConfig: {
    errorHandler: (error: any) => {
      // console.log("error", error);
      // We don't want to handle the error globally, just throw it.
      throw error;
    },
    errorThrower: (res: any) => {
      throw res;
    },
  },
  requestInterceptors: [],
  responseInterceptors: [],
};

/**
 * @see  https://umijs.org/zh-CN/plugins/plugin-initial-state
 * */
export async function getInitialState(): Promise<{
  settings?: Partial<LayoutSettings>;
  currentUser?: API.CurrentUser;
  loading?: boolean;
  fetchUserInfo?: () => Promise<API.CurrentUser | undefined>;
}> {
  const fetchUserInfo = async () => {
    // try {
    //   const msg = await queryCurrentUser();
    //   return msg.data;
    // } catch (error) {
    //   history.push(loginPath);
    // }
    return undefined;
  };
  // 如果不是登录页面，执行
  // if (history.location.pathname !== loginPath) {
  //   const currentUser = await fetchUserInfo();
  //   return {
  //     fetchUserInfo,
  //     currentUser,
  //     settings: defaultSettings,
  //   };
  // }
  delete defaultSettings.logo;
  return {
    fetchUserInfo,
    settings: {
      logo: require('@/assets/images/logo.png'),
      ...defaultSettings,
    },
  };
}

// ProLayout 支持的api https://procomponents.ant.design/components/layout
export const layout: RunTimeLayoutConfig = ({ initialState, setInitialState }) => {
  return {
    rightContentRender: () => <RightContent />,
    disableContentMargin: false,
    footerRender: () => <Footer />,
    onPageChange: () => {
      const { location } = history;
      // 如果没有登录，重定向到 login
      // if (!initialState?.currentUser && location.pathname !== loginPath) {
      //   history.push(loginPath);
      // }
    },
    links: isDev
      ? [
        <Link key="openapi" to="/umi/plugin/openapi" target="_blank">
          <LinkOutlined />
          <span>OpenAPI 文档</span>
        </Link>,
        <Link to="/~docs" key="docs">
          <BookOutlined />
          <span>业务组件文档</span>
        </Link>,
      ]
      : [],
    menu: {
      request: async () => {
        const officialDomains = ["indexd.org", "localhost"]
        if (officialDomains.includes(window.location.hostname)) {
          return routes.map((route) => ({
            ...route,
            icon: mapIcon(route.icon || 'home'),
          }));
        } else {
          return routes.filter((route) => route.path !== '/site-map').map((route) => ({
            ...route,
            icon: mapIcon(route.icon || 'home'),
          }));
        }
      },
    },
    menuHeaderRender: undefined,
    // 自定义 403 页面
    // unAccessible: <div>unAccessible</div>,
    // 增加一个 loading 的状态
    childrenRender: (children, props) => {
      // if (initialState?.loading) return <PageLoading />;
      return (
        <>
          {children}
          {/* {!props.location?.pathname?.includes('/login') && (
            // How to hide the settingdrawer? https://github.com/ant-design/ant-design-pro/issues/9608#issue-1141660626
            <SettingDrawer
              prefixCls="hide"
              disableUrlParams
              enableDarkTheme
              settings={initialState?.settings}
              onSettingChange={(settings) => {
                setInitialState((preInitialState) => ({
                  ...preInitialState,
                  settings,
                }));
              }}
            />
          )} */}
        </>
      );
    },
    ...initialState?.settings,
  };
};
