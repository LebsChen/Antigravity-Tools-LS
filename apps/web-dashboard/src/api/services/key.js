import apiClient from '../client';

export const keyService = {
  // 获取所有 API Keys
  list: () => apiClient.get('/keys'),

  // 创建 API Key
  create: (name = '') => apiClient.post('/keys', { name }),

  // 删除 API Key
  remove: (key) => apiClient.delete(`/keys/${key}`),

  // 重命名或修改 API Key
  update: (oldKey, { name, key }) => apiClient.patch(`/keys/${oldKey}`, { name, key }),
};
