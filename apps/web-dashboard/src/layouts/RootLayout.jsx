import React, { useEffect } from 'react';
import { Outlet } from 'react-router-dom';
import Sidebar from '../components/Sidebar';
import Header from '../components/Header';
import { ToastContainer } from '../components/Toast';
import useStatsStore from '../store/useStatsStore';
import useAppStore from '../store/useAppStore';

const RootLayout = () => {
  const { fetchMetrics } = useStatsStore();
  const { setSystemStatus } = useAppStore();

  useEffect(() => {
    const syncStatus = async () => {
      try {
        // 核心：调用全局健康检查，确保资产状态（OPTIMAL/DEGRADED）同步
        await useAppStore.getState().checkSystemHealth();
        // 如果需要性能指标可以继续 fetchMetrics，但目前主要依赖 checkSystemHealth
        await fetchMetrics();
      } catch (err) {
        console.error('Global status sync failed:', err);
      }
    };

    syncStatus();
    const interval = setInterval(syncStatus, 15000); // 提高频率到 15s 确保及时性
    return () => clearInterval(interval);
  }, [fetchMetrics]);

  return (
    <div className="flex w-screen h-screen bg-background text-foreground overflow-hidden transition-colors duration-300">
      {/* Fixed Sidebar */}
      <Sidebar />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0">
        <Header />
        
        {/* Scrollable Page Body */}
        <main className="flex-1 overflow-y-auto px-10 pb-10 scrollbar-hide">
          <div className="fade-in max-w-[1600px] mx-auto">
            <Outlet />
          </div>
        </main>
      </div>

      {/* Global Toast Container */}
      <ToastContainer />
    </div>
  );
};

export default RootLayout;
