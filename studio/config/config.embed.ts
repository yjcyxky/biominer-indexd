// https://umijs.org/config/
import { defineConfig } from 'umi';

export default defineConfig({
  outputPath: '../assets/',
  runtimePublicPath: true,
  history: { type: 'hash' },
});
