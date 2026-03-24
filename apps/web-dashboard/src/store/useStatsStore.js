import { create } from 'zustand';
import apiClient from '../api/client';

export const useStatsStore = create((set, get) => ({
  summary: null,
  trends: [],
  metrics: null,
  modelStats: [],
  accountStats: [],
  faults: [],
  loading: false,
  error: null,
  
  // --- 兼容旧版 Stats.jsx 的字段 ---
  timeRange: '24h',
  modelTrends: [], // 映射到最新的 trends
  trendData: [],   // 历史趋势映射
  isLoading: false, // 映射到 loading

  setTimeRange: (range) => {
    set({ timeRange: range });
    // 切换范围后自动刷新
    get().fetchStats();
  },

  fetchSummary: async () => {
    try {
      const res = await apiClient.get('/stats/summary');
      set({ summary: res });
    } catch (err) {
      console.error('Failed to fetch summary:', err);
    }
  },

  fetchMetrics: async () => {
    try {
      const res = await apiClient.get('/stats/metrics');
      set({ metrics: res });
    } catch (err) {
      console.error('Failed to fetch metrics:', err);
    }
  },

  fetchModelStats: async () => {
    const range = get().timeRange;
    const hours = range === '7d' ? 168 : range === '30d' ? 720 : range === 'all' ? 87600 : 24;
    try {
      const res = await apiClient.get(`/stats/models?hours=${hours}`);
      set({ modelStats: res });
    } catch (err) {
      console.error('Failed to fetch model stats:', err);
    }
  },

  fetchAccountStats: async () => {
    const range = get().timeRange;
    const hours = range === '7d' ? 168 : range === '30d' ? 720 : range === 'all' ? 87600 : 24;
    try {
      const res = await apiClient.get(`/stats/accounts?hours=${hours}`);
      set({ accountStats: res });
    } catch (err) {
      console.error('Failed to fetch account stats:', err);
    }
  },

  fetchModelTrends: async () => {
    const range = get().timeRange;
    const hours = range === '7d' ? 168 : range === '30d' ? 720 : range === 'all' ? 87600 : 24;
    try {
      const res = await apiClient.get(`/stats/model-trends?hours=${hours}`);
      set({ 
        trends: res,
        modelTrends: res,
        trendData: res
      });
    } catch (err) {
      console.error('Failed to fetch model trends:', err);
    }
  },

  fetchFaults: async () => {
    try {
      const res = await apiClient.get('/monitor/logs');
      const errors = Array.isArray(res) ? res.filter(log => log.status >= 400) : [];
      set({ faults: errors.slice(0, 10) });
    } catch (err) {
      console.error('Failed to fetch faults:', err);
    }
  },

  // 综合抓取接口 (对齐旧版)
  fetchStats: async () => {
    set({ isLoading: true });
    await get().fetchAll();
    set({ isLoading: false });
  },

  fetchAll: async () => {
    await Promise.all([
      get().fetchSummary(),
      get().fetchMetrics(),
      get().fetchModelStats(),
      get().fetchAccountStats(),
      get().fetchModelTrends(),
      get().fetchFaults(),
    ]);
  }
}));

export default useStatsStore;
