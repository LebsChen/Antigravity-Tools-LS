import { create } from 'zustand';
import { processService } from '../api/services/process';

const useProcessStore = create((set, get) => ({
  processes: [],
  config: {
    max_instances: 5,
    idle_timeout_secs: 1800,
  },
  isLoading: false,
  error: null,

  fetchProcesses: async () => {
    set({ isLoading: true, error: null });
    try {
      const processes = await processService.list();
      // UX: 增加一点延时让加载动画肉眼可见
      await new Promise(resolve => setTimeout(resolve, 300));
      set({ processes, isLoading: false });
    } catch (err) {
      set({ error: err.message, isLoading: false });
    }
  },

  killProcess: async (id) => {
    try {
      await processService.kill(id);
      set((state) => ({
        processes: state.processes.filter((p) => p.id !== id),
      }));
    } catch (err) {
      set({ error: err.message });
      throw err;
    }
  },

  fetchConfig: async () => {
    try {
      const config = await processService.getConfig();
      set({ config });
    } catch (err) {
      console.error('Failed to fetch config:', err);
    }
  },

  updateConfig: async (newConfig) => {
    try {
      await processService.updateConfig(newConfig);
      set({ config: newConfig });
    } catch (err) {
      set({ error: err.message });
      throw err;
    }
  },
}));

export default useProcessStore;
