import apiClient from '../client';

export const accountService = {
  // 获取账号列表
  list: () => apiClient.get('/accounts'),

  // 通过 Token 导入单个人账号
  importToken: (token) => apiClient.post('/accounts/import', {
    refresh_token: token,
    email: null
  }),

  // 通过回调 URL 导入
  importCallbackUrl: (url) => apiClient.post('/auth/callback_url', { url }),

  // 删除账号
  remove: (accountId) => apiClient.delete(`/accounts/${accountId}`),

  // 刷新配额
  refreshQuota: (token) => apiClient.post('/quota', {}, {
    headers: { 'Authorization': `Bearer ${token}` }
  }),

  // 更新账号标签
  updateLabel: (accountId, label) => apiClient.post(`/accounts/${accountId}/label`, { label }),

  // 更新代理禁用状态
  updateProxyStatus: (id, disabled) => apiClient.post(`/accounts/${id}/proxy-status`, { disabled }),
  switchAccount: (id) => apiClient.post(`/accounts/${id}/switch`),
  // 重新排序账号
  reorder: (ids) => apiClient.post('/accounts', { ids }),
};
