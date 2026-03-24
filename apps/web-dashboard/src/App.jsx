import React, { useEffect } from 'react';
import { HashRouter as BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import useAppStore from './store/useAppStore';
import RootLayout from './layouts/RootLayout';
import Dashboard from './pages/Dashboard';
import Accounts from './pages/Accounts';
import Keys from './pages/Keys';
import Processes from './pages/Processes';
import Stats from './pages/Stats';
import Monitor from './pages/Monitor';
import Integration from './pages/Integration';
import Settings from './pages/Settings';
import About from './pages/About';

function App() {
  const { theme, checkSystemHealth } = useAppStore();
  
  // Initial health check and interval
  useEffect(() => {
    checkSystemHealth();
    const interval = setInterval(checkSystemHealth, 5 * 60 * 1000); // 5 minutes
    return () => clearInterval(interval);
  }, [checkSystemHealth]);

  // Sync theme to HTML root on mount and theme change
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<RootLayout />}>
          <Route index element={<Dashboard />} />
          <Route path="accounts" element={<Accounts />} />
          <Route path="keys" element={<Keys />} />
          <Route path="processes" element={<Processes />} />
          <Route path="stats" element={<Stats />} />
          <Route path="logs" element={<Monitor />} />
          <Route path="integration" element={<Integration />} />
          <Route path="settings" element={<Settings />} />
          <Route path="about" element={<About />} />
          
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
