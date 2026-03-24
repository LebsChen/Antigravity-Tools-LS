import { create } from 'zustand';
import { keyService } from '../api/services/key';

const useKeyStore = create((set, get) => ({
  keys: [],
  isLoading: false,
  error: null,

  fetchKeys: async () => {
    set({ isLoading: true, error: null });
    try {
      const keys = await keyService.list();
      await new Promise(resolve => setTimeout(resolve, 300)); // UX: 增加一点延时让加载动画肉眼可见
      set({ keys, isLoading: false });
    } catch (err) {
      set({ error: err.message, isLoading: false });
    }
  },

  createKey: async (name) => {
    set({ isLoading: true, error: null });
    try {
      const newKey = await keyService.create(name);
      set((state) => ({
        keys: [...state.keys, newKey],
        isLoading: false,
      }));
      return newKey;
    } catch (err) {
      set({ error: err.message, isLoading: false });
      throw err;
    }
  },

  deleteKey: async (keyString) => {
    set({ isLoading: true, error: null });
    try {
      await keyService.remove(keyString);
      set((state) => ({
        keys: state.keys.filter((k) => k.key !== keyString),
        isLoading: false,
      }));
    } catch (err) {
      set({ error: err.message, isLoading: false });
      throw err;
    }
  },

  renameKey: async (oldKey, newName, newKey) => {
    try {
      const updated = await keyService.update(oldKey, { name: newName, key: newKey });
      set((state) => ({
        keys: state.keys.map((k) => k.key === oldKey ? updated : k),
      }));
      return updated;
    } catch (err) {
      throw err;
    }
  },
}));

export default useKeyStore;
