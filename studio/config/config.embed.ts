// https://umijs.org/config/
import { defineConfig } from 'umi';

export default defineConfig({
  outputPath: '../assets/',
  publicPath: '/assets/',
  history: { type: 'hash' },
});
