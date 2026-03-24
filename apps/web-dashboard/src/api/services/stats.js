import apiClient from '../client';

export const statsService = {
  // 获取综合概览
  getSummary: () => apiClient.get('/stats/summary'),

  // 获取趋势数据 (每小时)
  getHourlyTrends: (hours = 24) => apiClient.get(`/stats/hourly?hours=${hours}`),

  // 获取趋势数据 (每日)
  getDailyTrends: (days = 7) => apiClient.get(`/stats/daily?days=${days}`),

  // 获取模型统计
  getModelStats: (hours = 24) => apiClient.get(`/stats/models?hours=${hours}`),

  // 获取账号统计
  getAccountStats: (hours = 24) => apiClient.get(`/stats/accounts?hours=${hours}`),

  // 获取模型趋势 (每小时)
  getModelTrendsHourly: (hours = 24) => apiClient.get(`/stats/model-trends?hours=${hours}`),

  // 获取模型趋势 (每日)
  getModelTrendsDaily: (days = 7) => apiClient.get(`/stats/model-trends-daily?days=${days}`),
};
