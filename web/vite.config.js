
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import Components from 'unplugin-vue-components/vite';
import {PrimeVueResolver} from 'unplugin-vue-components/resolvers';

// https://vitejs.dev/config/
export default defineConfig({
  server:{
    proxy:{
      '/data/*':'http://127.0.0.1:5050',
      '/api': 'http://127.0.0.1:5050',
    }
  },
  plugins: [
    vue(),
    Components({
      resolvers: [
        PrimeVueResolver()
      ]
    })],
    build:{
      outDir:'../crates/server/build'
    }
})
