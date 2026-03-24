import { create } from 'zustand';
import { accountService } from '../api/services/account';

const useAccountStore = create((set, get) => ({
  accounts: [],
  loading: false,
  error: null,

  // 获取账号列表
  fetchAccounts: async () => {
    set({ loading: true, error: null });
    try {
      const accounts = await accountService.list();
      set({ accounts, loading: false });
    } catch (err) {
      set({ error: err.message, loading: false });
    }
  },

  // 通过 Token 导入 (支持批量)
  importByTokens: async (tokens) => {
    set({ loading: true, error: null });
    let successCount = 0;
    for (const token of tokens) {
      try {
        await accountService.importToken(token);
        successCount++;
      } catch (err) {
        console.error(`Token 导入失败: ${token}`, err);
      }
    }
    await get().fetchAccounts();
    set({ loading: false });
    return successCount;
  },

  // 通过回调 URL 导入
  importByCallbackUrl: async (url) => {
    set({ loading: true, error: null });
    try {
      await accountService.importCallbackUrl(url);
      await get().fetchAccounts();
      set({ loading: false });
      return true;
    } catch (err) {
      set({ error: err.message, loading: false });
      throw err;
    }
  },

  // 删除账号
  removeAccount: async (accountId) => {
    try {
      await accountService.remove(accountId);
      set((state) => ({
        accounts: state.accounts.filter((a) => a.id !== accountId)
      }));
    } catch (err) {
      set({ error: err.message });
      throw err;
    }
  },

  // 刷新单个账号的配额
  refreshQuota: async (token) => {
    try {
      await accountService.refreshQuota(token);
      // 刷新成功后重新拉取列表以更新 UI 数据
      await get().fetchAccounts();
    } catch (err) {
      console.error('刷新配额失败:', err);
      throw err;
    }
  },

  // 导出账号配置 (仅 email 和 token)
  exportAccount: (account) => {
    const config = {
      email: account.email,
      refresh_token: account.refresh_token
    };
    const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${account.email || 'account'}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  },

  // 更新账号标签
  updateAccountLabel: async (accountId, label) => {
    try {
      await accountService.updateLabel(accountId, label);
      // 更新成功后重新获取列表
      await get().fetchAccounts();
    } catch (err) {
      console.error('更新标签失败:', err);
      throw err;
    }
  },

  // 切换账号的代理禁用状态
  toggleProxyStatus: async (accountId, disabled) => {
    try {
      await accountService.updateProxyStatus(accountId, disabled);
      // 更新成功后重新获取列表
      await get().fetchAccounts();
    } catch (err) {
      console.error('切换代理状态失败:', err);
      throw err;
    }
  },

  // 切换到本地 IDE (静默登录)
  switchAccount: async (accountId) => {
    set({ loading: true, error: null });
    try {
      await accountService.switchAccount(accountId);
      // 成功后可重新刷新列表以确保 UI 同步 (虽然主要影响在外部 IDE)
      await get().fetchAccounts();
      set({ loading: false });
      return true;
    } catch (err) {
      const errorMsg = err.response?.data || err.message;
      set({ error: errorMsg, loading: false });
      throw new Error(errorMsg);
    }
  },

  // 重新排序账号
  reorderAccounts: async (accountIds) => {
    const { accounts } = get();

    // 1. 建立 ID 映射并重建数组 (乐观更新)
    const map = new Map(accounts.map(acc => [acc.id, acc]));
    const reorderedAccounts = accountIds.map(id => map.get(id)).filter(Boolean);
    const remainingAccounts = accounts.filter(acc => !accountIds.includes(acc.id));
    const finalAccounts = [...reorderedAccounts, ...remainingAccounts];

    // 乐观更新
    set({ accounts: finalAccounts });

    try {
      await accountService.reorder(accountIds);
    } catch (err) {
      console.error('重新排序失败，正在回滚:', err);
      // 回滚
      set({ accounts });
      throw err;
    }
  }
}));

export default useAccountStore;
