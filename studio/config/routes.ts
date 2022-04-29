export default [
  {
    path: '/user',
    layout: false,
    routes: [
      {
        path: '/user',
        routes: [
          {
            name: 'login',
            path: '/user/login',
            component: './user/Login',
          },
        ],
      },
      {
        component: './404',
      },
    ],
  },
  {
    path: '/index',
    name: 'home',
    icon: 'home',
    component: './Index',
  },
  {
    name: 'data-repo',
    icon: 'database',
    path: '/data-repo',
    component: './DataRepo',
  },
  {
    path: '/about',
    name: 'about',
    icon: 'question-circle',
    component: './About',
  },
  {
    path: '/',
    redirect: '/index',
  },
  {
    path: '/:guid(biominer.fudan-pgx/[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})',
    component: './DataRepo',
  },
  {
    component: './404',
  },
];
