import axios from 'axios';

// 动态识别 Tauri 生产网络环境（无 Proxy），强制直开底层 5173
const isTauri = typeof window !== 'undefined' && (!!window.__TAURI__ || window.location.protocol === 'tauri:');
const baseURL = isTauri ? 'http://127.0.0.1:5173/v1' : '/v1';

const apiClient = axios.create({
  baseURL,
  timeout: 300000, // 延长至 5 分钟，适配核心资产下载
  headers: {
    'Content-Type': 'application/json',
  },
});

// 响应拦截器：统一处理数据解包与错误日志
apiClient.interceptors.response.use(
  (response) => {
    // 如果后端返回结构是 { success: true, data: ... }
    if (response.data && typeof response.data === 'object' && 'success' in response.data) {
      if (response.data.success) {
        return response.data.data;
      } else {
        return Promise.reject(new Error(response.data.message || '业务逻辑错误'));
      }
    }
    // 否则直接返回 data (适配部分原始 OpenAI 兼容接口)
    return response.data;
  },
  (error) => {
    const message = error.response?.data?.message || error.message || '网络请求失败';
    console.error('🌐 [API Client Error]:', message);
    return Promise.reject(error);
  }
);

export default apiClient;
