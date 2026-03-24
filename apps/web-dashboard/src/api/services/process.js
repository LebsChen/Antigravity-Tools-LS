import apiClient from '../client';

export const processService = {
  // 获取实例列表
  list: () => apiClient.get('/instances'),

  // 终止实例
  kill: (id) => apiClient.delete(`/instances/${id}`),

  // 获取治理配置
  getConfig: () => apiClient.get('/instances/config'),

  // 更新治理配置
  updateConfig: (config) => apiClient.patch('/instances/config', config),
};
